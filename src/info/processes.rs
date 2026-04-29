use super::InfoModule;

pub struct ProcessesModule;
impl ProcessesModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for ProcessesModule {
    fn key(&self) -> &'static str {
        "Processes"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(count_processes()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn count_processes() -> Option<usize> {
    let entries = std::fs::read_dir("/proc").ok()?;
    let mut count = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.chars().all(|c| c.is_ascii_digit()) {
            count += 1;
        }
    }
    Some(count)
}

#[cfg(not(target_os = "linux"))]
fn count_processes() -> Option<usize> {
    None
}
