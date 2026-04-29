use super::{run_command_stdout, InfoModule};
use std::process::Command;
use std::time::Duration;

pub struct WmModule {
    is_de: bool,
    timeout: Duration,
}
impl WmModule {
    pub fn de(timeout: Duration) -> Self {
        Self {
            is_de: true,
            timeout,
        }
    }
    pub fn wm(timeout: Duration) -> Self {
        Self {
            is_de: false,
            timeout,
        }
    }
}

impl InfoModule for WmModule {
    fn key(&self) -> &'static str {
        if self.is_de {
            "DE"
        } else {
            "WM"
        }
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(if self.is_de {
            detect_de()
        } else {
            detect_wm(self.timeout)
        })
    }
}

#[cfg(target_os = "linux")]
pub fn detect_de() -> String {
    for var in ["XDG_CURRENT_DESKTOP", "DESKTOP_SESSION"] {
        if let Ok(v) = std::env::var(var) {
            if !v.is_empty() {
                return v;
            }
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
pub fn detect_wm(timeout: Duration) -> String {
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        for (var, name) in [
            ("HYPRLAND_INSTANCE_SIGNATURE", "Hyprland"),
            ("SWAYSOCK", "Sway"),
            ("KDE_FULL_SESSION", "KWin"),
        ] {
            if std::env::var(var).is_ok() {
                return name.to_string();
            }
        }
        return "Wayland compositor".to_string();
    }
    // X11
    if let Some(s) = run_command_stdout(
        {
            let mut cmd = Command::new("wmctrl");
            cmd.arg("-m");
            cmd
        },
        timeout,
    ) {
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("Name:") {
                let n = rest.trim().to_string();
                if !n.is_empty() && n != "N/A" {
                    return n;
                }
            }
        }
    }
    "unknown".to_string()
}

#[cfg(target_os = "macos")]
pub fn detect_wm(_timeout: Duration) -> String {
    "Quartz Compositor".to_string()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn detect_wm(_timeout: Duration) -> String {
    "unknown".to_string()
}
