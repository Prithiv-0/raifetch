use super::{run_command_stdout, InfoModule};
use std::time::Duration;

pub struct KernelModule {
    timeout: Duration,
}

impl KernelModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for KernelModule {
    fn key(&self) -> &'static str {
        "Kernel"
    }

    fn value(&self) -> anyhow::Result<String> {
        // Read kernel release from /proc/version_signature or uname
        let version = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| {
                run_command_stdout(
                    {
                        let mut cmd = std::process::Command::new("uname");
                        cmd.arg("-r");
                        cmd
                    },
                    self.timeout,
                )
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string())
            });
        Ok(version)
    }
}
