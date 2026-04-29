#[cfg(target_os = "macos")]
use super::run_command_stdout;
use super::InfoModule;
use std::time::Duration;

pub struct HostModule {
    timeout: Duration,
}
impl HostModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for HostModule {
    fn key(&self) -> &'static str {
        "Host"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(get_host_info(self.timeout).unwrap_or_else(|| whoami::devicename()))
    }
}

#[cfg(target_os = "linux")]
fn get_host_info(_timeout: Duration) -> Option<String> {
    let product = std::fs::read_to_string("/sys/class/dmi/id/product_name")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty() && s != "To Be Filled By O.E.M.");

    let vendor = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty() && s != "To Be Filled By O.E.M.");

    match (vendor, product) {
        (Some(v), Some(p)) => Some(format!("{v} {p}")),
        (None, Some(p)) => Some(p),
        (Some(v), None) => Some(v),
        _ => None,
    }
}

#[cfg(target_os = "macos")]
fn get_host_info(timeout: Duration) -> Option<String> {
    run_command_stdout(
        {
            let mut cmd = std::process::Command::new("sysctl");
            cmd.args(["-n", "hw.model"]);
            cmd
        },
        timeout,
    )
    .and_then(|s| {
        let v = s.trim().to_string();
        if v.is_empty() {
            None
        } else {
            Some(format!("Apple {}", v))
        }
    })
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_host_info(_timeout: Duration) -> Option<String> {
    None
}
