use super::InfoModule;

pub struct OsModule;

impl OsModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for OsModule {
    fn key(&self) -> &'static str {
        "OS"
    }

    fn value(&self) -> anyhow::Result<String> {
        let distro = os_pretty_name().unwrap_or_else(|| "Unknown".to_string());
        let arch = std::env::consts::ARCH;
        Ok(format!("{distro} ({arch})"))
    }
}

pub fn os_pretty_name() -> Option<String> {
    let text = std::fs::read_to_string("/etc/os-release").ok()?;
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
            let value = value.trim_matches('"').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("NAME=") {
            let value = value.trim_matches('"').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

pub fn username() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "user".to_string())
}

pub fn hostname() -> String {
    std::fs::read_to_string("/proc/sys/kernel/hostname")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "localhost".to_string())
}
