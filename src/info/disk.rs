use super::InfoModule;
use crate::theme::Theme;

pub struct DiskModule { show_all: bool, theme: Theme }
impl DiskModule {
    pub fn new(show_all: bool, theme: Theme) -> Self { Self { show_all, theme } }
}

impl InfoModule for DiskModule {
    fn key(&self) -> &'static str { "Disk" }
    fn value(&self) -> anyhow::Result<String> {
        let mounts = read_mounts()?;
        let targets: Vec<&MountEntry> = if self.show_all {
            mounts.iter().collect()
        } else {
            mounts.iter().find(|m| m.mount == "/")
                .or_else(|| mounts.first())
                .into_iter().collect()
        };
        if targets.is_empty() { return Ok("No disk found".to_string()); }

        let lines: Vec<String> = targets.iter().map(|m| {
            let total = m.total as f64 / 1_073_741_824.0;
            let avail = m.avail as f64 / 1_073_741_824.0;
            let used  = total - avail;
            let pct   = (used / total.max(0.001)) * 100.0;
            let bar   = self.theme.bar(pct);
            format!("{bar} {used:.1} GiB / {total:.1} GiB ({pct:.0}%) [{}] ({})", m.fstype, m.mount)
        }).collect();

        Ok(lines.join("\n       "))
    }
}

struct MountEntry { mount: String, fstype: String, total: u64, avail: u64 }

/// Read disk stats directly from /proc/mounts + statvfs.
fn read_mounts() -> anyhow::Result<Vec<MountEntry>> {
    let text = std::fs::read_to_string("/proc/mounts")?;
    let mut out = Vec::new();

    for line in text.lines() {
        let mut cols = line.split_whitespace();
        let _dev    = cols.next().unwrap_or("");
        let mount   = cols.next().unwrap_or("").to_string();
        let fstype  = cols.next().unwrap_or("").to_string();

        // Skip pseudo/virtual filesystems
        if matches!(fstype.as_str(),
            "proc"|"sysfs"|"devtmpfs"|"devpts"|"tmpfs"|"cgroup"|"cgroup2"|
            "pstore"|"efivarfs"|"debugfs"|"tracefs"|"securityfs"|"hugetlbfs"|
            "mqueue"|"fusectl"|"fuse.portal"|"bpf"|"overlay"|"ramfs") {
            continue;
        }

        // statvfs(3) via libc-style syscall through nix-free approach
        if let Some((total, avail)) = statvfs_bytes(&mount) {
            out.push(MountEntry { mount, fstype, total, avail });
        }
    }
    Ok(out)
}

/// Get total/available bytes for a mount point via statfs syscall.
fn statvfs_bytes(mount: &str) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::mem;

    let path = CString::new(mount).ok()?;
    let mut stat: libc_statfs = unsafe { mem::zeroed() };
    let ret = unsafe { statfs(path.as_ptr(), &mut stat) };
    if ret != 0 { return None; }

    let block  = stat.f_bsize as u64;
    let total  = stat.f_blocks * block;
    let avail  = stat.f_bavail * block;
    Some((total, avail))
}

// Minimal statfs binding without the libc crate
#[repr(C)]
struct libc_statfs {
    f_type:    i64,
    f_bsize:   i64,
    f_blocks:  u64,
    f_bfree:   u64,
    f_bavail:  u64,
    f_files:   u64,
    f_ffree:   u64,
    f_fsid:    [i32; 2],
    f_namelen: i64,
    f_frsize:  i64,
    f_flags:   i64,
    f_spare:   [i64; 4],
}

unsafe extern "C" {
    fn statfs(path: *const i8, buf: *mut libc_statfs) -> i32;
}
