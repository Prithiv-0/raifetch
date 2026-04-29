use super::InfoModule;

pub struct BiosModule;
impl BiosModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for BiosModule {
    fn key(&self) -> &'static str {
        "BIOS"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_bios().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn read_bios() -> Option<String> {
    let vendor = std::fs::read_to_string("/sys/class/dmi/id/bios_vendor")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let version = std::fs::read_to_string("/sys/class/dmi/id/bios_version")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let date = std::fs::read_to_string("/sys/class/dmi/id/bios_date")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let mut parts = Vec::new();
    if let Some(v) = vendor {
        parts.push(v);
    }
    if let Some(v) = version {
        parts.push(v);
    }
    if let Some(d) = date {
        parts.push(d);
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

#[cfg(not(target_os = "linux"))]
fn read_bios() -> Option<String> {
    None
}
