use super::InfoModule;

pub struct DisplayModule;
impl DisplayModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for DisplayModule {
    fn key(&self) -> &'static str {
        "Display"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(read_displays().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn read_displays() -> Option<String> {
    let mut parts = Vec::new();
    let dir = std::fs::read_dir("/sys/class/drm").ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy().to_string();
        if !name.starts_with("card") || !name.contains('-') {
            continue;
        }
        let base = entry.path();
        let status = match std::fs::read_to_string(base.join("status")) {
            Ok(status) => status,
            Err(_) => continue,
        };
        if status.trim() != "connected" {
            continue;
        }
        let mode = std::fs::read_to_string(base.join("modes"))
            .ok()
            .and_then(|s| s.lines().next().map(|v| v.to_string()));
        if let Some(mode) = mode {
            parts.push(format!("{name} {mode}"));
        } else {
            parts.push(name);
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

#[cfg(not(target_os = "linux"))]
fn read_displays() -> Option<String> {
    None
}
