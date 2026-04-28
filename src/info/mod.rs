use std::sync::Arc;
use sysinfo::System;
use crate::config::ModulesConfig;
use crate::theme::Theme;

pub mod battery;
pub mod cpu;
pub mod disk;
pub mod host;
pub mod kernel;
pub mod locale;
pub mod memory;
pub mod network;
pub mod os;
pub mod packages;
pub mod resolution;
pub mod shell;
pub mod swap;
pub mod terminal;
pub mod uptime;
pub mod wm;

pub use battery::BatteryModule;
pub use cpu::CpuModule;
pub use disk::DiskModule;
pub use host::HostModule;
pub use kernel::KernelModule;
pub use locale::LocaleModule;
pub use memory::MemoryModule;
pub use network::NetworkModule;
pub use os::OsModule;
pub use packages::PackagesModule;
pub use resolution::ResolutionModule;
pub use shell::ShellModule;
pub use swap::SwapModule;
pub use terminal::TerminalModule;
pub use uptime::UptimeModule;
pub use wm::WmModule;

// ─── Trait ───────────────────────────────────────────────────────────────────

pub trait InfoModule: Send + Sync {
    fn key(&self) -> &'static str;
    fn value(&self) -> anyhow::Result<String>;
}

// ─── Custom shell module ──────────────────────────────────────────────────────

struct ShellCmdModule { key: String, command: String }

impl InfoModule for ShellCmdModule {
    fn key(&self) -> &'static str {
        // SAFETY: we leak the string to get 'static, only done once per module
        Box::leak(self.key.clone().into_boxed_str())
    }
    fn value(&self) -> anyhow::Result<String> {
        let out = std::process::Command::new("sh")
            .args(["-c", &self.command])
            .output()?;
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }
}

// ─── Builder ─────────────────────────────────────────────────────────────────

pub fn build_modules(
    sys: Arc<System>,
    cfg: &ModulesConfig,
    theme: &Theme,
    de_value: &str,      // pre-detected DE so WM can be suppressed
    auto_hide_wm: bool,
) -> Vec<Box<dyn InfoModule>> {
    let wm_val = wm::detect_wm();
    let hide_wm = auto_hide_wm && !wm_val.is_empty() && wm_val == de_value;

    let all: Vec<(&'static str, bool, Box<dyn InfoModule>)> = vec![
        ("os",         cfg.show_os,         Box::new(OsModule::new())),
        ("host",       cfg.show_host,        Box::new(HostModule::new())),
        ("kernel",     cfg.show_kernel,      Box::new(KernelModule::new())),
        ("uptime",     cfg.show_uptime,      Box::new(UptimeModule::new(Arc::clone(&sys)))),
        ("cpu",        cfg.show_cpu,         Box::new(CpuModule::new(Arc::clone(&sys)))),
        ("memory",     cfg.show_memory,      Box::new(MemoryModule::new(Arc::clone(&sys), theme.clone()))),
        ("swap",       cfg.show_swap,        Box::new(SwapModule::new(Arc::clone(&sys), theme.clone()))),
        ("disk",       cfg.show_disk,        Box::new(DiskModule::new(cfg.show_all_disks, theme.clone()))),
        ("battery",    cfg.show_battery,     Box::new(BatteryModule::new(theme.clone()))),
        ("network",    cfg.show_network,     Box::new(NetworkModule::new())),
        ("resolution", cfg.show_resolution,  Box::new(ResolutionModule::new())),
        ("shell",      cfg.show_shell,       Box::new(ShellModule::new())),
        ("terminal",   cfg.show_terminal,    Box::new(TerminalModule::new())),
        ("de",         cfg.show_de,          Box::new(WmModule::de())),
        ("wm",         cfg.show_wm && !hide_wm, Box::new(WmModule::wm())),
        ("packages",   cfg.show_packages,    Box::new(PackagesModule::new())),
        ("locale",     cfg.show_locale,      Box::new(LocaleModule::new())),
    ];

    let mut by_name: std::collections::HashMap<&str, Box<dyn InfoModule>> = all
        .into_iter()
        .filter(|(_, enabled, _)| *enabled)
        .map(|(name, _, m)| (name, m))
        .collect();

    let mut result: Vec<Box<dyn InfoModule>> = Vec::new();
    for name in &cfg.order {
        if let Some(m) = by_name.remove(name.as_str()) {
            result.push(m);
        }
    }
    result.extend(by_name.into_values());

    // Append custom shell modules at end (in order defined)
    for cm in &cfg.custom {
        result.push(Box::new(ShellCmdModule {
            key: cm.key.clone(),
            command: cm.command.clone(),
        }));
    }

    result
}
