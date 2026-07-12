use crate::models::{PortProtocol, PortUsage};
use std::collections::HashSet;

#[derive(Default)]
struct ParsedProcess {
    pid: i32,
    command: String,
    user: String,
    protocol: Option<PortProtocol>,
    state: String,
}

pub fn parse_lsof_ports(output: &str) -> Vec<PortUsage> {
    let mut current = ParsedProcess::default();
    let mut ports = Vec::new();
    let mut seen_ids = HashSet::new();

    for line in output.lines().filter(|line| !line.is_empty()) {
        let Some(marker) = line.chars().next() else {
            continue;
        };
        let value = &line[marker.len_utf8()..];

        match marker {
            'p' => {
                current.pid = value.parse().unwrap_or_default();
                current.protocol = None;
                current.state.clear();
            }
            'c' => current.command = value.to_string(),
            'u' => current.user = value.to_string(),
            'P' => {
                current.protocol = match value {
                    "TCP" => Some(PortProtocol::Tcp),
                    "UDP" => Some(PortProtocol::Udp),
                    _ => None,
                };
                if current.protocol == Some(PortProtocol::Udp) {
                    current.state.clear();
                }
            }
            'T' if value.starts_with("ST=") => current.state = value[3..].to_string(),
            'n' => append_endpoint(&current, value, &mut ports, &mut seen_ids),
            _ => {}
        }
    }

    ports.sort_by(|left, right| {
        left.port
            .cmp(&right.port)
            .then_with(|| left.command.cmp(&right.command))
            .then_with(|| left.pid.cmp(&right.pid))
    });
    ports
}

fn append_endpoint(
    current: &ParsedProcess,
    value: &str,
    ports: &mut Vec<PortUsage>,
    seen_ids: &mut HashSet<String>,
) {
    let Some(protocol) = current.protocol else {
        return;
    };
    let Some((address, port)) = parse_endpoint(value) else {
        return;
    };

    let state = if current.state.is_empty() && protocol == PortProtocol::Tcp {
        "LISTEN".to_string()
    } else {
        current.state.clone()
    };

    let id = port_usage_id(protocol, port, current.pid, &address);
    if !seen_ids.insert(id.clone()) {
        return;
    }

    ports.push(PortUsage {
        id,
        command: current.command.clone(),
        pid: current.pid,
        user: current.user.clone(),
        protocol,
        address,
        port,
        state,
        executable_path: String::new(),
        working_directory: String::new(),
        parent_command: String::new(),
        is_project_service: false,
        user_owned: false,
    });
}

fn parse_endpoint(text: &str) -> Option<(String, u16)> {
    let endpoint = text.split_once("->").map(|item| item.0).unwrap_or(text);
    let separator = endpoint.rfind(':')?;
    let port = endpoint[separator + 1..].parse().ok()?;
    let address = match &endpoint[..separator] {
        "" => "*".to_string(),
        value => value.to_string(),
    };
    Some((address, port))
}

pub fn port_usage_id(protocol: PortProtocol, port: u16, pid: i32, address: &str) -> String {
    let protocol = match protocol {
        PortProtocol::Tcp => "tcp",
        PortProtocol::Udp => "udp",
    };
    format!("{protocol}-{port}-{pid}-{address}")
}

#[cfg(test)]
mod tests {
    use super::parse_lsof_ports;
    use crate::models::PortProtocol;

    #[test]
    fn parses_tcp_and_udp_lsof_field_output() {
        let output = [
            "p123",
            "cnode",
            "u501",
            "PTCP",
            "TST=LISTEN",
            "n127.0.0.1:3000",
            "p456",
            "cdnsmasq",
            "u0",
            "PUDP",
            "n*:53",
        ]
        .join("\n");

        let ports = parse_lsof_ports(&output);

        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].protocol, PortProtocol::Udp);
        assert_eq!(ports[0].port, 53);
        assert_eq!(ports[1].protocol, PortProtocol::Tcp);
        assert_eq!(ports[1].state, "LISTEN");
    }

    #[test]
    fn dedupes_repeated_fd_records_for_same_endpoint() {
        let output = [
            "p123",
            "cnode",
            "u501",
            "f7",
            "PTCP",
            "n*:3000",
            "TST=LISTEN",
            "f9",
            "PTCP",
            "n*:3000",
            "TST=LISTEN",
        ]
        .join("\n");

        let ports = parse_lsof_ports(&output);

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].id, "tcp-3000-123-*");
    }
}
