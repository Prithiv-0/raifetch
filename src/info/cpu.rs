use std::sync::Arc;
use sysinfo::System;
use super::InfoModule;

pub struct CpuModule { sys: Arc<System> }
impl CpuModule { pub fn new(sys: Arc<System>) -> Self { Self { sys } } }

impl InfoModule for CpuModule {
    fn key(&self) -> &'static str { "CPU" }
    fn value(&self) -> anyhow::Result<String> {
        let cpus   = self.sys.cpus();
        let model  = cpus.first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        let cores  = cpus.len();
        let usage: f32 = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cores as f32;

        // Frequency from sysinfo (MHz → GHz)
        let freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
        let freq_str = if freq_mhz > 0 {
            format!(" @ {:.2} GHz", freq_mhz as f64 / 1000.0)
        } else {
            String::new()
        };

        Ok(format!("{model} ({cores} cores){freq_str} [{usage:.0}%]"))
    }
}
