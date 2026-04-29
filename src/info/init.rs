use super::InfoModule;

pub struct InitModule;
impl InitModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for InitModule {
    fn key(&self) -> &'static str {
        "Init"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(detect_init().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn detect_init() -> Option<String> {
    std::fs::read_to_string("/proc/1/comm")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::fs::read_to_string("/proc/1/cmdline")
                .ok()
                .and_then(|s| s.split('\0').next().map(|v| v.to_string()))
                .filter(|s| !s.is_empty())
        })
}

#[cfg(target_os = "macos")]
fn detect_init() -> Option<String> {
    Some("launchd".to_string())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn detect_init() -> Option<String> {
    None
}
