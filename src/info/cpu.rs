use super::InfoModule;

#[cfg(target_os = "linux")]
use super::procfs;

pub struct CpuModule;
impl CpuModule { pub fn new() -> Self { Self } }

impl InfoModule for CpuModule {
    fn key(&self) -> &'static str { "CPU" }
    fn value(&self) -> anyhow::Result<String> {
        let cpu = get_cpu_info();

        // Frequency: /sys is more accurate than /proc/cpuinfo's "cpu MHz" field
        let freq_str = get_cpu_freq()
            .map(|mhz| format!(" @ {:.2} GHz", mhz as f64 / 1000.0))
            .unwrap_or_default();

        Ok(format!("{} ({} cores){}", cpu.model, cpu.logical_cores, freq_str))
    }
}

pub struct CpuInfo {
    pub model: String,
    pub logical_cores: usize,
}

#[cfg(target_os = "linux")]
fn get_cpu_info() -> CpuInfo {
    let info = procfs::read_cpuinfo();
    CpuInfo {
        model: info.model,
        logical_cores: info.logical_cores,
    }
}

#[cfg(target_os = "macos")]
fn get_cpu_info() -> CpuInfo {
    let model = std::process::Command::new("sysctl").args(["-n", "machdep.cpu.brand_string"]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_else(|_| "Unknown".to_string());
    let logical_cores = std::process::Command::new("sysctl").args(["-n", "hw.logicalcpu"]).output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().parse().unwrap_or(1)).unwrap_or(1);
    CpuInfo { model, logical_cores }
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_cpu_info() -> CpuInfo {
    CpuInfo { model: "Unknown".to_string(), logical_cores: 1 }
}

#[cfg(target_os = "linux")]
/// Read current CPU frequency from /sys (kHz → MHz).
fn get_cpu_freq() -> Option<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|khz| khz / 1000)
}

#[cfg(target_os = "macos")]
fn get_cpu_freq() -> Option<u64> {
    std::process::Command::new("sysctl").args(["-n", "hw.cpufrequency"]).output()
        .ok().and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse::<u64>().ok())
        .map(|hz| hz / 1_000_000)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_cpu_freq() -> Option<u64> {
    None
}
