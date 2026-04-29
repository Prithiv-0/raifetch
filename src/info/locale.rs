use super::InfoModule;

pub struct LocaleModule;
impl LocaleModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for LocaleModule {
    fn key(&self) -> &'static str {
        "Locale"
    }
    fn value(&self) -> anyhow::Result<String> {
        // Priority: $LANG → $LC_ALL → /etc/locale.conf → /etc/default/locale → "unknown"
        let lang = std::env::var("LANG")
            .or_else(|_| std::env::var("LC_ALL"))
            .or_else(|_| std::env::var("LC_MESSAGES"))
            .ok()
            .filter(|s| !s.is_empty() && s != "C" && s != "POSIX")
            .or_else(|| read_locale_conf())
            .unwrap_or_else(|| "unknown".to_string());

        // Timezone: $TZ → /etc/timezone → /etc/localtime symlink
        let tz = std::env::var("TZ")
            .ok()
            .or_else(|| {
                std::fs::read_to_string("/etc/timezone")
                    .ok()
                    .map(|s| s.trim().to_string())
            })
            .or_else(|| read_localtime_tz())
            .unwrap_or_else(|| "UTC".to_string());

        Ok(format!("{lang} | {tz}"))
    }
}

fn read_locale_conf() -> Option<String> {
    // Try /etc/locale.conf (systemd systems)
    for path in ["/etc/locale.conf", "/etc/default/locale"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(val) = line.strip_prefix("LANG=") {
                    let v = val.trim_matches('"').to_string();
                    if !v.is_empty() && v != "C" {
                        return Some(v);
                    }
                }
            }
        }
    }
    None
}

fn read_localtime_tz() -> Option<String> {
    // /etc/localtime is a symlink → ../usr/share/zoneinfo/Region/City
    let target = std::fs::read_link("/etc/localtime").ok()?;
    let s = target.to_string_lossy();
    // Find "zoneinfo/" in the path and take everything after it
    s.find("zoneinfo/").map(|i| s[i + 9..].to_string())
}
