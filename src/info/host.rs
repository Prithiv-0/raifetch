use super::InfoModule;

pub struct HostModule;
impl HostModule { pub fn new() -> Self { Self } }

impl InfoModule for HostModule {
    fn key(&self) -> &'static str { "Host" }
    fn value(&self) -> anyhow::Result<String> {
        Ok(get_host_info().unwrap_or_else(|| whoami::devicename()))
    }
}

#[cfg(target_os = "linux")]
fn get_host_info() -> Option<String> {
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
        (None, Some(p))    => Some(p),
        (Some(v), None)    => Some(v),
        _                  => None,
    }
}

#[cfg(target_os = "macos")]
fn get_host_info() -> Option<String> {
    std::process::Command::new("sysctl").args(["-n", "hw.model"]).output()
        .ok()
        .and_then(|o| {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if s.is_empty() { None } else { Some(format!("Apple {}", s)) }
        })
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_host_info() -> Option<String> {
    None
}
