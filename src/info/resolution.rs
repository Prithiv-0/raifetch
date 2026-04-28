use std::process::Command;
use super::InfoModule;

pub struct ResolutionModule;
impl ResolutionModule { pub fn new() -> Self { Self } }

impl InfoModule for ResolutionModule {
    fn key(&self) -> &'static str { "Resolution" }
    fn value(&self) -> anyhow::Result<String> {
        #[cfg(target_os = "macos")]
        if let Some(r) = macos_res() { return Ok(r); }

        #[cfg(target_os = "linux")]
        {
            // Wayland: try kscreen-doctor or hyprctl
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                if let Some(r) = hyprctl_res() { return Ok(r); }
                if let Some(r) = kscreen_res() { return Ok(r); }
                if let Some(r) = wlr_randr_res() { return Ok(r); }
            }
            // X11 fallback
            if let Some(r) = xrandr_res() { return Ok(r); }
        }
        Ok("unknown".to_string())
    }
}

fn hyprctl_res() -> Option<String> {
    let out = Command::new("hyprctl").args(["monitors", "-j"]).output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    // Simple parse: look for "width" and "height" JSON fields
    let mut resolutions = Vec::new();
    for (i, _) in text.match_indices("\"width\"") {
        let sub = &text[i..];
        let w = extract_num(sub, "\"width\":")?;
        let h = extract_num(sub, "\"height\":")?;
        let hz = extract_float(sub, "\"refreshRate\":").unwrap_or(60.0);
        resolutions.push(format!("{w}×{h} @ {hz:.0}Hz"));
    }
    if resolutions.is_empty() { None } else { Some(resolutions.join(", ")) }
}

fn kscreen_res() -> Option<String> {
    let out = Command::new("kscreen-doctor").arg("-o").output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.contains("Modes:") || line.contains("x") {
            if let Some(res) = line.split_whitespace()
                .find(|s| s.contains('x') && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
                return Some(res.to_string());
            }
        }
    }
    None
}

fn wlr_randr_res() -> Option<String> {
    let out = Command::new("wlr-randr").output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    for line in text.lines() {
        if line.trim_start().starts_with(|c: char| c.is_ascii_digit()) && line.contains('x') {
            return Some(line.trim().split_whitespace().next()?.to_string());
        }
    }
    None
}

fn xrandr_res() -> Option<String> {
    let out = Command::new("xrandr").arg("--current").output().ok()?;
    let text = String::from_utf8(out.stdout).ok()?;
    let mut results = Vec::new();
    for line in text.lines() {
        if line.contains(" connected") {
            if let Some(res) = line.split_whitespace()
                .find(|s| s.contains('x') && s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)) {
                results.push(res.split('+').next().unwrap_or(res).to_string());
            }
        }
    }
    if results.is_empty() { None } else { Some(results.join(", ")) }
}

fn extract_num(s: &str, key: &str) -> Option<u32> {
    let i = s.find(key)? + key.len();
    let rest = s[i..].trim_start_matches(|c: char| c == ' ' || c == ':');
    rest.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok()
}

fn extract_float(s: &str, key: &str) -> Option<f64> {
    let i = s.find(key)? + key.len();
    let rest = s[i..].trim_start_matches(|c: char| c == ' ' || c == ':');
    rest.chars().take_while(|c| c.is_ascii_digit() || *c == '.').collect::<String>().parse().ok()
}

#[cfg(target_os = "macos")]
fn macos_res() -> Option<String> {
    let out = Command::new("system_profiler").args(["SPDisplaysDataType"]).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let mut results = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Resolution: ") {
            results.push(rest.trim().to_string());
        }
    }
    if results.is_empty() { None } else { Some(results.join(", ")) }
}
