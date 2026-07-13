//! Port 工具页相关的 AppState 方法。
use super::AppState;
use crate::models::PortScanResponse;
use crate::port_monitor;

impl AppState {
    pub fn scan_ports(&self) -> Result<PortScanResponse, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        if let Some(response) = &guard.port_scan {
            return Ok(response.clone());
        }

        drop(guard);
        self.refresh_ports()
    }

    pub fn refresh_ports(&self) -> Result<PortScanResponse, String> {
        let (ignore, prefixes) = {
            let guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            (
                guard.port_ignore_ports.clone(),
                guard.port_project_path_prefixes.clone(),
            )
        };
        let response = port_monitor::scan_ports_with_rules(&ignore, &prefixes)?;
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_scan = Some(response.clone());
        Ok(response)
    }

    pub fn terminate_port_processes(
        &self,
        port_ids: Vec<String>,
    ) -> Result<PortScanResponse, String> {
        let cached_ports = {
            let guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            guard
                .port_scan
                .as_ref()
                .map(|response| response.ports.clone())
                .ok_or_else(|| "请先刷新端口列表".to_string())?
        };

        let current_response = {
            let (ignore, prefixes) = {
                let guard = self
                    .inner
                    .lock()
                    .map_err(|_| "无法获取应用状态".to_string())?;
                (
                    guard.port_ignore_ports.clone(),
                    guard.port_project_path_prefixes.clone(),
                )
            };
            port_monitor::scan_ports_with_rules(&ignore, &prefixes)?
        };
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            guard.port_scan = Some(current_response.clone());
        }

        // 保护端口：任一目标 port 号 ∈ protect → 整批失败，不杀进程。
        let protect = {
            let guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            guard.port_protect_ports.clone()
        };
        let hit = protected_port_hits(&current_response.ports, &port_ids, &protect);
        if !hit.is_empty() {
            let list = hit
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "操作已取消：包含受保护端口 {list}。请先从保护列表移除后再终止。"
            ));
        }

        port_monitor::terminate_cached_ports(&cached_ports, &current_response.ports, &port_ids)?;
        std::thread::sleep(std::time::Duration::from_millis(400));
        self.refresh_ports()
    }

    pub fn port_auto_refresh(&self) -> Result<bool, String> {
        let guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        Ok(guard.port_auto_refresh)
    }

    pub fn set_port_auto_refresh(&self, enabled: bool) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_auto_refresh = enabled;
        Ok(())
    }

    /// 规则变更后丢弃缓存，下次 scan/refresh 重新应用。
    pub fn invalidate_port_scan_cache(&self) -> Result<(), String> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| "无法获取应用状态".to_string())?;
        guard.port_scan = None;
        Ok(())
    }
}

/// 在当前扫描结果中，找出 `port_ids` 命中且 port 号属于 protect 的端口号（去重排序）。
pub(crate) fn protected_port_hits(
    current_ports: &[crate::models::PortUsage],
    port_ids: &[String],
    protect: &[u16],
) -> Vec<u16> {
    if protect.is_empty() || port_ids.is_empty() {
        return Vec::new();
    }
    let mut hit: Vec<u16> = current_ports
        .iter()
        .filter(|p| port_ids.iter().any(|id| id == &p.id))
        .map(|p| p.port)
        .filter(|port| protect.contains(port))
        .collect();
    hit.sort_unstable();
    hit.dedup();
    hit
}

#[cfg(test)]
mod tests {
    use super::{protected_port_hits, AppState};
    use crate::models::{LaunchMode, PortProtocol, PortUsage, TerminalType, ThemeMode};

    fn sample_port(id: &str, port: u16) -> PortUsage {
        PortUsage {
            id: id.to_string(),
            command: "node".into(),
            pid: 1234,
            user: "test".into(),
            protocol: PortProtocol::Tcp,
            address: "127.0.0.1".into(),
            port,
            state: "LISTEN".into(),
            executable_path: "/usr/bin/node".into(),
            working_directory: "/tmp".into(),
            parent_command: String::new(),
            is_project_service: true,
            user_owned: true,
        }
    }

    #[test]
    fn protected_port_hits_finds_selected_protected_ports() {
        let ports = vec![
            sample_port("p-3000", 3000),
            sample_port("p-4000", 4000),
            sample_port("p-3000-b", 3000),
        ];
        let hit = protected_port_hits(
            &ports,
            &["p-3000".into(), "p-4000".into()],
            &[3000, 5432],
        );
        assert_eq!(hit, vec![3000]);
    }

    #[test]
    fn protected_port_hits_empty_when_no_overlap() {
        let ports = vec![sample_port("p-4000", 4000)];
        let hit = protected_port_hits(&ports, &["p-4000".into()], &[3000]);
        assert!(hit.is_empty());
    }

    #[test]
    fn set_port_protect_ports_normalizes() {
        let state = AppState::new(
            TerminalType::System,
            LaunchMode::NewTab,
            ThemeMode::System,
            Vec::new(),
            Vec::new(),
            true,
        );
        state
            .set_port_protect_ports(vec![3000, 3000, 0, 8080])
            .unwrap();
        assert_eq!(state.port_protect_ports().unwrap(), vec![3000, 8080]);
    }
}
