use super::InfoModule;
use super::procfs;

pub struct UptimeModule;
impl UptimeModule { pub fn new() -> Self { Self } }

impl InfoModule for UptimeModule {
    fn key(&self) -> &'static str { "Uptime" }
    fn value(&self) -> anyhow::Result<String> {
        let secs  = procfs::read_uptime_secs();
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
