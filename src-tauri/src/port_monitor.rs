mod parser;
mod process;

use crate::models::{PortProtocol, PortScanResponse, PortUsage};
use chrono::Utc;
use parser::parse_lsof_ports;
use std::collections::{HashMap, HashSet};

const SCAN_COMMAND_DESCRIPTION: &str = "/usr/sbin/lsof -nP -F pcunPTn -iTCP -sTCP:LISTEN -iUDP";

pub fn scan_ports() -> Result<PortScanResponse, String> {
    scan_ports_with_rules(&[], &[])
}

/// 扫描并应用偏好规则：ignore 端口剔除；path prefixes 扩大 is_project_service。
pub fn scan_ports_with_rules(
    ignore_ports: &[u16],
    project_path_prefixes: &[String],
) -> Result<PortScanResponse, String> {
    let (output, raw_line_count) = process::scan_ports_output()?;
    let mut ports = parse_lsof_ports(&output);
    enrich_ports(&mut ports);
    apply_port_rules(&mut ports, ignore_ports, project_path_prefixes);

    Ok(PortScanResponse {
        ports,
        raw_line_count,
        command_description: SCAN_COMMAND_DESCRIPTION.to_string(),
        scanned_at: Utc::now(),
    })
}

/// 规则后处理：忽略端口不展示；prefix 可把符合本地监听条件的端口标为项目服务。
pub fn apply_port_rules(
    ports: &mut Vec<PortUsage>,
    ignore_ports: &[u16],
    project_path_prefixes: &[String],
) {
    if !ignore_ports.is_empty() {
        ports.retain(|port| !ignore_ports.contains(&port.port));
    }
    if project_path_prefixes.is_empty() {
        return;
    }
    for port in ports.iter_mut() {
        if port.is_project_service {
            continue;
        }
        if expands_project_service(port, project_path_prefixes) {
            port.is_project_service = true;
        }
    }
}

fn expands_project_service(port: &PortUsage, prefixes: &[String]) -> bool {
    if port.protocol != PortProtocol::Tcp || port.state != "LISTEN" || !port.user_owned {
        return false;
    }
    if !is_local_address(&port.address) {
        return false;
    }
    let cwd = port.working_directory.as_str();
    if cwd.is_empty() {
        return false;
    }
    prefixes.iter().any(|prefix| {
        let p = prefix.trim();
        !p.is_empty() && (cwd == p || cwd.starts_with(&format!("{p}/")))
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
    use super::{apply_port_rules, expands_project_service, is_project_service, pids_for_port_ids};
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

    #[test]
    fn apply_port_rules_drops_ignored_ports() {
        let mut ports = vec![port("a", 1, true), port("b", 2, true)];
        ports[1].port = 5432;
        apply_port_rules(&mut ports, &[5432], &[]);
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].id, "a");
    }

    #[test]
    fn path_prefix_expands_project_service() {
        let mut item = port("a", 42, true);
        item.is_project_service = false;
        item.executable_path = "/Applications/Chrome.app/Contents/MacOS/Chrome".to_string();
        item.working_directory = "/Users/me/codes/app".to_string();
        // 系统路径可执行文件本非项目服务，但 cwd 命中 prefix 可扩大
        assert!(expands_project_service(
            &item,
            &["/Users/me/codes".to_string()]
        ));
        apply_port_rules(&mut vec![item.clone()], &[], &["/Users/me/codes".to_string()]);
        let mut list = vec![item];
        apply_port_rules(&mut list, &[], &["/Users/me/codes".to_string()]);
        assert!(list[0].is_project_service);
    }
}
