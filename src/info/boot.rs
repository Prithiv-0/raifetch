use super::InfoModule;
use std::time::Duration;

#[cfg(target_os = "macos")]
use super::run_command_stdout;

pub struct BootModule {
    timeout: Duration,
}

impl BootModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for BootModule {
    fn key(&self) -> &'static str {
        "Boot"
    }

    fn value(&self) -> anyhow::Result<String> {
        if let Some(epoch) = read_boot_time_epoch(self.timeout) {
            if let Some(formatted) = format_epoch_local(epoch) {
                return Ok(formatted);
            }
            return Ok(epoch.to_string());
        }
        Ok("unknown".to_string())
    }
}

#[cfg(target_os = "linux")]
fn read_boot_time_epoch(_timeout: Duration) -> Option<i64> {
    let text = std::fs::read_to_string("/proc/stat").ok()?;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("btime ") {
            return rest.trim().parse::<i64>().ok();
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn read_boot_time_epoch(timeout: Duration) -> Option<i64> {
    let text = run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "kern.boottime"]);
            cmd
        },
        timeout,
    )?;
    let sec_str = text.split("sec = ").nth(1)?;
    let sec_val = sec_str.split(',').next()?;
    sec_val.parse::<i64>().ok()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_boot_time_epoch(_timeout: Duration) -> Option<i64> {
    None
}

#[cfg(unix)]
fn format_epoch_local(secs: i64) -> Option<String> {
    use std::mem;

    let t: libc::time_t = secs as libc::time_t;
    let mut tm: libc::tm = unsafe { mem::zeroed() };
    let res = unsafe { libc::localtime_r(&t, &mut tm as *mut libc::tm) };
    if res.is_null() {
        return None;
    }

    let mut buf = [0u8; 64];
    let fmt = b"%Y-%m-%d %H:%M\0";
    let n = unsafe {
        libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            fmt.as_ptr() as *const libc::c_char,
            &tm as *const libc::tm,
        )
    };
    if n == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&buf[..n as usize]).to_string())
}

#[cfg(not(unix))]
fn format_epoch_local(_secs: i64) -> Option<String> {
    None
}
