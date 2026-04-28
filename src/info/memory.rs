use super::InfoModule;
use crate::theme::Theme;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct MemoryModule { theme: Theme }
impl MemoryModule { pub fn new(theme: Theme) -> Self { Self { theme } } }

impl InfoModule for MemoryModule {
    fn key(&self) -> &'static str { "Memory" }
    fn value(&self) -> anyhow::Result<String> {
        let (used, total) = read_mem()?;
        if total == 0 { return Ok("Unknown".to_string()); }
        let pct   = (used as f64 / total as f64) * 100.0;
        fn mib(b: u64) -> f64 { b as f64 / 1_048_576.0 }
        let bar   = self.theme.bar(pct);
        Ok(format!("{bar} {:.0} MiB / {:.0} MiB ({:.0}%)", mib(used), mib(total), pct))
    }
}

#[cfg(target_os = "linux")]
fn read_mem() -> anyhow::Result<(u64, u64)> {
    let m = procfs::read_meminfo()?;
    Ok((m.mem_used(), m.mem_total))
}

#[cfg(target_os = "macos")]
fn read_mem() -> anyhow::Result<(u64, u64)> {
    let total = std::process::Command::new("sysctl").args(["-n", "hw.memsize"]).output()
        .ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u64>().ok()).unwrap_or(0);
    
    let out = std::process::Command::new("vm_stat").output().map_err(|_| anyhow::anyhow!("vm_stat failed"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut pages_active = 0u64;
    let mut pages_wired = 0u64;
    for line in text.lines() {
        if line.starts_with("Pages active:") {
            pages_active = line.split(':').nth(1).unwrap_or("").trim().trim_end_matches('.').parse().unwrap_or(0);
        } else if line.starts_with("Pages wired down:") {
            pages_wired = line.split(':').nth(1).unwrap_or("").trim().trim_end_matches('.').parse().unwrap_or(0);
        }
    }
    // Assume 4096 page size
    let used = (pages_active + pages_wired) * 4096;
    Ok((used, total))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_mem() -> anyhow::Result<(u64, u64)> {
    Ok((0, 0))
}
