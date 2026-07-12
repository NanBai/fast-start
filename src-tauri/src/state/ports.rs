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
        let response = port_monitor::scan_ports()?;
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

        let current_response = port_monitor::scan_ports()?;
        {
            let mut guard = self
                .inner
                .lock()
                .map_err(|_| "无法获取应用状态".to_string())?;
            guard.port_scan = Some(current_response.clone());
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

}
