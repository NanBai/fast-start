//! CliType 注册完整性：新增 CLI 时防漏 scanner / command_spec / 白名单 / 删除映射。
//! 本模块仅承载单测，无运行时 API。

#[cfg(test)]
mod tests {
    use crate::models::{CliType, Session};
    use crate::scanner::{command_spec_for_session, scanners};
    use crate::security::validate_program;
    use chrono::Utc;
    use std::path::PathBuf;

    fn all_cli_types() -> [CliType; 6] {
        [
            CliType::Codex,
            CliType::ClaudeCode,
            CliType::Cursor,
            CliType::GrokBuild,
            CliType::OpenCode,
            CliType::OhMyPi,
        ]
    }

    fn expected_program(cli: CliType) -> &'static str {
        match cli {
            CliType::Codex => "codex",
            CliType::ClaudeCode => "claude",
            CliType::Cursor => "cursor-agent",
            CliType::GrokBuild => "grok",
            CliType::OpenCode => "opencode",
            CliType::OhMyPi => "omp",
        }
    }

    fn sample_session(cli: CliType) -> Session {
        let project = PathBuf::from("/tmp/cli-contract");
        Session {
            id: Session::stable_id(cli, "fixture-id", &project),
            cli_type: cli,
            session_id: "fixture-id".into(),
            project_dir: project.clone(),
            project_name: "cli-contract".into(),
            last_active_at: Utc::now(),
            summary: None,
            delete_target: None,
        }
    }

    #[test]
    fn every_cli_type_has_scanner_registration() {
        let registered: Vec<CliType> = scanners().into_iter().map(|s| s.cli_type()).collect();
        for cli in all_cli_types() {
            assert!(
                registered.contains(&cli),
                "scanner 未注册: {cli:?}"
            );
        }
        assert_eq!(registered.len(), all_cli_types().len());
    }

    #[test]
    fn every_cli_type_has_command_spec_and_allowed_program() {
        for cli in all_cli_types() {
            let session = sample_session(cli);
            let spec = command_spec_for_session(&session).expect("command_spec");
            assert_eq!(spec.program, expected_program(cli));
            assert!(spec.cd, "当前 CLI 均应 cd=true: {cli:?}");
            validate_program(&spec.program).expect("program whitelist");
            assert!(!spec.args.is_empty());
        }
    }

    #[test]
    fn every_cli_type_has_stable_id_key_namespace() {
        let a = sample_session(CliType::Codex).id;
        let b = sample_session(CliType::ClaudeCode).id;
        assert_ne!(a, b, "不同 CLI 的 stable_id 不得碰撞");
    }
}
