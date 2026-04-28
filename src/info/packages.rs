use std::process::Command;
use std::fs;
use std::time::SystemTime;
use super::InfoModule;

pub struct PackagesModule;
impl PackagesModule { pub fn new() -> Self { Self } }

impl InfoModule for PackagesModule {
    fn key(&self) -> &'static str { "Packages" }
    fn value(&self) -> anyhow::Result<String> {
        // Cache in /tmp for 5 minutes
        let uid = libc_getuid();
        let cache_path = format!("/tmp/raifetch_pkg_{uid}");
        if let Some(cached) = read_cache(&cache_path) {
            return Ok(cached);
        }
        let result = collect_packages();
        let _ = fs::write(&cache_path, &result);
        Ok(result)
    }
}

fn collect_packages() -> String {
    let mut parts = Vec::new();

    if cmd_exists("pacman") {
        if let Some(n) = count_lines(&["pacman", "-Qq"]) {
            parts.push(format!("{n} (pacman)"));
        }
    }
    if cmd_exists("dpkg") {
        if let Some(n) = count_lines_filter(&["dpkg", "-l"], "ii ") {
            parts.push(format!("{n} (dpkg)"));
        }
    }
    if cmd_exists("rpm") {
        if let Some(n) = count_lines(&["rpm", "-qa"]) {
            parts.push(format!("{n} (rpm)"));
        }
    }
    if cmd_exists("flatpak") {
        if let Some(n) = count_lines_skip1(&["flatpak", "list"]).filter(|&n| n > 0) {
            parts.push(format!("{n} (flatpak)"));
        }
    }
    if cmd_exists("snap") {
        if let Some(n) = count_lines_skip1(&["snap", "list"]).filter(|&n| n > 0) {
            parts.push(format!("{n} (snap)"));
        }
    }
    if cmd_exists("brew") {
        if let Some(n) = count_lines(&["brew", "list", "--formula"]) {
            parts.push(format!("{n} (brew)"));
        }
    }
    if cmd_exists("port") {
        if let Some(n) = count_lines(&["port", "installed"]) {
            parts.push(format!("{n} (macports)"));
        }
    }
    if cmd_exists("nix-env") {
        if let Some(n) = count_lines(&["nix-env", "-qa", "--installed"]) {
            parts.push(format!("{n} (nix)"));
        }
    }

    if parts.is_empty() { "unknown".to_string() } else { parts.join(", ") }
}

fn read_cache(path: &str) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    if age.as_secs() > 300 { return None; } // 5 minute TTL
    fs::read_to_string(path).ok()
}

fn cmd_exists(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output()
        .map(|o| o.status.success()).unwrap_or(false)
}

fn run(args: &[&str]) -> Option<String> {
    let (prog, rest) = args.split_first()?;
    Command::new(prog).args(rest).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
}

fn count_lines(args: &[&str]) -> Option<usize> {
    run(args).map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
}

fn count_lines_skip1(args: &[&str]) -> Option<usize> {
    run(args).map(|s| s.lines().skip(1).filter(|l| !l.trim().is_empty()).count())
}

fn count_lines_filter(args: &[&str], prefix: &str) -> Option<usize> {
    run(args).map(|s| s.lines().filter(|l| l.starts_with(prefix)).count())
}

fn libc_getuid() -> u32 {
    // Safe syscall wrapper - just read from /proc/self/status instead
    fs::read_to_string("/proc/self/status").ok()
        .and_then(|s| {
            s.lines().find(|l| l.starts_with("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse().ok())
        })
        .unwrap_or(1000)
}
