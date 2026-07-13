//! 启动预检：只读检查 cwd / program PATH / 源；有 block 时 launch 不得写 wrapper。

use crate::launcher;
use crate::models::{LaunchCommandPreview, Session};
use crate::scanner::command_spec_for_session;
use crate::security::{validate_program, validate_session_id};
use crate::session_source::{check_session_source, SourceCheck};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreflightSeverity {
    Block,
    Warn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightCheck {
    pub code: String,
    pub severity: PreflightSeverity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreflightResult {
    pub session_list_id: String,
    pub ok: bool,
    pub checks: Vec<PreflightCheck>,
    pub preview: Option<LaunchCommandPreview>,
}

/// 可注入的 program 解析，生产实现复用 launcher login PATH 缓存。
pub trait ResolveProgram {
    fn resolve(&self, program: &str) -> Option<PathBuf>;
}

/// 生产用：login shell PATH + `~/.grok/bin`（与 wrapper 注入语义一致）。
pub struct LoginPathProgramResolver;

impl ResolveProgram for LoginPathProgramResolver {
    fn resolve(&self, program: &str) -> Option<PathBuf> {
        launcher::resolve_program_on_launch_path(program)
    }
}

/// 纯函数预检。未知 session（`session=None`）返回 Ok 语义的 block，不 panic。
pub fn preflight_session(
    session_list_id: &str,
    session: Option<&Session>,
    ops_ready: bool,
    path_resolver: &impl ResolveProgram,
) -> PreflightResult {
    let mut checks = Vec::new();

    let Some(session) = session else {
        checks.push(block(
            "session_not_found",
            "未找到对应 session，请刷新列表后重试",
        ));
        return finish(session_list_id, checks, None);
    };

    // cwd
    let cwd = Path::new(&session.project_dir);
    if !cwd.exists() {
        checks.push(block(
            "cwd_missing",
            format!(
                "工作目录不存在：{}",
                session.project_dir.to_string_lossy()
            ),
        ));
    } else if !cwd.is_dir() {
        checks.push(block(
            "cwd_not_dir",
            format!(
                "工作目录不是文件夹：{}",
                session.project_dir.to_string_lossy()
            ),
        ));
    }

    // session_id / CommandSpec 形状（不走 validate_command_spec 的 cwd 存在性，避免与 cwd_* 重复）
    if validate_session_id(&session.session_id).is_err() {
        checks.push(block(
            "invalid_session_id",
            "session id 无效，无法构造恢复命令",
        ));
    }

    let preview = match command_spec_for_session(session) {
        Ok(spec) => {
            if validate_program(&spec.program).is_err() {
                checks.push(block(
                    "invalid_spec",
                    format!("不允许的启动程序：{}", spec.program),
                ));
            } else if path_resolver.resolve(&spec.program).is_none() {
                checks.push(block(
                    "program_not_found",
                    format!(
                        "未在终端 PATH 上找到程序「{}」，请确认已安装并在登录 shell 中可用",
                        spec.program
                    ),
                ));
            }
            Some(LaunchCommandPreview {
                cwd: spec.cwd.to_string_lossy().to_string(),
                program: spec.program,
                args: spec.args,
                cd: spec.cd,
            })
        }
        Err(err) => {
            // command_spec 当前仅因 session_id 失败；若已记 invalid_session_id 则不重复
            if !checks.iter().any(|c| c.code == "invalid_session_id") {
                checks.push(block("invalid_spec", format!("无法构造启动命令：{err}")));
            }
            None
        }
    };

    // 源探测（与 inspect 同源）
    match check_session_source(session, ops_ready) {
        SourceCheck::Unverified => {
            checks.push(warn(
                "source_unverified",
                "会话源尚未校验（可能仍在缓存窗口），启动不因源拦截",
            ));
        }
        SourceCheck::Missing => {
            checks.push(block(
                "source_missing",
                "会话源文件/记录已不存在，请刷新列表或删除该条目",
            ));
        }
        SourceCheck::Present { .. } => {}
    }

    finish(session_list_id, checks, preview)
}

/// 任一 block 的中文 message 拼接，供 launch_session Err 使用。
pub fn block_messages(result: &PreflightResult) -> Option<String> {
    let msgs: Vec<&str> = result
        .checks
        .iter()
        .filter(|c| c.severity == PreflightSeverity::Block)
        .map(|c| c.message.as_str())
        .collect();
    if msgs.is_empty() {
        None
    } else {
        Some(msgs.join("；"))
    }
}

fn finish(
    session_list_id: &str,
    checks: Vec<PreflightCheck>,
    preview: Option<LaunchCommandPreview>,
) -> PreflightResult {
    let ok = !checks
        .iter()
        .any(|c| c.severity == PreflightSeverity::Block);
    PreflightResult {
        session_list_id: session_list_id.to_string(),
        ok,
        checks,
        preview,
    }
}

fn block(code: &str, message: impl Into<String>) -> PreflightCheck {
    PreflightCheck {
        code: code.to_string(),
        severity: PreflightSeverity::Block,
        message: message.into(),
    }
}

fn warn(code: &str, message: impl Into<String>) -> PreflightCheck {
    PreflightCheck {
        code: code.to_string(),
        severity: PreflightSeverity::Warn,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        block_messages, preflight_session, PreflightSeverity, ResolveProgram,
    };
    use crate::models::{
        CliType, Session, SessionDeleteKind, SessionDeleteTarget,
    };
    use chrono::Utc;
    use rusqlite::Connection;
    use std::collections::HashMap;
    use std::fs;
    use std::path::{PathBuf, Path};

    struct MapResolver(HashMap<String, PathBuf>);

    impl ResolveProgram for MapResolver {
        fn resolve(&self, program: &str) -> Option<PathBuf> {
            self.0.get(program).cloned()
        }
    }

    fn resolver_with(programs: &[&str]) -> MapResolver {
        let mut map = HashMap::new();
        for p in programs {
            map.insert((*p).to_string(), PathBuf::from(format!("/bin/{p}")));
        }
        MapResolver(map)
    }

    fn make_session(
        cli: CliType,
        session_id: &str,
        project: PathBuf,
        target: Option<SessionDeleteTarget>,
    ) -> Session {
        Session {
            id: Session::stable_id(cli, session_id, &project),
            cli_type: cli,
            session_id: session_id.to_string(),
            project_dir: project.clone(),
            project_name: Session::project_name_from_dir(&project),
            last_active_at: Utc::now(),
            summary: None,
            delete_target: target,
        }
    }

    fn codes(result: &super::PreflightResult) -> Vec<(&str, PreflightSeverity)> {
        result
            .checks
            .iter()
            .map(|c| (c.code.as_str(), c.severity))
            .collect()
    }

    #[test]
    fn unknown_session_is_ok_result_with_block() {
        let r = preflight_session("missing-id", None, true, &resolver_with(&["codex"]));
        assert!(!r.ok);
        assert_eq!(r.session_list_id, "missing-id");
        assert!(r.preview.is_none());
        assert!(codes(&r).contains(&("session_not_found", PreflightSeverity::Block)));
        assert!(block_messages(&r).unwrap().contains("未找到"));
    }

    #[test]
    fn cwd_missing_blocks() {
        let project = PathBuf::from("/tmp/definitely-does-not-exist-fast-start-preflight");
        let s = make_session(CliType::Codex, "abc-123", project, None);
        let r = preflight_session(&s.id, Some(&s), false, &resolver_with(&["codex"]));
        assert!(!r.ok);
        assert!(codes(&r).contains(&("cwd_missing", PreflightSeverity::Block)));
    }

    #[test]
    fn cwd_not_dir_blocks() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("not-dir");
        fs::write(&file, b"x").unwrap();
        let s = make_session(CliType::Codex, "abc-123", file, None);
        let r = preflight_session(&s.id, Some(&s), false, &resolver_with(&["codex"]));
        assert!(!r.ok);
        assert!(codes(&r).contains(&("cwd_not_dir", PreflightSeverity::Block)));
    }

    #[test]
    fn cache_window_source_unverified_warn_does_not_block_when_rest_ok() {
        let temp = tempfile::tempdir().unwrap();
        let s = make_session(CliType::Codex, "abc-123", temp.path().to_path_buf(), None);
        let r = preflight_session(&s.id, Some(&s), false, &resolver_with(&["codex"]));
        assert!(r.ok, "ops_ready=false 不应因源拦截: {:?}", r.checks);
        assert!(codes(&r).contains(&("source_unverified", PreflightSeverity::Warn)));
        assert!(r.preview.is_some());
        assert!(block_messages(&r).is_none());
    }

    #[test]
    fn program_not_found_blocks() {
        let temp = tempfile::tempdir().unwrap();
        let s = make_session(CliType::Codex, "abc-123", temp.path().to_path_buf(), None);
        let r = preflight_session(&s.id, Some(&s), false, &resolver_with(&[]));
        assert!(!r.ok);
        assert!(codes(&r).contains(&("program_not_found", PreflightSeverity::Block)));
    }

    #[test]
    fn opencode_row_missing_blocks_even_if_db_exists() {
        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().join("proj");
        fs::create_dir_all(&cwd).unwrap();
        let db = temp.path().join("opencode.db");
        let conn = Connection::open(&db).unwrap();
        conn.execute_batch(
            "CREATE TABLE session (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                slug TEXT NOT NULL,
                directory TEXT NOT NULL,
                title TEXT NOT NULL,
                version TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL
            );",
        )
        .unwrap();
        // db 在，行不在
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: db,
            kind: SessionDeleteKind::File,
        };
        let s = make_session(
            CliType::OpenCode,
            "ses_gone",
            cwd,
            Some(target),
        );
        let r = preflight_session(&s.id, Some(&s), true, &resolver_with(&["opencode"]));
        assert!(!r.ok);
        assert!(codes(&r).contains(&("source_missing", PreflightSeverity::Block)));
    }

    #[test]
    fn healthy_session_ok_with_preview() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("s.jsonl");
        fs::write(&file, b"{}").unwrap();
        let target = SessionDeleteTarget {
            root: temp.path().to_path_buf(),
            path: file,
            kind: SessionDeleteKind::File,
        };
        let s = make_session(
            CliType::Codex,
            "abc-123",
            temp.path().to_path_buf(),
            Some(target),
        );
        let r = preflight_session(&s.id, Some(&s), true, &resolver_with(&["codex"]));
        assert!(r.ok, "{:?}", r.checks);
        let preview = r.preview.expect("preview");
        assert_eq!(preview.program, "codex");
        assert_eq!(preview.args, vec!["resume", "abc-123"]);
        assert!(preview.cd);
        assert_eq!(Path::new(&preview.cwd), temp.path());
    }
}
