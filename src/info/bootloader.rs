use super::InfoModule;

pub struct BootloaderModule;
impl BootloaderModule {
    pub fn new() -> Self {
        Self
    }
}

impl InfoModule for BootloaderModule {
    fn key(&self) -> &'static str {
        "Bootloader"
    }
    fn value(&self) -> anyhow::Result<String> {
        Ok(detect_bootloader().unwrap_or_else(|| "unknown".to_string()))
    }
}

#[cfg(target_os = "linux")]
fn detect_bootloader() -> Option<String> {
    let uefi = std::path::Path::new("/sys/firmware/efi").exists();
    let firmware = if uefi { "UEFI" } else { "BIOS" };

    let loader = if std::path::Path::new("/boot/loader/loader.conf").exists()
        || std::path::Path::new("/boot/EFI/systemd/systemd-bootx64.efi").exists()
    {
        Some("systemd-boot".to_string())
    } else if std::path::Path::new("/boot/grub/grub.cfg").exists()
        || std::path::Path::new("/boot/grub2/grub.cfg").exists()
    {
        Some("grub".to_string())
    } else if std::path::Path::new("/boot/EFI/refind/refind.conf").exists() {
        Some("rEFInd".to_string())
    } else {
        None
    };

    Some(match loader {
        Some(l) => format!("{l} ({firmware})"),
        None => firmware.to_string(),
    })
}

#[cfg(not(target_os = "linux"))]
fn detect_bootloader() -> Option<String> {
    None
}
