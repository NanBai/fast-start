mod parser;
mod process;

use crate::models::{PortProtocol, PortScanResponse, PortUsage};
use chrono::Utc;
use parser::parse_lsof_ports;
use std::collections::{HashMap, HashSet};

const SCAN_COMMAND_DESCRIPTION: &str = "/usr/sbin/lsof -nP -F pcunPTn -iTCP -sTCP:LISTEN -iUDP";

pub fn scan_ports() -> Result<PortScanResponse, String> {
    let (output, raw_line_count) = process::scan_ports_output()?;
    let mut ports = parse_lsof_ports(&output);
    enrich_ports(&mut ports);

    Ok(PortScanResponse {
        ports,
        raw_line_count,
        command_description: SCAN_COMMAND_DESCRIPTION.to_string(),
        scanned_at: Utc::now(),
    })
}

pub fn terminate_cached_ports(
    cached_ports: &[PortUsage],
    current_ports: &[PortUsage],
    port_ids: &[String],
) -> Result<(), String> {
    let pids = pids_for_port_ids(cached_ports, current_ports, port_ids)?;
    process::terminate_pids(&pids)
}

fn enrich_ports(ports: &mut [PortUsage]) {
    let pids = ports.iter().map(|port| port.pid).collect::<HashSet<_>>();
    let executable_paths = process::resolve_executable_paths(&pids);
    let working_dirs = process::resolve_working_directories(&pids);
    let parent_commands = process::resolve_parent_commands(&pids);
    let user_tokens = current_user_tokens();
    let home_dir = dirs::home_dir().map(|path| path.to_string_lossy().to_string());

    for port in ports {
        port.executable_path = executable_paths.get(&port.pid).cloned().unwrap_or_default();
        port.working_directory = working_dirs.get(&port.pid).cloned().unwrap_or_default();
        port.parent_command = parent_commands.get(&port.pid).cloned().unwrap_or_default();
        port.user_owned = is_owned_by_current_user(&port.user, &user_tokens);
        port.is_project_service = is_project_service(port, home_dir.as_deref());
    }
}

fn pids_for_port_ids(
    cached_ports: &[PortUsage],
    current_ports: &[PortUsage],
    port_ids: &[String],
) -> Result<Vec<i32>, String> {
    if port_ids.is_empty() {
        return Err("请选择要关闭的端口服务".to_string());
    }

    let cached_by_id = cached_ports
        .iter()
        .map(|port| (port.id.as_str(), port))
        .collect::<HashMap<_, _>>();
    let current_by_id = current_ports
        .iter()
        .map(|port| (port.id.as_str(), port))
        .collect::<HashMap<_, _>>();
    let mut pids = Vec::new();

    for port_id in port_ids {
        let cached = cached_by_id
            .get(port_id.as_str())
            .ok_or_else(|| "端口记录已过期，请刷新后重试".to_string())?;
        let current = current_by_id
            .get(port_id.as_str())
            .ok_or_else(|| "端口记录已过期，请刷新后重试".to_string())?;
        if !same_kill_target(cached, current) {
            return Err("端口记录已变化，请刷新后重试".to_string());
        }
        if !current.user_owned {
            return Err("只允许关闭当前用户的端口进程".to_string());
        }
        if current.pid <= 0 {
            return Err("端口进程信息无效，请刷新后重试".to_string());
        }
        if !pids.contains(&current.pid) {
            pids.push(current.pid);
        }
    }

    pids.sort_unstable();
    Ok(pids)
}

fn same_kill_target(cached: &PortUsage, current: &PortUsage) -> bool {
    cached.id == current.id
        && cached.command == current.command
        && cached.pid == current.pid
        && cached.user == current.user
        && cached.protocol == current.protocol
        && cached.address == current.address
        && cached.port == current.port
        && cached.state == current.state
        && cached.executable_path == current.executable_path
}

fn is_project_service(port: &PortUsage, home_dir: Option<&str>) -> bool {
    port.protocol == PortProtocol::Tcp
        && port.state == "LISTEN"
        && is_local_address(&port.address)
        && port.user_owned
        && is_user_process_path(&port.executable_path, &port.working_directory, home_dir)
}

fn is_local_address(address: &str) -> bool {
    address == "*"
        || address == "0.0.0.0"
        || address == "::"
        || address == "[::]"
        || address == "[::1]"
        || address == "::1"
        || address.starts_with("127.")
        || address.to_ascii_lowercase().contains("localhost")
}

fn is_user_process_path(
    executable_path: &str,
    working_directory: &str,
    home_dir: Option<&str>,
) -> bool {
    if executable_path.is_empty() {
        return !working_directory.is_empty()
            && !is_app_or_system_path(working_directory, home_dir);
    }

    !is_app_or_system_path(executable_path, home_dir)
}

fn is_app_or_system_path(path: &str, home_dir: Option<&str>) -> bool {
    if path.starts_with("/Applications/") {
        return true;
    }
    if let Some(home_dir) = home_dir {
        let user_app_dir = format!("{home_dir}/Applications/");
        if path.starts_with(&user_app_dir) {
            return true;
        }
    }

    [
        "/usr/sbin/",
        "/usr/libexec/",
        "/System/Library/",
        "/sbin/",
        "/Library/Apple/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
}

fn is_owned_by_current_user(user: &str, tokens: &HashSet<String>) -> bool {
    tokens.contains(user)
}

fn current_user_tokens() -> HashSet<String> {
    let mut tokens = HashSet::new();
    tokens.insert(unsafe { libc::getuid() }.to_string());
    for key in ["USER", "LOGNAME"] {
        if let Ok(value) = std::env::var(key) {
            if !value.is_empty() {
                tokens.insert(value);
            }
        }
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::{is_project_service, pids_for_port_ids};
    use crate::models::{PortProtocol, PortUsage};

    fn port(id: &str, pid: i32, user_owned: bool) -> PortUsage {
        PortUsage {
            id: id.to_string(),
            command: "node".to_string(),
            pid,
            user: "501".to_string(),
            protocol: PortProtocol::Tcp,
            address: "127.0.0.1".to_string(),
            port: 3000,
            state: "LISTEN".to_string(),
            executable_path: "/usr/local/bin/node".to_string(),
            working_directory: "/tmp/app".to_string(),
            parent_command: "npm".to_string(),
            is_project_service: true,
            user_owned,
        }
    }

    #[test]
    fn pids_for_port_ids_dedupes_cached_current_user_ports() {
        let ports = vec![port("a", 42, true), port("b", 42, true), port("c", 7, true)];

        let pids = pids_for_port_ids(&ports, &ports, &["b".to_string(), "a".to_string()]).unwrap();

        assert_eq!(pids, vec![42]);
    }

    #[test]
    fn pids_for_port_ids_rejects_unknown_or_other_user_ports() {
        let ports = vec![port("a", 42, false)];

        assert!(pids_for_port_ids(&ports, &ports, &["missing".to_string()]).is_err());
        assert!(pids_for_port_ids(&ports, &ports, &["a".to_string()]).is_err());
    }

    #[test]
    fn pids_for_port_ids_rejects_stale_or_changed_records() {
        let cached = vec![port("a", 42, true)];
        let mut changed = port("a", 42, true);
        changed.command = "python".to_string();

        assert!(pids_for_port_ids(&cached, &[], &["a".to_string()]).is_err());
        assert!(pids_for_port_ids(&cached, &[changed], &["a".to_string()]).is_err());
    }

    #[test]
    fn project_service_requires_local_tcp_listen_user_executable() {
        let mut item = port("a", 42, true);

        assert!(is_project_service(&item, Some("/Users/me")));
        item.executable_path = "/Applications/Chrome.app/Contents/MacOS/Chrome".to_string();
        assert!(!is_project_service(&item, Some("/Users/me")));
        item.executable_path.clear();
        item.working_directory.clear();
        assert!(!is_project_service(&item, Some("/Users/me")));
        item.working_directory = "/tmp/app".to_string();
        assert!(is_project_service(&item, Some("/Users/me")));
    }
}
