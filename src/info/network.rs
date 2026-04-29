#[cfg(target_os = "macos")]
use super::run_command_stdout;
use super::InfoModule;
use std::time::Duration;

pub struct NetworkModule {
    timeout: Duration,
}
impl NetworkModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for NetworkModule {
    fn key(&self) -> &'static str {
        "Network"
    }
    fn value(&self) -> anyhow::Result<String> {
        let ifaces = get_ifaces(self.timeout);
        #[cfg(target_os = "linux")]
        let ips = iface_ipv4_map();
        let mut parts: Vec<String> = Vec::new();

        for iface in &ifaces {
            if iface == "lo"
                || iface.starts_with("lo")
                || iface.starts_with("awdl")
                || iface.starts_with("llw")
            {
                continue;
            }
            #[cfg(target_os = "linux")]
            let ip = ips.get(iface).cloned();
            #[cfg(not(target_os = "linux"))]
            let ip = get_ip_for_iface(iface, self.timeout);

            if let Some(ip) = ip {
                parts.push(format!("{iface} ({ip})"));
            }
            if parts.len() >= 2 {
                break;
            }
        }

        if parts.is_empty() {
            Ok("No network".to_string())
        } else {
            Ok(parts.join(", "))
        }
    }
}

#[cfg(target_os = "linux")]
/// Prefer default-route interfaces, then append other active interfaces.
fn get_ifaces(_timeout: Duration) -> Vec<String> {
    let mut out = default_route_ifaces();
    let Ok(text) = std::fs::read_to_string("/proc/net/dev") else {
        return out;
    };
    for iface in text
        .lines()
        .skip(2)
        .filter_map(|line| line.split(':').next().map(|s| s.trim().to_string()))
    {
        if !out.iter().any(|seen| seen == &iface) {
            out.push(iface);
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn default_route_ifaces() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string("/proc/net/route") else {
        return Vec::new();
    };
    let mut routes = Vec::new();
    for line in text.lines().skip(1) {
        let cols: Vec<&str> = line.split_whitespace().collect();
        if cols.len() < 7 || cols[1] != "00000000" {
            continue;
        }
        let metric = cols[6].parse::<u32>().unwrap_or(u32::MAX);
        routes.push((metric, cols[0].to_string()));
    }
    routes.sort_by_key(|(metric, _)| *metric);

    let mut out = Vec::new();
    for (_, iface) in routes {
        if !out.iter().any(|seen| seen == &iface) {
            out.push(iface);
        }
    }
    out
}

#[cfg(target_os = "macos")]
fn get_ifaces(timeout: Duration) -> Vec<String> {
    if let Some(text) =
        run_command_stdout(std::process::Command::new("ifconfig").args(["-l"]), timeout)
    {
        return text.split_whitespace().map(|s| s.to_string()).collect();
    }
    vec![]
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_ifaces(_timeout: Duration) -> Vec<String> {
    vec![]
}

#[cfg(target_os = "linux")]
fn iface_ipv4_map() -> std::collections::HashMap<String, String> {
    use std::ffi::CStr;
    use std::net::Ipv4Addr;

    let mut map = std::collections::HashMap::new();
    unsafe {
        let mut addrs: *mut libc::ifaddrs = std::ptr::null_mut();
        if libc::getifaddrs(&mut addrs) != 0 {
            return map;
        }

        let mut cur = addrs;
        while !cur.is_null() {
            let addr = &*cur;
            if !addr.ifa_name.is_null()
                && !addr.ifa_addr.is_null()
                && (*addr.ifa_addr).sa_family as i32 == libc::AF_INET
            {
                let name = CStr::from_ptr(addr.ifa_name).to_string_lossy().to_string();
                let sin = addr.ifa_addr as *const libc::sockaddr_in;
                let ip = Ipv4Addr::from(u32::from_be((*sin).sin_addr.s_addr)).to_string();
                map.entry(name).or_insert(ip);
            }
            cur = addr.ifa_next;
        }
        libc::freeifaddrs(addrs);
    }
    map
}

#[cfg(target_os = "macos")]
fn get_ip_for_iface(iface: &str, timeout: Duration) -> Option<String> {
    let text = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("ifconfig");
            cmd.args([iface]);
            cmd
        },
        timeout,
    )?;
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("inet ") {
            return Some(rest.split_whitespace().next()?.trim().to_string());
        }
    }
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_ip_for_iface(_iface: &str, _timeout: Duration) -> Option<String> {
    None
}
