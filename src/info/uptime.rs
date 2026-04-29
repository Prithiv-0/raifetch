#[cfg(target_os = "macos")]
use super::run_command_stdout;
use super::InfoModule;
use std::time::Duration;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct UptimeModule {
    timeout: Duration,
}
impl UptimeModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for UptimeModule {
    fn key(&self) -> &'static str {
        "Uptime"
    }
    fn value(&self) -> anyhow::Result<String> {
        let secs = get_uptime_secs(self.timeout);
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins = (secs % 3600) / 60;

        let mut parts = Vec::new();
        if days > 0 {
            parts.push(format!("{days}d"));
        }
        if hours > 0 {
            parts.push(format!("{hours}h"));
        }
        parts.push(format!("{mins}m"));
        Ok(parts.join(" "))
    }
}

#[cfg(target_os = "linux")]
fn get_uptime_secs(_timeout: Duration) -> u64 {
    procfs::read_uptime_secs()
}

#[cfg(target_os = "macos")]
fn get_uptime_secs(timeout: Duration) -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    // sysctl kern.boottime returns a struct timeval: { sec = 1629837, usec = ... }
    if let Some(text) = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "kern.boottime"]);
            cmd
        },
        timeout,
    ) {
        if let Some(sec_str) = text.split("sec = ").nth(1) {
            if let Some(sec_val) = sec_str.split(',').next() {
                if let Ok(boot_time) = sec_val.parse::<u64>() {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    return now.saturating_sub(boot_time);
                }
            }
        }
    }
    0
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_uptime_secs(_timeout: Duration) -> u64 {
    0
}
