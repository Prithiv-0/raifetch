use std::sync::Arc;
use sysinfo::System;
use super::InfoModule;

pub struct CpuModule { sys: Arc<System> }
impl CpuModule { pub fn new(sys: Arc<System>) -> Self { Self { sys } } }

impl InfoModule for CpuModule {
    fn key(&self) -> &'static str { "CPU" }
    fn value(&self) -> anyhow::Result<String> {
        let cpus  = self.sys.cpus();
        let model = cpus.first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cores = cpus.len();

        // Frequency: read from /sys for accuracy, fall back to sysinfo
        let freq_str = sys_cpu_freq()
            .or_else(|| cpus.first().map(|c| c.frequency()).filter(|&f| f > 0))
            .map(|mhz| format!(" @ {:.2} GHz", mhz as f64 / 1000.0))
            .unwrap_or_default();

        Ok(format!("{model} ({cores} cores){freq_str}"))
    }
}

/// Read current CPU frequency from /sys (kHz → MHz).
fn sys_cpu_freq() -> Option<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|khz| khz / 1000)
}
