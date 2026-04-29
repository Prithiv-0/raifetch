#[cfg(target_os = "macos")]
use super::macos_display;
use super::{run_command_stdout, InfoModule};
#[cfg(target_os = "linux")]
use std::io::{Read, Write};
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::time::Duration;

pub struct ResolutionModule {
    timeout: Duration,
}
impl ResolutionModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for ResolutionModule {
    fn key(&self) -> &'static str {
        "Resolution"
    }
    fn value(&self) -> anyhow::Result<String> {
        #[cfg(target_os = "macos")]
        if let Some(r) = macos_res(self.timeout) {
            return Ok(r);
        }

        #[cfg(target_os = "linux")]
        {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                if let Some(r) = hyprland_socket_res(self.timeout) {
                    return Ok(r);
                }
                if let Some(r) = drm_res() {
                    return Ok(r);
                }
                if let Some(r) = hyprctl_res(self.timeout) {
                    return Ok(r);
                }
                if let Some(r) = kscreen_res(self.timeout) {
                    return Ok(r);
                }
                if let Some(r) = wlr_randr_res(self.timeout) {
                    return Ok(r);
                }
            }
            if let Some(r) = xrandr_res(self.timeout) {
                return Ok(r);
            }
            if let Some(r) = drm_res() {
                return Ok(r);
            }
        }
        Ok("unknown".to_string())
    }
}

#[cfg(target_os = "linux")]
fn hyprland_socket_res(timeout: Duration) -> Option<String> {
    let runtime = std::env::var("XDG_RUNTIME_DIR").ok()?;
    let signature = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").ok()?;
    if runtime.is_empty() || signature.is_empty() {
        return None;
    }

    let socket = format!("{runtime}/hypr/{signature}/.socket.sock");
    let mut stream = UnixStream::connect(socket).ok()?;
    let io_timeout = Some(timeout.max(Duration::from_millis(1)));
    let _ = stream.set_read_timeout(io_timeout);
    let _ = stream.set_write_timeout(io_timeout);
    stream.write_all(b"j/monitors").ok()?;

    let mut text = String::new();
    stream.read_to_string(&mut text).ok()?;
    parse_hypr_monitors(&text)
}

#[cfg(target_os = "linux")]
fn drm_res() -> Option<String> {
    let dir = std::fs::read_dir("/sys/class/drm").ok()?;
    let mut resolutions = Vec::new();
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
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
        let Some(mode) = std::fs::read_to_string(base.join("modes"))
            .ok()
            .and_then(|s| s.lines().next().map(|line| line.trim().to_string()))
        else {
            continue;
        };
        if !mode.is_empty() {
            resolutions.push(mode.replace('x', "×"));
        }
    }
    if resolutions.is_empty() {
        None
    } else {
        Some(resolutions.join(", "))
    }
}

fn hyprctl_res(timeout: Duration) -> Option<String> {
    let text = run_command_stdout(
        {
            let mut cmd = Command::new("hyprctl");
            cmd.args(["monitors", "-j"]);
            cmd
        },
        timeout,
    )?;
    parse_hypr_monitors(&text)
}

fn parse_hypr_monitors(text: &str) -> Option<String> {
    let mut resolutions = Vec::new();
    for (i, _) in text.match_indices("\"width\"") {
        let sub = &text[i..];
        let w = extract_num(sub, "\"width\":")?;
        let h = extract_num(sub, "\"height\":")?;
        let hz = extract_float(sub, "\"refreshRate\":").unwrap_or(60.0);
        resolutions.push(format!("{w}×{h} @ {hz:.0}Hz"));
    }
    if resolutions.is_empty() {
        None
    } else {
        Some(resolutions.join(", "))
    }
}

fn kscreen_res(timeout: Duration) -> Option<String> {
    let text = run_command_stdout(
        {
            let mut cmd = Command::new("kscreen-doctor");
            cmd.arg("-o");
            cmd
        },
        timeout,
    )?;
    for line in text.lines() {
        if line.contains("Modes:") || line.contains("x") {
            if let Some(res) = line.split_whitespace().find(|s| {
                s.contains('x')
                    && s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            }) {
                return Some(res.to_string());
            }
        }
    }
    None
}

fn wlr_randr_res(timeout: Duration) -> Option<String> {
    let text = run_command_stdout(Command::new("wlr-randr"), timeout)?;
    for line in text.lines() {
        if line.trim_start().starts_with(|c: char| c.is_ascii_digit()) && line.contains('x') {
            return Some(line.trim().split_whitespace().next()?.to_string());
        }
    }
    None
}

fn xrandr_res(timeout: Duration) -> Option<String> {
    let text = run_command_stdout(
        {
            let mut cmd = Command::new("xrandr");
            cmd.arg("--current");
            cmd
        },
        timeout,
    )?;
    let mut results = Vec::new();
    for line in text.lines() {
        if line.contains(" connected") {
            if let Some(res) = line.split_whitespace().find(|s| {
                s.contains('x')
                    && s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            }) {
                results.push(res.split('+').next().unwrap_or(res).to_string());
            }
        }
    }
    if results.is_empty() {
        None
    } else {
        Some(results.join(", "))
    }
}

fn extract_num(s: &str, key: &str) -> Option<u32> {
    let i = s.find(key)? + key.len();
    let rest = s[i..].trim_start_matches(|c: char| c == ' ' || c == ':');
    rest.chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn extract_float(s: &str, key: &str) -> Option<f64> {
    let i = s.find(key)? + key.len();
    let rest = s[i..].trim_start_matches(|c: char| c == ' ' || c == ':');
    rest.chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect::<String>()
        .parse()
        .ok()
}

#[cfg(target_os = "macos")]
fn macos_res(timeout: Duration) -> Option<String> {
    macos_display::macos_display_info(timeout).and_then(|info| {
        if info.resolutions.is_empty() {
            None
        } else {
            Some(info.resolutions.join(", "))
        }
    })
}
