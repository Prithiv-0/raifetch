use super::InfoModule;

pub struct HostModule;
impl HostModule { pub fn new() -> Self { Self } }

impl InfoModule for HostModule {
    fn key(&self) -> &'static str { "Host" }
    fn value(&self) -> anyhow::Result<String> {
        // Try /sys/class/dmi/id/product_name (board/laptop model)
        let product = std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .map(|s| s.trim().to_string())
            .ok()
            .filter(|s| !s.is_empty() && s != "To Be Filled By O.E.M.");

        let vendor = std::fs::read_to_string("/sys/class/dmi/id/sys_vendor")
            .map(|s| s.trim().to_string())
            .ok()
            .filter(|s| !s.is_empty() && s != "To Be Filled By O.E.M.");

        match (vendor, product) {
            (Some(v), Some(p)) => Ok(format!("{v} {p}")),
            (None, Some(p))    => Ok(p),
            (Some(v), None)    => Ok(v),
            _                  => Ok(whoami::devicename()),
        }
    }
}
