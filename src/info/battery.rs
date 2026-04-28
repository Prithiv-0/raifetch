use super::InfoModule;
use std::fs;
use crate::theme::Theme;

pub struct BatteryModule { theme: Theme }
impl BatteryModule { pub fn new(theme: Theme) -> Self { Self { theme } } }

impl InfoModule for BatteryModule {
    fn key(&self) -> &'static str { "Battery" }
    fn value(&self) -> anyhow::Result<String> {
        if let Some((pct, status)) = get_battery_info() {
            let icon = match status.as_str() {
                "Charging" => "⚡", "Full" => "🔌", _ => "🔋",
            };
            let bar = self.theme.bar(pct as f64);
            return Ok(format!("{bar} {pct}% {icon} [{status}]"));
        }
        Ok("No battery".to_string())
    }
}

#[cfg(target_os = "linux")]
fn get_battery_info() -> Option<(u8, String)> {
    for bat in &["BAT0", "BAT1", "BAT"] {
        let base = format!("/sys/class/power_supply/{bat}");
        if let Ok(cap) = fs::read_to_string(format!("{base}/capacity")) {
            let pct    = cap.trim().parse::<u8>().unwrap_or(0);
            let status = fs::read_to_string(format!("{base}/status"))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "Unknown".to_string());
            return Some((pct, status));
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn get_battery_info() -> Option<(u8, String)> {
    if let Ok(out) = std::process::Command::new("pmset").args(["-g", "batt"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        // Look for line like:  -InternalBattery-0 (id=4653155)        100%; discharging; 6:49 remaining
        for line in text.lines() {
            if line.contains("InternalBattery") || line.contains("Battery") {
                if let Some(pct_str) = line.split('\t').nth(1).or_else(|| line.split("  ").last()) {
                    let parts: Vec<&str> = pct_str.split(';').map(|s| s.trim()).collect();
                    if parts.is_empty() { continue; }
                    let pct = parts[0].replace("%", "").parse::<u8>().unwrap_or(0);
                    let mut status = "Unknown".to_string();
                    if parts.len() > 1 {
                        status = parts[1].to_string();
                        // Capitalize first letter
                        if let Some(c) = status.chars().next() {
                            status = format!("{}{}", c.to_uppercase(), &status[1..]);
                        }
                    }
                    if status == "Charged" { status = "Full".to_string(); }
                    return Some((pct, status));
                }
            }
        }
    }
    None
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_battery_info() -> Option<(u8, String)> {
    None
}
