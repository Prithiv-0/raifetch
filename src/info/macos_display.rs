use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

use crate::info::run_command_stdout;

#[derive(Debug, Default, Clone)]
pub struct MacDisplayInfo {
    pub gpus: Vec<String>,
    pub resolutions: Vec<String>,
}

static INFO: OnceLock<Option<MacDisplayInfo>> = OnceLock::new();

pub fn macos_display_info(timeout: Duration) -> Option<&'static MacDisplayInfo> {
    INFO.get_or_init(|| load_info(timeout)).as_ref()
}

fn load_info(timeout: Duration) -> Option<MacDisplayInfo> {
    let text = run_command_stdout(
        {
            let mut cmd = Command::new("system_profiler");
            cmd.args(["SPDisplaysDataType"]);
            cmd
        },
        timeout,
    )?;

    let mut gpus = Vec::new();
    let mut resolutions = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Chipset Model:") {
            gpus.push(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("Resolution:") {
            resolutions.push(rest.trim().to_string());
        }
    }

    if gpus.is_empty() && resolutions.is_empty() {
        None
    } else {
        Some(MacDisplayInfo { gpus, resolutions })
    }
}
