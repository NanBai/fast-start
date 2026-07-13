//! Session 查找 / 预检 / 启动 / 删除 / 最近启动与偏好 getter。
use super::{
    normalize_id_list, normalize_project_dirs, normalize_project_dirs_for_sessions,
    normalize_session_ids_for_sessions, AppState, BULK_DELETE_LIMIT, RECENT_LAUNCHES_LIMIT,
};
use crate::launch_preflight::{
    block_messages, preflight_session, LoginPathProgramResolver, PreflightResult,
};
use crate::launcher::{launcher_for, launchers, LaunchError};
use crate::models::{
    BulkDeleteFailure, BulkDeleteResult, CliScanError, LaunchCommandPreview, LaunchMode,
    RecentLaunch, ScanResponse, Session, TerminalType, ThemeMode,
};
use crate::scanner::command_spec_for_session;
use crate::session_delete::delete_session_target;
use crate::session_health::{
    inspect_one, SessionHealthReport, INSPECT_SESSION_HEALTH_LIMIT,
};
use crate::state::scan_cache::{save_scan_cache, snapshot_from_sessions};
use chrono::Utc;

impl AppState {
    /// 按列表稳定 id（`Session.id`）查找，**不是** CLI 原始 `session_id`。
    pub fn find_session(&self, session_list_id: &str) -> Result<Session, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard
            .sessions
            .iter()
            .find(|session| session.id == session_list_id)
            .cloned()
            .ok_or_else(|| "未找到对应 session".to_string())
    }

    pub fn preferred_terminal(&self) -> Result<TerminalType, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.preferred_terminal)
    }

    pub fn set_preferred_terminal(&self, terminal: TerminalType) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.preferred_terminal = terminal;
        Ok(())
    }

    pub fn launch_mode(&self) -> Result<LaunchMode, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.launch_mode)
    }

    pub fn set_launch_mode(&self, mode: LaunchMode) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.launch_mode = mode;
        Ok(())
    }

    pub fn theme_mode(&self) -> Result<ThemeMode, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.theme_mode)
    }

    pub fn set_theme_mode(&self, mode: ThemeMode) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.theme_mode = mode;
        Ok(())
    }

    pub fn favorite_project_dirs(&self) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.favorite_project_dirs.clone())
    }

    pub fn sanitize_favorite_project_dirs(
        &self,
        project_dirs: Vec<String>,
    ) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(normalize_project_dirs_for_sessions(
            project_dirs,
            &guard.sessions,
        ))
    }

    pub fn set_favorite_project_dirs(&self, project_dirs: Vec<String>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.favorite_project_dirs = normalize_project_dirs(project_dirs);
        Ok(())
    }

    pub fn favorite_session_ids(&self) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.favorite_session_ids.clone())
    }

    pub fn sanitize_favorite_session_ids(
        &self,
        session_ids: Vec<String>,
    ) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(normalize_session_ids_for_sessions(
            session_ids,
            &guard.sessions,
        ))
    }

    pub fn set_favorite_session_ids(&self, session_ids: Vec<String>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.favorite_session_ids = normalize_id_list(session_ids);
        Ok(())
    }

    /// 只读启动预检；未知 id 仍 Ok（业务结果在 `PreflightResult` 内）。
    pub fn preflight_launch(&self, session_list_id: &str) -> Result<PreflightResult, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        let session = guard
            .sessions
            .iter()
            .find(|session| session.id == session_list_id);
        Ok(preflight_session(
            session_list_id,
            session,
            guard.ops_ready,
            &LoginPathProgramResolver,
        ))
    }

    /// 只读健康探测。超过 200 ids → Err；未知 id 跳过。
    pub fn inspect_session_health(
        &self,
        session_list_ids: &[String],
    ) -> Result<SessionHealthReport, String> {
        if session_list_ids.len() > INSPECT_SESSION_HEALTH_LIMIT {
            return Err(format!(
                "单次健康探测最多 {} 条，当前 {}",
                INSPECT_SESSION_HEALTH_LIMIT,
                session_list_ids.len()
            ));
        }
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        let mut items = Vec::with_capacity(session_list_ids.len());
        for id in session_list_ids {
            if let Some(session) = guard.sessions.iter().find(|s| s.id == *id) {
                items.push(inspect_one(session, guard.ops_ready));
            }
        }
        Ok(SessionHealthReport { items })
    }

    /// `session_list_id` = `Session.id`（列表稳定 id）。成功时返回 session 供写入最近记录。
    /// 有 preflight block 时返回中文 Err 且不写 wrapper / 不启动。
    pub fn launch_session(&self, session_list_id: &str) -> Result<Session, String> {
        let preflight = self.preflight_launch(session_list_id)?;
        if let Some(msg) = block_messages(&preflight) {
            return Err(msg);
        }

        let session = self.find_session(session_list_id)?;
        let preferred = self.preferred_terminal()?;
        let mode = self.launch_mode()?;
        let launcher = launcher_for(preferred).ok_or_else(|| "终端类型不受支持".to_string())?;

        if !launcher.is_available() {
            return Err("所选终端不可用".to_string());
        }

        // Terminal.app 不支持开 tab：选了 NewTab 时回退到 NewWindow 并提示。
        if mode == LaunchMode::NewTab && !launcher.supports_tab() {
            launcher
                .launch(&command_spec_for_session(&session)?, LaunchMode::NewWindow)
                .map_err(|err: LaunchError| err.message())?;
            return Ok(session);
        }

        let spec = command_spec_for_session(&session)?;
        launcher
            .launch(&spec, mode)
            .map_err(|err: LaunchError| err.message())?;
        Ok(session)
    }

    pub fn preview_launch_command(
        &self,
        session_list_id: &str,
    ) -> Result<LaunchCommandPreview, String> {
        // 与 preflight preview 同一 CommandSpec 组装路径
        let preflight = self.preflight_launch(session_list_id)?;
        if let Some(preview) = preflight.preview {
            return Ok(preview);
        }
        let session = self.find_session(session_list_id)?;
        let spec = command_spec_for_session(&session)?;
        Ok(LaunchCommandPreview {
            cwd: spec.cwd.to_string_lossy().to_string(),
            program: spec.program,
            args: spec.args,
            cd: spec.cd,
        })
    }

    pub fn set_recent_launches(&self, launches: Vec<RecentLaunch>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.recent_launches = launches;
        Ok(())
    }

    /// 成功启动后写入；同 session 去重置顶；上限 RECENT_LAUNCHES_LIMIT。
    pub fn record_recent_launch(&self, session: &Session) -> Result<Vec<RecentLaunch>, String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        let entry = RecentLaunch {
            session_list_id: session.id.clone(),
            cli_type: session.cli_type,
            project_name: session.project_name.clone(),
            project_dir: session.project_dir.to_string_lossy().to_string(),
            summary: session.summary.clone(),
            launched_at: Utc::now(),
        };
        guard
            .recent_launches
            .retain(|item| item.session_list_id != entry.session_list_id);
        guard.recent_launches.insert(0, entry);
        if guard.recent_launches.len() > RECENT_LAUNCHES_LIMIT {
            guard.recent_launches.truncate(RECENT_LAUNCHES_LIMIT);
        }
        Ok(guard.recent_launches.clone())
    }

    /// 剔除已不在 sessions 中的历史项。
    /// 返回 `(launches, changed)`：`changed=true` 时调用方应落盘。
    pub fn sanitize_recent_launches(&self) -> Result<(Vec<RecentLaunch>, bool), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        // 尚未扫描时不清理，避免冷启动把历史清空。
        if guard.sessions.is_empty() {
            return Ok((guard.recent_launches.clone(), false));
        }
        let before = guard.recent_launches.len();
        let allowed: std::collections::HashSet<String> =
            guard.sessions.iter().map(|s| s.id.clone()).collect();
        guard
            .recent_launches
            .retain(|item| allowed.contains(&item.session_list_id));
        let changed = guard.recent_launches.len() != before;
        Ok((guard.recent_launches.clone(), changed))
    }

    pub fn port_ignore_ports(&self) -> Result<Vec<u16>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.port_ignore_ports.clone())
    }

    pub fn set_port_ignore_ports(&self, ports: Vec<u16>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_ignore_ports = crate::preferences::normalize_ports(ports);
        Ok(())
    }

    pub fn port_protect_ports(&self) -> Result<Vec<u16>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.port_protect_ports.clone())
    }

    pub fn set_port_protect_ports(&self, ports: Vec<u16>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_protect_ports = crate::preferences::normalize_ports(ports);
        Ok(())
    }

    pub fn port_project_path_prefixes(&self) -> Result<Vec<String>, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.port_project_path_prefixes.clone())
    }

    pub fn set_port_project_path_prefixes(&self, prefixes: Vec<String>) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_project_path_prefixes = prefixes;
        Ok(())
    }

    /// `session_list_id` = `Session.id`（列表稳定 id）。
    pub fn delete_session(&self, session_list_id: &str) -> Result<ScanResponse, String> {
        self.delete_session_inner(session_list_id)?;
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(self.response_from_guard(&guard, false))
    }

    /// 批量删除：循环与单条同一全路径；partial success；上限 50。
    pub fn delete_sessions(
        &self,
        session_list_ids: &[String],
    ) -> Result<BulkDeleteResult, String> {
        // 保序去重，避免同一 id 成功后再失败进 failures。
        let mut seen = std::collections::HashSet::new();
        let unique_ids: Vec<&String> = session_list_ids
            .iter()
            .filter(|id| !id.is_empty() && seen.insert(*id))
            .collect();
        if unique_ids.is_empty() {
            return Err("请至少选择一条 session".to_string());
        }
        if unique_ids.len() > BULK_DELETE_LIMIT {
            return Err(format!(
                "单次最多删除 {} 条，当前 {}",
                BULK_DELETE_LIMIT,
                unique_ids.len()
            ));
        }

        let mut deleted_ids = Vec::new();
        let mut failures = Vec::new();
        for id in unique_ids {
            match self.delete_session_inner(id) {
                Ok(()) => deleted_ids.push(id.clone()),
                Err(message) => failures.push(BulkDeleteFailure {
                    session_list_id: id.clone(),
                    message,
                }),
            }
        }

        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        let response = self.response_from_guard(&guard, false);
        Ok(BulkDeleteResult {
            deleted_ids,
            failures,
            sessions: response.sessions,
            scan_errors: response.scan_errors,
            from_cache: response.from_cache,
            scan_duration_ms: response.scan_duration_ms,
        })
    }

    /// 单条删除核心：OpenCode 行 / 文件目录 + 缓存更新。bulk 与 command 共用。
    fn delete_session_inner(&self, session_list_id: &str) -> Result<(), String> {
        let session = self.find_session(session_list_id)?;
        // OpenCode 会话在 SQLite 行里，不删 db 文件；其余 CLI 走文件/目录删除。
        // 缓存窗（ops_ready=false）下 delete_target 为 None → 明确失败「请刷新」。
        match session.cli_type {
            crate::models::CliType::OpenCode => {
                crate::scanner::opencode::delete_session_by_id(&session.session_id)?;
            }
            _ => {
                delete_session_target(session.delete_target.as_ref())
                    .map_err(|err| err.message())?;
            }
        }

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.sessions.retain(|item| item.id != session_list_id);

        // 同步磁盘 snapshot，避免冷启动把已删 session 带回。
        if let Some(path) = guard.scan_cache_path.clone() {
            let errors: Vec<CliScanError> = guard
                .scan_errors
                .iter()
                .map(|(cli_type, message)| CliScanError {
                    cli_type: *cli_type,
                    message: message.clone(),
                })
                .collect();
            let total_ms = guard.last_scan_duration_ms.unwrap_or(0);
            let snapshot = snapshot_from_sessions(&guard.sessions, &errors, total_ms);
            if let Err(err) = save_scan_cache(&path, &snapshot) {
                eprintln!("scan-cache write after delete failed: {err}");
            }
        }

        Ok(())
    }

    pub fn list_available_terminals(&self) -> Vec<TerminalType> {
        launchers()
            .iter()
            .filter(|launcher| launcher.is_available())
            .map(|launcher| launcher.terminal_type())
            .collect()
    }
}
