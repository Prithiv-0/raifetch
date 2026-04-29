use super::{run_command_stdout, InfoModule};
use std::fs;
use std::process::Command;
use std::time::{Duration, SystemTime};

pub struct PackagesModule {
    extras: Vec<String>,
    timeout: Duration,
}
impl PackagesModule {
    pub fn new(extras: Vec<String>, timeout: Duration) -> Self {
        Self { extras, timeout }
    }
}

impl InfoModule for PackagesModule {
    fn key(&self) -> &'static str {
        "Packages"
    }
    fn value(&self) -> anyhow::Result<String> {
        // Cache in /tmp for 5 minutes
        let uid = libc_getuid();
        let cache_path = format!("/tmp/raifetch_pkg_{uid}");
        if let Some(cached) = read_cache(&cache_path) {
            return Ok(cached);
        }
        let result = collect_packages(&self.extras, self.timeout);
        let _ = fs::write(&cache_path, &result);
        Ok(result)
    }
}

fn collect_packages(extras: &[String], timeout: Duration) -> String {
    let mut parts = Vec::new();

    let primary = primary_manager(timeout);
    if let Some(pm) = primary {
        if let Some(n) = count_manager(pm, timeout) {
            parts.push(format!("{n} ({})", label_for(pm)));
        }
    } else {
        // Fallback to the first detected manager if distro is unknown.
        for pm in ["pacman", "dpkg", "rpm", "brew", "port", "nix"] {
            if let Some(n) = count_manager(pm, timeout) {
                parts.push(format!("{n} ({})", label_for(pm)));
                break;
            }
        }
    }

    let extras_norm: Vec<String> = extras.iter().map(|s| s.to_lowercase()).collect();
    for pm in extras_norm {
        if Some(pm.as_str()) == primary {
            continue;
        }
        if let Some(n) = count_manager(&pm, timeout) {
            parts.push(format!("{n} ({})", label_for(&pm)));
        }
    }

    if parts.is_empty() {
        "unknown".to_string()
    } else {
        parts.join(", ")
    }
}

fn read_cache(path: &str) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    if age.as_secs() > 300 {
        return None;
    } // 5 minute TTL
    fs::read_to_string(path).ok()
}

fn cmd_exists(cmd_name: &str, timeout: Duration) -> bool {
    let _ = timeout;
    command_exists(cmd_name)
}

fn command_exists(cmd_name: &str) -> bool {
    if cmd_name.contains('/') {
        return std::path::Path::new(cmd_name).is_file();
    }
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(cmd_name).is_file()))
        .unwrap_or(false)
}

fn run(args: &[&str], timeout: Duration) -> Option<String> {
    let (prog, rest) = args.split_first()?;
    run_command_stdout(
        {
            let mut cmd = Command::new(prog);
            cmd.args(rest);
            cmd
        },
        timeout,
    )
}

fn count_lines(args: &[&str], timeout: Duration) -> Option<usize> {
    run(args, timeout).map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
}

fn count_lines_skip1(args: &[&str], timeout: Duration) -> Option<usize> {
    run(args, timeout).map(|s| s.lines().skip(1).filter(|l| !l.trim().is_empty()).count())
}

fn count_lines_filter(args: &[&str], prefix: &str, timeout: Duration) -> Option<usize> {
    run(args, timeout).map(|s| s.lines().filter(|l| l.starts_with(prefix)).count())
}

fn count_manager(pm: &str, timeout: Duration) -> Option<usize> {
    match pm {
        "pacman" => count_pacman_local().or_else(|| {
            if command_exists("pacman") {
                count_lines(&["pacman", "-Qq"], timeout)
            } else {
                None
            }
        }),
        "dpkg" => count_dpkg_status().or_else(|| {
            if command_exists("dpkg") {
                count_lines_filter(&["dpkg", "-l"], "ii ", timeout)
            } else {
                None
            }
        }),
        "rpm" => {
            if cmd_exists("rpm", timeout) {
                count_lines(&["rpm", "-qa"], timeout)
            } else {
                None
            }
        }
        "flatpak" => {
            if cmd_exists("flatpak", timeout) {
                count_lines_skip1(&["flatpak", "list"], timeout).filter(|&n| n > 0)
            } else {
                None
            }
        }
        "snap" => {
            if cmd_exists("snap", timeout) {
                count_lines_skip1(&["snap", "list"], timeout).filter(|&n| n > 0)
            } else {
                None
            }
        }
        "brew" => count_brew(timeout),
        "port" => {
            if cmd_exists("port", timeout) {
                count_lines(&["port", "installed"], timeout)
            } else {
                None
            }
        }
        "nix" => {
            if cmd_exists("nix-env", timeout) {
                count_lines(&["nix-env", "-qa", "--installed"], timeout)
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn count_pacman_local() -> Option<usize> {
    let dir = fs::read_dir("/var/lib/pacman/local").ok()?;
    let mut count = 0usize;
    for entry in dir.flatten() {
        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            count += 1;
        }
    }
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn count_pacman_local() -> Option<usize> {
    None
}

#[cfg(target_os = "linux")]
fn count_dpkg_status() -> Option<usize> {
    let text = fs::read_to_string("/var/lib/dpkg/status").ok()?;
    let count = text
        .split("\n\n")
        .filter(|pkg| {
            pkg.lines()
                .any(|line| line == "Status: install ok installed")
        })
        .count();
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

#[cfg(not(target_os = "linux"))]
fn count_dpkg_status() -> Option<usize> {
    None
}

fn count_brew(timeout: Duration) -> Option<usize> {
    let Some(cellar) = brew_cellar_path(timeout) else {
        return None;
    };
    let mut count = 0usize;
    if let Ok(entries) = fs::read_dir(cellar) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    count += 1;
                }
            }
        }
    }
    if count > 0 {
        Some(count)
    } else {
        None
    }
}

fn brew_cellar_path(timeout: Duration) -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("HOMEBREW_CELLAR") {
        let pb = std::path::PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    for p in ["/opt/homebrew/Cellar", "/usr/local/Cellar"] {
        let pb = std::path::PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    run_command_stdout(
        {
            let mut cmd = Command::new("brew");
            cmd.arg("--cellar");
            cmd
        },
        timeout,
    )
    .map(|s| std::path::PathBuf::from(s.trim().to_string()))
    .filter(|p| p.exists())
}

fn primary_manager(_timeout: Duration) -> Option<&'static str> {
    #[cfg(target_os = "macos")]
    {
        if cmd_exists("brew", _timeout) {
            return Some("brew");
        }
        return None;
    }
    #[cfg(target_os = "linux")]
    {
        let (id, id_like) = os_release_ids();
        let id_like = id_like.join(" ");
        let id = id.unwrap_or_default();
        if matches!(
            id.as_str(),
            "arch" | "archlinux" | "manjaro" | "endeavouros"
        ) || id_like.contains("arch")
        {
            return Some("pacman");
        }
        if matches!(
            id.as_str(),
            "debian" | "ubuntu" | "linuxmint" | "pop" | "raspbian" | "kali"
        ) || id_like.contains("debian")
        {
            return Some("dpkg");
        }
        if matches!(
            id.as_str(),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux"
        ) || id_like.contains("rhel")
            || id_like.contains("fedora")
        {
            return Some("rpm");
        }
        if matches!(id.as_str(), "opensuse" | "sles") || id_like.contains("suse") {
            return Some("rpm");
        }
        if matches!(id.as_str(), "nixos") || id_like.contains("nixos") {
            return Some("nix");
        }
        None
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = _timeout;
        None
    }
}

fn os_release_ids() -> (Option<String>, Vec<String>) {
    let text = fs::read_to_string("/etc/os-release").unwrap_or_default();
    let mut id = None;
    let mut id_like = Vec::new();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("ID=") {
            id = Some(v.trim_matches('"').to_lowercase());
        } else if let Some(v) = line.strip_prefix("ID_LIKE=") {
            id_like = v
                .trim_matches('"')
                .split_whitespace()
                .map(|s| s.to_lowercase())
                .collect();
        }
    }
    (id, id_like)
}

fn label_for(pm: &str) -> String {
    match pm {
        "port" => "macports".to_string(),
        "nix" => "nix".to_string(),
        _ => pm.to_string(),
    }
}

fn libc_getuid() -> u32 {
    // Safe syscall wrapper - just read from /proc/self/status instead
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1000)
}
