use super::InfoModule;

pub struct MoboModule;
impl MoboModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for MoboModule {
    fn key(&self) -> &'static str {
        "Mobo"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_mobo().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn read_mobo() -> Option<String> {
    let vendor = std::fs::read_to_string("/sys/class/dmi/id/board_vendor")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let name = std::fs::read_to_string("/sys/class/dmi/id/board_name")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let version = std::fs::read_to_string("/sys/class/dmi/id/board_version")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    match (vendor, name, version) {
        (Some(v), Some(n), Some(ver)) => Some(format!("{v} {n} ({ver})")),
        (Some(v), Some(n), None) => Some(format!("{v} {n}")),
        (None, Some(n), Some(ver)) => Some(format!("{n} ({ver})")),
        (None, Some(n), None) => Some(n),
        (Some(v), None, _) => Some(v),
        _ => None,
    }
}

#[cfg(not(target_os = "linux"))]
fn read_mobo() -> Option<String> {
    None
}
