use super::InfoModule;

pub struct EntropyModule;
impl EntropyModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for EntropyModule {
    fn key(&self) -> &'static str {
        "Entropy"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_entropy().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn read_entropy() -> Option<String> {
    let avail = std::fs::read_to_string("/proc/sys/kernel/random/entropy_avail").ok()?;
    let avail = avail.trim();
    let pool = std::fs::read_to_string("/proc/sys/kernel/random/poolsize").ok();
    if let Some(pool) = pool {
        let pool = pool.trim();
        if !pool.is_empty() {
            return Some(format!("{avail} / {pool}"));
        }
    }
    Some(avail.to_string())
}

#[cfg(not(target_os = "linux"))]
fn read_entropy() -> Option<String> {
    None
}
