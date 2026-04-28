use super::InfoModule;
use std::fs;
use crate::theme::Theme;

pub struct BatteryModule { theme: Theme }
impl BatteryModule { pub fn new(theme: Theme) -> Self { Self { theme } } }

impl InfoModule for BatteryModule {
    fn key(&self) -> &'static str { "Battery" }
    fn value(&self) -> anyhow::Result<String> {
        for bat in &["BAT0", "BAT1", "BAT"] {
            let base = format!("/sys/class/power_supply/{bat}");
            if let Ok(cap) = fs::read_to_string(format!("{base}/capacity")) {
                let pct    = cap.trim().parse::<u8>().unwrap_or(0);
                let status = fs::read_to_string(format!("{base}/status"))
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|_| "Unknown".to_string());
                let icon = match status.as_str() {
                    "Charging" => "⚡", "Full" => "🔌", _ => "🔋",
                };
                let bar = self.theme.bar(pct as f64);
                return Ok(format!("{bar} {pct}% {icon} [{status}]"));
            }
        }
        Ok("No battery".to_string())
    }
}
