use crate::models::CommandSpec;
use std::path::{Path, PathBuf};

const ALLOWED_PROGRAMS: &[&str] = &["codex", "claude", "cursor-agent", "grok", "opencode", "omp"];

pub fn validate_program(program: &str) -> Result<(), String> {
    if ALLOWED_PROGRAMS.contains(&program) {
        Ok(())
    } else {
        Err(format!("不允许的程序: {program}"))
    }
}

pub fn validate_session_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("session id 不能为空".to_string());
    }

    let valid = id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'));
    if valid {
        Ok(())
    } else {
        Err("session id 含非法字符".to_string())
    }
}

pub fn validate_cwd(path: &Path) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(path).map_err(|_| "工作目录不存在".to_string())?;
    if !canonical.is_dir() {
        return Err("工作目录不存在".to_string());
    }
    Ok(canonical)
}

/// 校验 CommandSpec：cwd、program 白名单、**完整 argv 形状**（不允许未知 flag / 多余参数）。
pub fn validate_command_spec(spec: &CommandSpec) -> Result<Option<PathBuf>, String> {
    // 当前各 CLI 都 cd=true；cd=false 仅保留给未来不依赖工作目录的命令。
    let cwd = if spec.cd {
        Some(validate_cwd(&spec.cwd)?)
    } else {
        None
    };
    validate_program(&spec.program)?;
    validate_resume_args(&spec.program, &spec.args)?;
    Ok(cwd)
}

/// 按已知 CLI 的 resume 形状校验 args，拒绝「只校 last」带来的隐式约定。
/// - codex: `resume <id>`
/// - claude / cursor-agent / grok: `--resume <id>`
/// - opencode: `--session <id>`
fn validate_resume_args(program: &str, args: &[String]) -> Result<(), String> {
    match program {
        "codex" => match args {
            [verb, id] if verb == "resume" => validate_session_id(id),
            _ => Err("codex 参数形状无效，期望: resume <session-id>".to_string()),
        },
        "claude" | "cursor-agent" | "grok" => match args {
            [flag, id] if flag == "--resume" => validate_session_id(id),
            _ => Err(format!(
                "{program} 参数形状无效，期望: --resume <session-id>"
            )),
        },
        "opencode" => match args {
            [flag, id] if flag == "--session" => validate_session_id(id),
            _ => Err("opencode 参数形状无效，期望: --session <session-id>".to_string()),
        },
        "omp" => match args {
            [flag, id] if flag == "-r" || flag == "--resume" => validate_session_id(id),
            _ => Err("omp 参数形状无效，期望: -r <session-id> 或 --resume <session-id>".to_string()),
        },
        other => Err(format!("不允许的程序: {other}")),
    }
}

pub fn applescript_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub fn shell_escape(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.' | '/'))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{validate_command_spec, validate_resume_args};
    use crate::models::CommandSpec;
    use std::path::PathBuf;

    #[test]
    fn accepts_known_resume_shapes() {
        validate_resume_args("codex", &["resume".into(), "abc-123".into()]).unwrap();
        validate_resume_args("claude", &["--resume".into(), "abc-123".into()]).unwrap();
        validate_resume_args("cursor-agent", &["--resume".into(), "abc-123".into()]).unwrap();
        validate_resume_args("grok", &["--resume".into(), "abc-123".into()]).unwrap();
        validate_resume_args("opencode", &["--session".into(), "ses_abc123".into()]).unwrap();
        validate_resume_args("omp", &["-r".into(), "abc123".into()]).unwrap();
        validate_resume_args("omp", &["--resume".into(), "abc123".into()]).unwrap();
    }

    #[test]
    fn rejects_unknown_flags_extra_args_and_bad_ids() {
        assert!(validate_resume_args("codex", &["--resume".into(), "abc".into()]).is_err());
        assert!(validate_resume_args("claude", &["resume".into(), "abc".into()]).is_err());
        assert!(validate_resume_args(
            "grok",
            &["--resume".into(), "abc".into(), "extra".into()]
        )
        .is_err());
        assert!(validate_resume_args("codex", &["resume".into(), "bad;id".into()]).is_err());
        assert!(validate_resume_args("codex", &[]).is_err());
    }

    #[test]
    fn validate_command_spec_checks_program_and_args() {
        let temp = tempfile::tempdir().unwrap();
        let ok = CommandSpec {
            cwd: temp.path().to_path_buf(),
            program: "grok".into(),
            args: vec!["--resume".into(), "019f559d-97a5-7ac0-9e2b-3c340dd33d6b".into()],
            cd: true,
        };
        assert!(validate_command_spec(&ok).is_ok());

        let bad = CommandSpec {
            cwd: temp.path().to_path_buf(),
            program: "grok".into(),
            args: vec!["--resume".into(), "id".into(), "--evil".into()],
            cd: true,
        };
        assert!(validate_command_spec(&bad).is_err());

        let bad_prog = CommandSpec {
            cwd: temp.path().to_path_buf(),
            program: "rm".into(),
            args: vec!["--resume".into(), "abc".into()],
            cd: true,
        };
        assert!(validate_command_spec(&bad_prog).is_err());

        let _ = PathBuf::from("/tmp"); // keep import used in older rustc if needed
    }
}
