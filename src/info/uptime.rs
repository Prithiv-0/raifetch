use super::InfoModule;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct UptimeModule;
impl UptimeModule { pub fn new() -> Self { Self } }

impl InfoModule for UptimeModule {
    fn key(&self) -> &'static str { "Uptime" }
    fn value(&self) -> anyhow::Result<String> {
        let secs  = get_uptime_secs();
        let days  = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins  = (secs % 3600)  / 60;

        let mut parts = Vec::new();
        if days  > 0 { parts.push(format!("{days}d")); }
        if hours > 0 { parts.push(format!("{hours}h")); }
        parts.push(format!("{mins}m"));
        Ok(parts.join(" "))
    }
}

#[cfg(target_os = "linux")]
fn get_uptime_secs() -> u64 {
    procfs::read_uptime_secs()
}

#[cfg(target_os = "macos")]
fn get_uptime_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    // sysctl kern.boottime returns a struct timeval: { sec = 1629837, usec = ... }
    let out = std::process::Command::new("sysctl").args(["-n", "kern.boottime"]).output().ok();
    if let Some(out) = out {
        let text = String::from_utf8_lossy(&out.stdout);
        if let Some(sec_str) = text.split("sec = ").nth(1) {
            if let Some(sec_val) = sec_str.split(',').next() {
                if let Ok(boot_time) = sec_val.parse::<u64>() {
                    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                    return now.saturating_sub(boot_time);
                }
            }
        }
    }
    0
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_uptime_secs() -> u64 {
    0
}
