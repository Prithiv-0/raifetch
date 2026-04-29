#[cfg(target_os = "macos")]
use super::macos_display;
use super::{run_command_stdout, InfoModule};
use std::path::Path;
use std::time::Duration;

pub struct GpuModule {
    timeout: Duration,
    show_temp: bool,
    show_vram: bool,
}

impl GpuModule {
    pub fn new(timeout: Duration, show_temp: bool, show_vram: bool) -> Self {
        Self {
            timeout,
            show_temp,
            show_vram,
        }
    }
}

impl InfoModule for GpuModule {
    fn key(&self) -> &'static str {
        "GPU"
    }

    fn value(&self) -> anyhow::Result<String> {
        let gpus = get_gpus(self.timeout);
        if gpus.is_empty() {
            return Ok("Unknown".to_string());
        }

        let mut extras = Vec::new();
        if self.show_temp {
            if let Some(temp) = read_gpu_temp_c() {
                extras.push(format!("Temp {:.1}C", temp));
            }
        }
        if self.show_vram {
            if let Some(vram) = read_gpu_vram() {
                extras.push(format!("VRAM {vram}"));
            }
        }

        let mut out = gpus.join(", ");
        if !extras.is_empty() {
            out.push_str(" | ");
            out.push_str(&extras.join(" | "));
        }
        Ok(out)
    }
}

#[cfg(target_os = "linux")]
fn get_gpus(timeout: Duration) -> Vec<String> {
    let mut gpus = Vec::new();
    if let Some(cached) = read_boot_cache() {
        return cached;
    }
    if let Some(text) = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("lspci");
            cmd.arg("-mm");
            cmd
        },
        timeout,
    ) {
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("vga compatible controller")
                || lower.contains("3d controller")
                || lower.contains("display controller")
            {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut in_quote = false;
                for c in line.chars() {
                    if c == '"' {
                        in_quote = !in_quote;
                    } else if c == ' ' && !in_quote {
                        if !current.is_empty() {
                            parts.push(current.clone());
                            current.clear();
                        }
                    } else {
                        current.push(c);
                    }
                }
                if !current.is_empty() {
                    parts.push(current);
                }

                if parts.len() >= 4 {
                    let mut vendor = parts[2]
                        .replace(" Corporation", "")
                        .replace(" Inc.", "")
                        .replace(" Micro Devices", "")
                        .replace(" Technology", "");

                    // Handle "Advanced Micro Devices, Inc. [AMD/ATI]"
                    if vendor.contains("Advanced Micro Devices") {
                        vendor = "AMD".to_string();
                    }

                    // Extract exact name from brackets if present, e.g. "Alder Lake-S [UHD Graphics]" -> "UHD Graphics"
                    let mut model = parts[3].clone();
                    if let Some(start) = model.find('[') {
                        if let Some(end) = model.rfind(']') {
                            if end > start {
                                model = model[start + 1..end].to_string();
                            }
                        }
                    }

                    // Further clean model name
                    model = model.replace(" Series", "");

                    gpus.push(format!("{} {}", vendor, model));
                }
            }
        }
    }
    write_boot_cache(&gpus);
    gpus
}

#[cfg(target_os = "macos")]
fn get_gpus(timeout: Duration) -> Vec<String> {
    macos_display::macos_display_info(timeout)
        .map(|info| info.gpus.clone())
        .unwrap_or_default()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_gpus(_timeout: Duration) -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "linux")]
fn read_boot_cache() -> Option<Vec<String>> {
    let boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").ok()?;
    let boot_id = boot_id.trim();
    if boot_id.is_empty() {
        return None;
    }
    let path = format!("/tmp/raifetch_gpu_{}.cache", boot_id);
    let text = std::fs::read_to_string(path).ok()?;
    let gpus: Vec<String> = text
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}

#[cfg(target_os = "linux")]
fn write_boot_cache(gpus: &[String]) {
    let boot_id = std::fs::read_to_string("/proc/sys/kernel/random/boot_id").ok();
    let Some(boot_id) = boot_id else {
        return;
    };
    let boot_id = boot_id.trim();
    if boot_id.is_empty() {
        return;
    }
    let path = format!("/tmp/raifetch_gpu_{}.cache", boot_id);
    let data = gpus.join("\n");
    let _ = std::fs::write(path, data);
}

// ─── Optional extras ────────────────────────────────────────────────────────

fn read_gpu_temp_c() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let dir = std::fs::read_dir("/sys/class/drm").ok()?;
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !is_card_name(&name) {
                continue;
            }
            let hwmon = entry.path().join("device/hwmon");
            if let Some(temp) = read_temp_from_hwmon_dir(&hwmon) {
                return Some(temp);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_temp_from_hwmon_dir(path: &Path) -> Option<f64> {
    let dirs = std::fs::read_dir(path).ok()?;
    for dir in dirs.flatten() {
        let entries = std::fs::read_dir(dir.path()).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("temp") && name.ends_with("_input") {
                if let Some(temp) = read_temp_value(&entry.path()) {
                    return Some(temp);
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_temp_value(path: &Path) -> Option<f64> {
    let raw = std::fs::read_to_string(path).ok()?;
    let value = raw.trim().parse::<f64>().ok()?;
    if value > 1000.0 {
        Some(value / 1000.0)
    } else {
        Some(value)
    }
}

fn read_gpu_vram() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let dir = std::fs::read_dir("/sys/class/drm").ok()?;
        let mut total = 0u64;
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !is_card_name(&name) {
                continue;
            }
            let path = entry.path().join("device/mem_info_vram_total");
            if let Some(bytes) = read_u64_file(&path) {
                total = total.saturating_add(bytes);
            }
        }
        if total > 0 {
            return Some(format_bytes(total));
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn is_card_name(name: &str) -> bool {
    name.strip_prefix("card")
        .map(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn read_u64_file(path: &Path) -> Option<u64> {
    std::fs::read_to_string(path)
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

#[cfg(target_os = "linux")]
fn format_bytes(bytes: u64) -> String {
    const GIB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MIB: f64 = 1024.0 * 1024.0;
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GiB", bytes as f64 / GIB)
    } else {
        format!("{:.0} MiB", bytes as f64 / MIB)
    }
}
