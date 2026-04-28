/// Direct /proc filesystem parsers — replaces sysinfo for one-shot fetch.
/// Reading /proc/meminfo and /proc/cpuinfo directly avoids sysinfo's
/// caching/allocation overhead (~3ms) that exists for time-series monitors.

// ─── Memory ──────────────────────────────────────────────────────────────────

pub struct MemInfo {
    /// Total physical RAM in bytes
    pub mem_total:     u64,
    /// Available RAM in bytes (MemAvailable — accounts for reclaimable cache)
    pub mem_available: u64,
    /// Total swap in bytes
    pub swap_total:    u64,
    /// Free swap in bytes
    pub swap_free:     u64,
}

impl MemInfo {
    pub fn mem_used(&self) -> u64 { self.mem_total.saturating_sub(self.mem_available) }
    pub fn swap_used(&self) -> u64 { self.swap_total.saturating_sub(self.swap_free) }
}

/// Parse /proc/meminfo.
/// Uses MemAvailable (not MemFree) for accurate "free" memory —
/// MemFree excludes reclaimable buffers/cache and would show too-low free memory.
pub fn read_meminfo() -> anyhow::Result<MemInfo> {
    let text = std::fs::read_to_string("/proc/meminfo")?;
    let mut mem_total     = 0u64;
    let mut mem_available = 0u64;
    let mut swap_total    = 0u64;
    let mut swap_free     = 0u64;

    for line in text.lines() {
        // Format: "FieldName:   12345 kB"
        let mut parts = line.splitn(2, ':');
        let key = parts.next().unwrap_or("").trim();
        let val = parts.next().unwrap_or("").trim()
            .split_whitespace().next().unwrap_or("0")
            .parse::<u64>().unwrap_or(0) * 1024; // kB → bytes

        match key {
            "MemTotal"     => mem_total     = val,
            "MemAvailable" => mem_available = val,
            "SwapTotal"    => swap_total    = val,
            "SwapFree"     => swap_free     = val,
            _ => {}
        }
    }
    Ok(MemInfo { mem_total, mem_available, swap_total, swap_free })
}

// ─── CPU ─────────────────────────────────────────────────────────────────────

pub struct CpuInfo {
    pub model:          String,
    pub logical_cores:  usize,
}

/// Parse /proc/cpuinfo for model name and core count.
pub fn read_cpuinfo() -> CpuInfo {
    let text = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut model         = String::from("Unknown");
    let mut logical_cores = 0usize;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("model name") {
            // Format: "model name\t: Intel(R) Core(TM) ..."
            if model == "Unknown" {
                model = rest.trim_start_matches(['\t', ' ', ':'])
                    .trim()
                    .replace("(R)", "").replace("(TM)", "")
                    .split_whitespace().collect::<Vec<_>>().join(" ");
            }
        }
        if line.starts_with("processor") { logical_cores += 1; }
    }
    if logical_cores == 0 { logical_cores = 1; }
    CpuInfo { model, logical_cores }
}

// ─── Uptime ──────────────────────────────────────────────────────────────────

/// Read uptime in seconds from /proc/uptime.
pub fn read_uptime_secs() -> u64 {
    std::fs::read_to_string("/proc/uptime").ok()
        .and_then(|s| s.split_whitespace().next()
            .and_then(|v| v.parse::<f64>().ok()))
        .map(|f| f as u64)
        .unwrap_or(0)
}
