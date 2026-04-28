use std::sync::Arc;
use sysinfo::System;
use super::InfoModule;
use crate::theme::Theme;

pub struct MemoryModule { sys: Arc<System>, theme: Theme }
impl MemoryModule {
    pub fn new(sys: Arc<System>, theme: Theme) -> Self { Self { sys, theme } }
}

impl InfoModule for MemoryModule {
    fn key(&self) -> &'static str { "Memory" }
    fn value(&self) -> anyhow::Result<String> {
        let total = self.sys.total_memory();
        let used  = self.sys.used_memory();
        let pct   = (used as f64 / total as f64) * 100.0;
        fn mib(b: u64) -> f64 { b as f64 / 1_048_576.0 }
        let bar = self.theme.bar(pct);
        Ok(format!("{bar} {:.0} MiB / {:.0} MiB ({:.0}%)", mib(used), mib(total), pct))
    }
}
