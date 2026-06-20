use crate::models::CommandSpec;
use std::path::{Path, PathBuf};

const ALLOWED_PROGRAMS: &[&str] = &["codex", "claude", "cursor-agent"];

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

pub fn validate_command_spec(spec: &CommandSpec) -> Result<Option<PathBuf>, String> {
    // 当前三家 CLI 都 cd=true；cd=false 仅保留给未来不依赖工作目录的命令。
    let cwd = if spec.cd {
        Some(validate_cwd(&spec.cwd)?)
    } else {
        None
    };
    validate_program(&spec.program)?;
    if let Some(id_arg) = spec.args.last() {
        if !matches!(id_arg.as_str(), "resume" | "--resume" | "--last") {
            validate_session_id(id_arg)?;
        }
    }
    Ok(cwd)
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
