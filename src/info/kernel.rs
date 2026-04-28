use super::InfoModule;

pub struct KernelModule;

impl KernelModule {
    pub fn new() -> Self { Self }
}

impl InfoModule for KernelModule {
    fn key(&self) -> &'static str { "Kernel" }

    fn value(&self) -> anyhow::Result<String> {
        // Read kernel release from /proc/version_signature or uname
        let version = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| {
                std::process::Command::new("uname").arg("-r").output().ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            });
        Ok(version)
    }
}
