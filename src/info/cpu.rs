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

        // Read CPU usage directly from /proc/stat with a 50ms sample
        // This avoids the mandatory 200ms sysinfo sleep
        let usage = proc_stat_usage(50).unwrap_or_else(|| {
            // fallback: use sysinfo value (may be 0 on first read)
            cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cores as f32
        });

        // Frequency: prefer /sys (more accurate than sysinfo on Linux)
        let freq_str = sys_cpu_freq()
            .map(|mhz| format!(" @ {:.2} GHz", mhz as f64 / 1000.0))
            .unwrap_or_else(|| {
                let mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
                if mhz > 0 { format!(" @ {:.2} GHz", mhz as f64 / 1000.0) }
                else { String::new() }
            });

        Ok(format!("{model} ({cores} cores){freq_str} [{usage:.0}%]"))
    }
}

/// Read total CPU utilisation from /proc/stat with a `sample_ms` interval.
fn proc_stat_usage(sample_ms: u64) -> Option<f32> {
    let s1 = read_stat()?;
    std::thread::sleep(std::time::Duration::from_millis(sample_ms));
    let s2 = read_stat()?;

    let idle_diff  = (s2.idle  + s2.iowait) as i64 - (s1.idle  + s1.iowait) as i64;
    let total_diff = s2.total as i64 - s1.total as i64;
    if total_diff <= 0 { return None; }
    Some((1.0 - idle_diff as f32 / total_diff as f32) * 100.0)
}

#[derive(Default)]
struct StatLine { idle: u64, iowait: u64, total: u64 }

fn read_stat() -> Option<StatLine> {
    let text = std::fs::read_to_string("/proc/stat").ok()?;
    let line = text.lines().next()?; // "cpu ..."
    let nums: Vec<u64> = line.split_whitespace()
        .skip(1)
        .filter_map(|s| s.parse().ok())
        .collect();
    // user nice system idle iowait irq softirq steal guest guest_nice
    if nums.len() < 5 { return None; }
    let idle   = nums[3];
    let iowait = nums.get(4).copied().unwrap_or(0);
    let total  = nums.iter().sum();
    Some(StatLine { idle, iowait, total })
}

/// Read current CPU frequency from /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq (kHz → MHz).
fn sys_cpu_freq() -> Option<u64> {
    std::fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|khz| khz / 1000)
}
