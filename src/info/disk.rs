use sysinfo::Disks;
use super::InfoModule;
use crate::theme::Theme;

pub struct DiskModule { show_all: bool, theme: Theme }
impl DiskModule {
    pub fn new(show_all: bool, theme: Theme) -> Self { Self { show_all, theme } }
}

impl InfoModule for DiskModule {
    fn key(&self) -> &'static str { "Disk" }
    fn value(&self) -> anyhow::Result<String> {
        let disks = Disks::new_with_refreshed_list();
        let list  = disks.list();

        let targets: Vec<_> = if self.show_all {
            list.iter().collect()
        } else {
            // Prefer root, else first
            let root = list.iter().find(|d| d.mount_point().as_os_str() == "/");
            root.or_else(|| list.first()).into_iter().collect()
        };

        if targets.is_empty() { return Ok("No disk found".to_string()); }

        let lines: Vec<String> = targets.iter().map(|d| {
            let total = d.total_space() as f64 / 1_073_741_824.0;
            let avail = d.available_space() as f64 / 1_073_741_824.0;
            let used  = total - avail;
            let pct   = (used / total.max(0.001)) * 100.0;
            let fs    = d.file_system().to_string_lossy();
            let mount = d.mount_point().display();
            let bar   = self.theme.bar(pct);
            format!("{bar} {used:.1} GiB / {total:.1} GiB ({pct:.0}%) [{fs}] ({mount})")
        }).collect();

        Ok(lines.join("\n       "))
    }
}
