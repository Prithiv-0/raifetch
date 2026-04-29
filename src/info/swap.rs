#[cfg(target_os = "macos")]
use super::run_command_stdout;
use super::InfoModule;
use crate::theme::Theme;
use std::time::Duration;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct SwapModule {
    theme: Theme,
    timeout: Duration,
}
impl SwapModule {
    pub fn new(theme: Theme, timeout: Duration) -> Self {
        Self { theme, timeout }
    }
}

impl InfoModule for SwapModule {
    fn key(&self) -> &'static str {
        "Swap"
    }
    fn value(&self) -> anyhow::Result<String> {
        let (used, total) = read_swap(self.timeout)?;
        if total == 0 {
            return Ok("None".to_string());
        }
        let pct = (used as f64 / total as f64) * 100.0;
        fn mib(b: u64) -> f64 {
            b as f64 / 1_048_576.0
        }
        let bar = self.theme.bar(pct);
        Ok(format!(
            "{bar} {:.0} MiB / {:.0} MiB ({:.0}%)",
            mib(used),
            mib(total),
            pct
        ))
    }
}

#[cfg(target_os = "linux")]
fn read_swap(_timeout: Duration) -> anyhow::Result<(u64, u64)> {
    let m = procfs::read_meminfo()?;
    Ok((m.swap_used(), m.swap_total))
}

#[cfg(target_os = "macos")]
fn read_swap(timeout: Duration) -> anyhow::Result<(u64, u64)> {
    let text = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "vm.swapusage"]);
            cmd
        },
        timeout,
    )
    .unwrap_or_default();
    // Format: total = 1024.00M  used = 12.00M  free = 1012.00M
    let mut total = 0u64;
    let mut used = 0u64;

    let parse_mb = |s: &str| -> u64 {
        if let Some(num) = s.trim_end_matches('M').parse::<f64>().ok() {
            (num * 1_048_576.0) as u64
        } else {
            0
        }
    };

    for part in text.split("  ") {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("total = ") {
            total = parse_mb(rest);
        } else if let Some(rest) = part.strip_prefix("used = ") {
            used = parse_mb(rest);
        }
    }

    Ok((used, total))
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_swap(_timeout: Duration) -> anyhow::Result<(u64, u64)> {
    Ok((0, 0))
}
