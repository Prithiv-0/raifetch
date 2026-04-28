use std::sync::Arc;
use sysinfo::System;
use super::InfoModule;

pub struct UptimeModule {
    _sys: Arc<System>,
}

impl UptimeModule {
    pub fn new(sys: Arc<System>) -> Self { Self { _sys: sys } }
}

impl InfoModule for UptimeModule {
    fn key(&self) -> &'static str { "Uptime" }

    fn value(&self) -> anyhow::Result<String> {
        let secs  = System::uptime();
        let days  = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let mins  = (secs % 3600)  / 60;

        let mut parts = Vec::new();
        if days  > 0 { parts.push(format!("{days}d"));  }
        if hours > 0 { parts.push(format!("{hours}h")); }
        parts.push(format!("{mins}m"));
        Ok(parts.join(" "))
    }
}
