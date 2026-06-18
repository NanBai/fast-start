use crate::models::{CliType, Session};
use std::path::PathBuf;

pub mod claude_code;
pub mod codex;

#[derive(Debug)]
pub enum ScanError {
    Io(std::io::Error),
    Parse(String),
    NotFound(String),
}

impl ScanError {
    pub fn message(&self) -> String {
        match self {
            ScanError::Io(err) => err.to_string(),
            ScanError::Parse(msg) => msg.clone(),
            ScanError::NotFound(msg) => msg.clone(),
        }
    }
}

impl From<std::io::Error> for ScanError {
    fn from(value: std::io::Error) -> Self {
        ScanError::Io(value)
    }
}

pub trait SessionScanner: Send + Sync {
    fn cli_type(&self) -> CliType;
    fn scan_sessions(&self) -> Result<Vec<Session>, ScanError>;
}

pub fn scanners() -> Vec<Box<dyn SessionScanner + Send + Sync>> {
    vec![
        Box::new(codex::CodexScanner::default()),
        Box::new(claude_code::ClaudeCodeScanner::default()),
    ]
}

pub fn decode_claude_project_dir(encoded: &str) -> Option<PathBuf> {
    let trimmed = encoded.strip_prefix('-')?;
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(format!("/{}", trimmed.replace('-', "/"))))
}

pub fn command_spec_for_session(session: &Session) -> Result<crate::models::CommandSpec, String> {
    use crate::models::{CliType, CommandSpec};
    use crate::security::validate_session_id;

    validate_session_id(&session.session_id)?;

    let (program, args) = match session.cli_type {
        CliType::Codex => (
            "codex",
            vec!["resume".to_string(), session.session_id.clone()],
        ),
        CliType::ClaudeCode => (
            "claude",
            vec!["--resume".to_string(), session.session_id.clone()],
        ),
        CliType::Cursor => (
            "cursor-agent",
            vec!["--resume".to_string(), session.session_id.clone()],
        ),
    };

    Ok(CommandSpec {
        cwd: session.project_dir.clone(),
        program: program.to_string(),
        args,
    })
}

#[cfg(test)]
mod tests {
    use super::decode_claude_project_dir;

    #[test]
    fn decode_claude_project_dir_works_for_simple_paths() {
        assert_eq!(
            decode_claude_project_dir("-Users-xb-Desktop-dev"),
            Some(std::path::PathBuf::from("/Users/xb/Desktop/dev"))
        );
    }

    #[test]
    fn local_scanners_find_sessions_when_installed() {
        use super::claude_code::ClaudeCodeScanner;
        use super::codex::CodexScanner;
        use super::SessionScanner;

        let codex = CodexScanner.scan_sessions();
        let claude = ClaudeCodeScanner.scan_sessions();

        if let Ok(items) = &codex {
            assert!(items.iter().all(|session| !session.session_id.is_empty()));
        }
        if let Ok(items) = &claude {
            assert!(items
                .iter()
                .all(|session| session.project_dir.is_absolute()));
        }

        assert!(codex.is_ok() || claude.is_ok());
    }
}
