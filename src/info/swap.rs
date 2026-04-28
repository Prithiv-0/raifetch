use super::InfoModule;
use super::procfs;
use crate::theme::Theme;

pub struct SwapModule { theme: Theme }
impl SwapModule { pub fn new(theme: Theme) -> Self { Self { theme } } }

impl InfoModule for SwapModule {
    fn key(&self) -> &'static str { "Swap" }
    fn value(&self) -> anyhow::Result<String> {
        let m     = procfs::read_meminfo()?;
        let total = m.swap_total;
        let used  = m.swap_used();
        if total == 0 { return Ok("None".to_string()); }
        let pct = (used as f64 / total as f64) * 100.0;
        fn mib(b: u64) -> f64 { b as f64 / 1_048_576.0 }
        let bar = self.theme.bar(pct);
        Ok(format!("{bar} {:.0} MiB / {:.0} MiB ({:.0}%)", mib(used), mib(total), pct))
    }
}
