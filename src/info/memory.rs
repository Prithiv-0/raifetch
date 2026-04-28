use super::InfoModule;
use super::procfs;
use crate::theme::Theme;

pub struct MemoryModule { theme: Theme }
impl MemoryModule { pub fn new(theme: Theme) -> Self { Self { theme } } }

impl InfoModule for MemoryModule {
    fn key(&self) -> &'static str { "Memory" }
    fn value(&self) -> anyhow::Result<String> {
        let m     = procfs::read_meminfo()?;
        let used  = m.mem_used();
        let total = m.mem_total;
        let pct   = (used as f64 / total as f64) * 100.0;
        fn mib(b: u64) -> f64 { b as f64 / 1_048_576.0 }
        let bar   = self.theme.bar(pct);
        Ok(format!("{bar} {:.0} MiB / {:.0} MiB ({:.0}%)", mib(used), mib(total), pct))
    }
}
