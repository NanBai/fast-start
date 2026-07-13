use crate::models::{CliType, Session};
use std::path::PathBuf;

pub mod claude_code;
pub mod codex;
pub mod cursor;
pub mod grok_build;
pub mod opencode;

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
        Box::new(cursor::CursorScanner::default()),
        Box::new(grok_build::GrokBuildScanner::default()),
        Box::new(opencode::OpenCodeScanner::default()),
    ]
}

pub fn decode_claude_project_dir(encoded: &str) -> Option<PathBuf> {
    let trimmed = encoded.strip_prefix('-')?;
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(format!("/{}", trimmed.replace('-', "/"))))
}

/// 列表摘要最大 Unicode 标量长度（硬截断只发生在本函数）。
pub const SUMMARY_MAX_CHARS: usize = 160;

/// 把一份原始简介文本（claude 的 aiTitle / lastPrompt、codex 的首条用户消息）
/// 规整成适合列表展示的短串：去首尾空白、折成单行、压连续空白、丢空串、
/// 再按 Unicode 标量 ≤ [`SUMMARY_MAX_CHARS`] 截断。
/// 拿到 None 时调用方应回退到 project_name。
pub fn clean_summary(raw: Option<&str>) -> Option<String> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return None;
    }
    let collapsed: String = raw.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return None;
    }
    let count = collapsed.chars().count();
    if count <= SUMMARY_MAX_CHARS {
        Some(collapsed)
    } else {
        Some(collapsed.chars().take(SUMMARY_MAX_CHARS).collect())
    }
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
        CliType::GrokBuild => (
            "grok",
            vec!["--resume".to_string(), session.session_id.clone()],
        ),
        CliType::OpenCode => (
            "opencode",
            vec!["--session".to_string(), session.session_id.clone()],
        ),
    };

    // 各 CLI 都是"cd 到工作目录 && resume/continue <id>"模式：
    // codex/claude/grok/opencode 的 id 虽全局唯一，但 cd 到原目录方便用户继续操作；
    // cursor 的 chatId 是 workspace 范围的，必须 cd 到正确目录 resume 才生效。
    Ok(CommandSpec {
        cwd: session.project_dir.clone(),
        program: program.to_string(),
        args,
        cd: true,
    })
}

#[cfg(test)]
mod tests {
    use super::{clean_summary, decode_claude_project_dir, SUMMARY_MAX_CHARS};

    #[test]
    fn decode_claude_project_dir_works_for_simple_paths() {
        assert_eq!(
            decode_claude_project_dir("-Users-xb-Desktop-dev"),
            Some(std::path::PathBuf::from("/Users/xb/Desktop/dev"))
        );
    }

    #[test]
    fn clean_summary_trims_and_collapses_whitespace() {
        assert_eq!(
            clean_summary(Some("  hello\n\tworld  ")).as_deref(),
            Some("hello world")
        );
        assert_eq!(clean_summary(Some("   ")), None);
        assert_eq!(clean_summary(None), None);
    }

    #[test]
    fn clean_summary_truncates_to_160_unicode_scalars() {
        let long: String = "字".repeat(200);
        let out = clean_summary(Some(&long)).unwrap();
        assert_eq!(out.chars().count(), SUMMARY_MAX_CHARS);
        assert_eq!(out, "字".repeat(SUMMARY_MAX_CHARS));

        let ascii: String = "a".repeat(161);
        let out = clean_summary(Some(&ascii)).unwrap();
        assert_eq!(out.len(), SUMMARY_MAX_CHARS);
        assert_eq!(out.chars().count(), SUMMARY_MAX_CHARS);
    }

    #[test]
    fn command_spec_always_cds_to_session_project_dir() {
        use super::command_spec_for_session;
        use crate::models::{CliType, Session};
        use chrono::Utc;
        use std::path::PathBuf;

        for cli_type in [
            CliType::Codex,
            CliType::ClaudeCode,
            CliType::Cursor,
            CliType::GrokBuild,
            CliType::OpenCode,
        ] {
            let session = Session {
                id: format!("{cli_type:?}"),
                cli_type,
                session_id: "abc-123".to_string(),
                project_dir: PathBuf::from("/tmp"),
                project_name: "tmp".to_string(),
                last_active_at: Utc::now(),
                summary: None,
                delete_target: None,
            };
            let spec = command_spec_for_session(&session).unwrap();
            assert!(spec.cd);
            assert_eq!(spec.cwd, PathBuf::from("/tmp"));
        }
    }

    #[test]
    fn command_spec_for_grok_build_uses_grok_resume() {
        use super::command_spec_for_session;
        use crate::models::{CliType, Session};
        use chrono::Utc;
        use std::path::PathBuf;

        let session = Session {
            id: "grok".to_string(),
            cli_type: CliType::GrokBuild,
            session_id: "019f559d-97a5-7ac0-9e2b-3c340dd33d6b".to_string(),
            project_dir: PathBuf::from("/tmp/project"),
            project_name: "project".to_string(),
            last_active_at: Utc::now(),
            summary: None,
            delete_target: None,
        };
        let spec = command_spec_for_session(&session).unwrap();
        assert_eq!(spec.program, "grok");
        assert_eq!(
            spec.args,
            vec![
                "--resume".to_string(),
                "019f559d-97a5-7ac0-9e2b-3c340dd33d6b".to_string()
            ]
        );
        assert!(spec.cd);
    }

    #[test]
    fn command_spec_for_opencode_uses_session_flag() {
        use super::command_spec_for_session;
        use crate::models::{CliType, Session};
        use chrono::Utc;
        use std::path::PathBuf;

        let session = Session {
            id: "oc".to_string(),
            cli_type: CliType::OpenCode,
            session_id: "ses_abc123".to_string(),
            project_dir: PathBuf::from("/tmp/project"),
            project_name: "project".to_string(),
            last_active_at: Utc::now(),
            summary: None,
            delete_target: None,
        };
        let spec = command_spec_for_session(&session).unwrap();
        assert_eq!(spec.program, "opencode");
        assert_eq!(
            spec.args,
            vec!["--session".to_string(), "ses_abc123".to_string()]
        );
        assert!(spec.cd);
    }

    #[test]
    fn scanner_roots_can_be_fixture_backed() {
        use super::claude_code::ClaudeCodeScanner;
        use super::codex::CodexScanner;
        use super::cursor::CursorScanner;
        use super::grok_build::GrokBuildScanner;
        use super::opencode::OpenCodeScanner;

        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let _ = CodexScanner::with_root(root.join("codex"));
        let _ = ClaudeCodeScanner::with_root(root.join("claude"));
        let _ = CursorScanner::with_root(root.join("cursor"));
        let _ = GrokBuildScanner::with_root(root.join("grok"));
        let _ = OpenCodeScanner::with_db(root.join("opencode.db"));
    }
}
