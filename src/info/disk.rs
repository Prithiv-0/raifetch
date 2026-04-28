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
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "macos")]
fn read_mounts() -> anyhow::Result<Vec<MountEntry>> {
    let mut out = Vec::new();
    let cmd = std::process::Command::new("df").arg("-k").output()?;
    let text = String::from_utf8_lossy(&cmd.stdout);
    let mut lines = text.lines();
    lines.next(); // Skip header
    for line in lines {
        let mut cols = line.split_whitespace();
        let _fs = cols.next().unwrap_or("");
        let total_kb: u64 = cols.next().unwrap_or("0").parse().unwrap_or(0);
        let _used_kb: u64 = cols.next().unwrap_or("0").parse().unwrap_or(0);
        let avail_kb: u64 = cols.next().unwrap_or("0").parse().unwrap_or(0);
        let _capacity = cols.next().unwrap_or("");
        let _iused = cols.next().unwrap_or("");
        let _ifree = cols.next().unwrap_or("");
        let _iused_pct = cols.next().unwrap_or("");
        let mount = cols.collect::<Vec<_>>().join(" ");
        
        if mount.starts_with("/System/Volumes") || mount == "/dev" || mount == "/net" || mount == "/home" {
            continue;
        }
        
        if total_kb > 0 {
            out.push(MountEntry {
                mount: if mount.is_empty() { "/".to_string() } else { mount },
                fstype: "apfs".to_string(), // mostly apfs on mac
                total: total_kb * 1024,
                avail: avail_kb * 1024,
            });
        }
    }
    Ok(out)
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn read_mounts() -> anyhow::Result<Vec<MountEntry>> {
    Ok(Vec::new())
}

#[cfg(target_os = "linux")]
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
#[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
unsafe extern "C" {
    fn statfs(path: *const i8, buf: *mut libc_statfs) -> i32;
}
