#[cfg(target_os = "macos")]
use super::run_command_stdout;
use super::InfoModule;
use std::time::Duration;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct CpuModule {
    timeout: Duration,
    show_temp: bool,
    show_cache: bool,
}
impl CpuModule {
    pub fn new(timeout: Duration, show_temp: bool, show_cache: bool) -> Self {
        Self {
            timeout,
            show_temp,
            show_cache,
        }
    }
}

impl InfoModule for CpuModule {
    fn key(&self) -> &'static str {
        "CPU"
    }
    fn value(&self) -> anyhow::Result<String> {
        let cpu = get_cpu_info(self.timeout);

        // Frequency: /sys is more accurate than /proc/cpuinfo's "cpu MHz" field
        let freq_str = get_cpu_freq(self.timeout)
            .map(|mhz| format!(" @ {:.2} GHz", mhz as f64 / 1000.0))
            .unwrap_or_default();

        let mut extra = Vec::new();
        if self.show_temp {
            if let Some(temp) = read_cpu_temp_c() {
                extra.push(format!("Temp {:.1}C", temp));
            }
        }
        if self.show_cache {
            if let Some(cache) = read_l3_cache() {
                extra.push(format!("L3 {cache}"));
            }
        }

        let mut out = format!("{} ({} cores){}", cpu.model, cpu.logical_cores, freq_str);
        if !extra.is_empty() {
            out.push_str(" | ");
            out.push_str(&extra.join(" | "));
        }
        Ok(out)
    }
}

pub struct CpuInfo {
    pub model: String,
    pub logical_cores: usize,
}

#[cfg(target_os = "linux")]
fn get_cpu_info(_timeout: Duration) -> CpuInfo {
    let info = procfs::read_cpuinfo();
    CpuInfo {
        model: info.model,
        logical_cores: info.logical_cores,
    }
}

#[cfg(target_os = "macos")]
fn get_cpu_info(timeout: Duration) -> CpuInfo {
    let model = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "machdep.cpu.brand_string"]);
            cmd
        },
        timeout,
    )
    .map(|s| s.trim().to_string())
    .unwrap_or_else(|| "Unknown".to_string());
    let logical_cores = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "hw.logicalcpu"]);
            cmd
        },
        timeout,
    )
    .and_then(|s| s.trim().parse().ok())
    .unwrap_or(1);
    CpuInfo {
        model,
        logical_cores,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_cpu_info(_timeout: Duration) -> CpuInfo {
    CpuInfo {
        model: "Unknown".to_string(),
        logical_cores: 1,
    }
}

#[cfg(target_os = "linux")]
/// Read current CPU frequency from /sys (kHz → MHz).
fn get_cpu_freq(_timeout: Duration) -> Option<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|khz| khz / 1000)
}

#[cfg(target_os = "macos")]
fn get_cpu_freq(timeout: Duration) -> Option<u64> {
    run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "hw.cpufrequency"]);
            cmd
        },
        timeout,
    )
    .and_then(|s| s.trim().parse::<u64>().ok())
    .map(|hz| hz / 1_000_000)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_cpu_freq(_timeout: Duration) -> Option<u64> {
    None
}

// ─── Extras ─────────────────────────────────────────────────────────────────

fn read_cpu_temp_c() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        if let Some(t) = read_cpu_temp_from_thermal() {
            return Some(t);
        }
        if let Some(t) = read_cpu_temp_from_hwmon() {
            return Some(t);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_cpu_temp_from_thermal() -> Option<f64> {
    let dir = std::fs::read_dir("/sys/class/thermal").ok()?;
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("thermal_zone") {
            continue;
        }
        let base = entry.path();
        let ty = match std::fs::read_to_string(base.join("type")) {
            Ok(ty) => ty,
            Err(_) => continue,
        };
        let ty_lc = ty.trim().to_lowercase();
        if !ty_lc.contains("cpu")
            && !ty_lc.contains("package")
            && !ty_lc.contains("x86_pkg")
            && !ty_lc.contains("tctl")
        {
            continue;
        }
        if let Some(val) = read_temp_value(base.join("temp")) {
            return Some(val);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_cpu_temp_from_hwmon() -> Option<f64> {
    let dir = std::fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in dir.flatten() {
        let base = entry.path();
        let name = std::fs::read_to_string(base.join("name"))
            .ok()
            .unwrap_or_default();
        let name_lc = name.trim().to_lowercase();
        if !name_lc.contains("coretemp") && !name_lc.contains("k10temp") && !name_lc.contains("cpu")
        {
            continue;
        }
        if let Some(val) = find_temp_in_hwmon(&base) {
            return Some(val);
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn find_temp_in_hwmon(base: &std::path::Path) -> Option<f64> {
    let entries = std::fs::read_dir(base).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("temp") && name.ends_with("_input") {
            if let Some(val) = read_temp_value(entry.path()) {
                return Some(val);
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn read_temp_value(path: std::path::PathBuf) -> Option<f64> {
    let s = std::fs::read_to_string(path).ok()?;
    let v = s.trim().parse::<f64>().ok()?;
    if v > 1000.0 {
        Some(v / 1000.0)
    } else {
        Some(v)
    }
}

fn read_l3_cache() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        let dir = std::fs::read_dir("/sys/devices/system/cpu/cpu0/cache").ok()?;
        for entry in dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("index") {
                continue;
            }
            let base = entry.path();
            let level = match std::fs::read_to_string(base.join("level")) {
                Ok(level) => level,
                Err(_) => continue,
            };
            if level.trim() != "3" {
                continue;
            }
            let size = match std::fs::read_to_string(base.join("size")) {
                Ok(size) => size,
                Err(_) => continue,
            };
            let size = size.trim();
            if !size.is_empty() {
                return Some(size.to_string());
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
