use sysinfo::Networks;
use super::InfoModule;

pub struct NetworkModule;
impl NetworkModule { pub fn new() -> Self { Self } }

impl InfoModule for NetworkModule {
    fn key(&self) -> &'static str { "Network" }
    fn value(&self) -> anyhow::Result<String> {
        // Try reading local IP from /proc/net/fib_trie (Linux)
        // Fallback: just show the first non-loopback interface name
        let networks = Networks::new_with_refreshed_list();
        let mut parts: Vec<String> = Vec::new();

        for (name, _data) in &networks {
            if name == "lo" { continue; }
            // Get the first IPv4 address via ip command fallback
            let ip = get_ip_for_iface(name)
                .unwrap_or_else(|| "?.?.?.?".to_string());
            parts.push(format!("{name} ({ip})"));
            if parts.len() >= 2 { break; }
        }

        if parts.is_empty() { Ok("No network".to_string()) }
        else { Ok(parts.join(", ")) }
    }
}

fn get_ip_for_iface(iface: &str) -> Option<String> {
    // Parse /proc/net/fib_trie is complex — use `ip` command for reliability
    let out = std::process::Command::new("ip")
        .args(["-4", "addr", "show", iface])
        .output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let ip = rest.split('/').next()?.trim().to_string();
            return Some(ip);
        }
    }
    None
}
