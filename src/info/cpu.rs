use super::InfoModule;
use super::procfs;

pub struct CpuModule;
impl CpuModule { pub fn new() -> Self { Self } }

impl InfoModule for CpuModule {
    fn key(&self) -> &'static str { "CPU" }
    fn value(&self) -> anyhow::Result<String> {
        let cpu = procfs::read_cpuinfo();

        // Frequency: /sys is more accurate than /proc/cpuinfo's "cpu MHz" field
        let freq_str = sys_cpu_freq()
            .map(|mhz| format!(" @ {:.2} GHz", mhz as f64 / 1000.0))
            .unwrap_or_default();

        Ok(format!("{} ({} cores){}", cpu.model, cpu.logical_cores, freq_str))
    }
}

/// Read current CPU frequency from /sys (kHz → MHz).
fn sys_cpu_freq() -> Option<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|khz| khz / 1000)
}
