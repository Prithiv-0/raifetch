use std::process::Command;
use super::InfoModule;

pub struct WmModule { is_de: bool }
impl WmModule {
    pub fn de() -> Self { Self { is_de: true  } }
    pub fn wm() -> Self { Self { is_de: false } }
}

impl InfoModule for WmModule {
    fn key(&self) -> &'static str { if self.is_de { "DE" } else { "WM" } }
    fn value(&self) -> anyhow::Result<String> {
        Ok(if self.is_de { detect_de() } else { detect_wm() })
    }
}

#[cfg(target_os = "linux")]
pub fn detect_de() -> String {
    for var in ["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() { return v; }
        }
    }
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
pub fn detect_de() -> String {
    "Aqua".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn detect_de() -> String {
    "unknown".to_string()
}

#[cfg(target_os = "linux")]
pub fn detect_wm() -> String {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        for (var, name) in [
            ("HYPRLAND_INSTANCE_SIGNATURE", "Hyprland"),
            ("SWAYSOCK",                    "Sway"),
            ("KDE_FULL_SESSION",            "KWin"),
        ] {
            if std::env::var(var).is_ok() { return name.to_string(); }
        }
        return "Wayland compositor".to_string();
    }
    // X11
    if let Ok(out) = Command::new("wmctrl").arg("-m").output() {
        if let Ok(s) = String::from_utf8(out.stdout) {
            for line in s.lines() {
                if let Some(rest) = line.strip_prefix("Name:") {
                    let n = rest.trim().to_string();
                    if !n.is_empty() && n != "N/A" { return n; }
                }
            }
        }
    }
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
pub fn detect_wm() -> String {
    "Quartz Compositor".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn detect_wm() -> String {
    "unknown".to_string()
}
