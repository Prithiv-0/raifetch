use super::InfoModule;

pub struct NetworkModule;
impl NetworkModule { pub fn new() -> Self { Self } }

impl InfoModule for NetworkModule {
    fn key(&self) -> &'static str { "Network" }
    fn value(&self) -> anyhow::Result<String> {
        // Read interfaces from /proc/net/dev (always present on Linux)
        let ifaces = read_ifaces();
        let mut parts: Vec<String> = Vec::new();

        for iface in &ifaces {
            if iface == "lo" { continue; }
            if let Some(ip) = get_ip_for_iface(iface) {
                parts.push(format!("{iface} ({ip})"));
            }
            if parts.len() >= 2 { break; }
        }

        if parts.is_empty() { Ok("No network".to_string()) }
        else { Ok(parts.join(", ")) }
    }
}

/// List non-loopback interface names from /proc/net/dev.
fn read_ifaces() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string("/proc/net/dev") else { return vec![] };
    text.lines().skip(2) // skip header rows
        .filter_map(|line| line.split(':').next().map(|s| s.trim().to_string()))
        .collect()
}

fn get_ip_for_iface(iface: &str) -> Option<String> {
    let out = std::process::Command::new("ip")
        .args(["-4", "addr", "show", iface])
        .output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if let Some(rest) = line.trim().strip_prefix("inet ") {
            return Some(rest.split('/').next()?.trim().to_string());
        }
    }
    None
}
