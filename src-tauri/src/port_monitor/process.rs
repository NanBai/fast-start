use std::collections::{HashMap, HashSet};
use std::process::Command;

const LSOF: &str = "/usr/sbin/lsof";
const KILL: &str = "/bin/kill";
#[cfg(not(target_os = "macos"))]
const PS: &str = "/bin/ps";

pub fn scan_ports_output() -> Result<(String, usize), String> {
    let output = Command::new(LSOF)
        .args(["-nP", "-F", "pcunPTn", "-iTCP", "-sTCP:LISTEN", "-iUDP"])
        .output()
        .map_err(|err| format!("端口扫描命令启动失败：{err}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if !output.status.success() && stdout.trim().is_empty() {
        return Err(if stderr.is_empty() {
            "端口扫描命令执行失败".to_string()
        } else {
            stderr
        });
    }

    let lines = stdout.lines().count();
    Ok((stdout, lines))
}

pub fn resolve_working_directories(pids: &HashSet<i32>) -> HashMap<i32, String> {
    let mut result = HashMap::new();
    let sorted = sorted_positive_pids(pids);

    for chunk in sorted.chunks(100) {
        let pid_arg = chunk
            .iter()
            .map(i32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let Ok(output) = Command::new(LSOF)
            .args(["-a", "-d", "cwd", "-p", &pid_arg, "-F", "pn"])
            .output()
        else {
            continue;
        };
        if !output.status.success() {
            continue;
        }
        parse_cwd_output(&String::from_utf8_lossy(&output.stdout), &mut result);
    }

    result
}

pub fn resolve_executable_paths(pids: &HashSet<i32>) -> HashMap<i32, String> {
    sorted_positive_pids(pids)
        .into_iter()
        .filter_map(|pid| resolve_executable_path(pid).map(|value| (pid, value)))
        .collect()
}

#[cfg(target_os = "macos")]
pub fn resolve_parent_commands(pids: &HashSet<i32>) -> HashMap<i32, String> {
    let mut pid_to_parent = HashMap::new();
    let mut parent_pids = HashSet::new();

    for pid in sorted_positive_pids(pids) {
        let Some(info) = resolve_bsd_info(pid) else {
            continue;
        };
        let parent_pid = info.pbi_ppid as i32;
        if parent_pid <= 0 || parent_pid == pid {
            continue;
        }
        pid_to_parent.insert(pid, parent_pid);
        parent_pids.insert(parent_pid);
    }

    let parent_commands = sorted_positive_pids(&parent_pids)
        .into_iter()
        .filter_map(|pid| {
            resolve_bsd_info(pid)
                .and_then(|info| string_from_c_chars(&info.pbi_comm))
                .map(|command| (pid, command))
        })
        .collect::<HashMap<_, _>>();

    pid_to_parent
        .into_iter()
        .filter_map(|(pid, ppid)| {
            parent_commands
                .get(&ppid)
                .map(|command| (pid, command.clone()))
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
pub fn resolve_parent_commands(pids: &HashSet<i32>) -> HashMap<i32, String> {
    let pid_to_parent = parent_pid_map(pids);
    let parents = pid_to_parent.values().copied().collect::<HashSet<_>>();
    let parent_commands = resolve_executable_paths(&parents);

    pid_to_parent
        .into_iter()
        .filter_map(|(pid, ppid)| {
            parent_commands
                .get(&ppid)
                .map(|command| (pid, command_name(command)))
        })
        .collect()
}

pub fn terminate_pids(pids: &[i32]) -> Result<(), String> {
    if pids.is_empty() {
        return Ok(());
    }
    let args = pids.iter().map(i32::to_string).collect::<Vec<_>>();
    let output = Command::new(KILL)
        .arg("-TERM")
        .args(&args)
        .output()
        .map_err(|err| format!("关闭服务命令启动失败：{err}"))?;

    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        "关闭端口服务失败".to_string()
    } else {
        stderr
    })
}

fn parse_cwd_output(output: &str, result: &mut HashMap<i32, String>) {
    let mut current_pid = None;
    for line in output.lines().filter(|line| !line.is_empty()) {
        let Some(marker) = line.chars().next() else {
            continue;
        };
        let value = &line[marker.len_utf8()..];
        match marker {
            'p' => current_pid = value.parse::<i32>().ok(),
            'n' => {
                if let Some(pid) = current_pid.take() {
                    result.insert(pid, value.to_string());
                }
            }
            _ => {}
        }
    }
}

#[cfg(target_os = "macos")]
fn resolve_executable_path(pid: i32) -> Option<String> {
    let mut buffer = vec![0 as libc::c_char; libc::PROC_PIDPATHINFO_MAXSIZE as usize];
    let len = unsafe {
        libc::proc_pidpath(
            pid,
            buffer.as_mut_ptr().cast::<libc::c_void>(),
            buffer.len() as u32,
        )
    };
    if len <= 0 {
        return None;
    }

    string_from_c_chars(&buffer[..len as usize])
}

#[cfg(not(target_os = "macos"))]
fn resolve_executable_path(pid: i32) -> Option<String> {
    run_ps_value(pid, "comm=")
}

#[cfg(target_os = "macos")]
fn resolve_bsd_info(pid: i32) -> Option<libc::proc_bsdinfo> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    let ret = unsafe {
        libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast::<libc::c_void>(),
            size,
        )
    };
    if ret < size {
        return None;
    }
    Some(unsafe { info.assume_init() })
}

#[cfg(target_os = "macos")]
fn string_from_c_chars(chars: &[libc::c_char]) -> Option<String> {
    let bytes = chars
        .iter()
        .copied()
        .take_while(|value| *value != 0)
        .map(|value| value as u8)
        .collect::<Vec<_>>();
    if bytes.is_empty() {
        return None;
    }
    Some(String::from_utf8_lossy(&bytes).to_string())
}

#[cfg(not(target_os = "macos"))]
fn parent_pid_map(pids: &HashSet<i32>) -> HashMap<i32, i32> {
    sorted_positive_pids(pids)
        .into_iter()
        .filter_map(|pid| {
            run_ps_value(pid, "ppid=")
                .and_then(|value| value.trim().parse::<i32>().ok())
                .filter(|ppid| *ppid > 0 && *ppid != pid)
                .map(|ppid| (pid, ppid))
        })
        .collect()
}

#[cfg(not(target_os = "macos"))]
fn run_ps_value(pid: i32, output_format: &str) -> Option<String> {
    let output = Command::new(PS)
        .args(["-p", &pid.to_string(), "-o", output_format])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn sorted_positive_pids(pids: &HashSet<i32>) -> Vec<i32> {
    let mut sorted = pids
        .iter()
        .copied()
        .filter(|pid| *pid > 0)
        .collect::<Vec<_>>();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

#[cfg(not(target_os = "macos"))]
fn command_name(command: &str) -> String {
    command
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
        .unwrap_or(command)
        .to_string()
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::{resolve_executable_paths, string_from_c_chars};
    use std::collections::HashSet;

    #[test]
    fn string_from_c_chars_stops_at_nul() {
        let value = [
            b'n' as libc::c_char,
            b'o' as libc::c_char,
            b'd' as libc::c_char,
            b'e' as libc::c_char,
            0,
            b'x' as libc::c_char,
        ];

        assert_eq!(string_from_c_chars(&value).as_deref(), Some("node"));
    }

    #[test]
    fn resolve_executable_paths_returns_full_path_for_current_process() {
        let pids = HashSet::from([std::process::id() as i32]);

        let paths = resolve_executable_paths(&pids);
        let path = paths
            .get(&(std::process::id() as i32))
            .expect("current process executable path should resolve");

        assert!(path.starts_with('/'), "expected absolute path, got {path}");
    }
}
