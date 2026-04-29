use crate::config::ModulesConfig;
use crate::theme::Theme;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub mod battery;
pub mod bios;
pub mod boot;
pub mod bootloader;
pub mod cpu;
pub mod disk;
pub mod display;
pub mod entropy;
pub mod font;
pub mod gpu;
pub mod gtk;
pub mod host;
pub mod icons;
pub mod init;
pub mod kernel;
pub mod locale;
#[cfg(target_os = "macos")]
pub mod macos_display;
pub mod memory;
pub mod mobo;
pub mod network;
pub mod os;
pub mod packages;
pub mod processes;
pub mod procfs;
pub mod resolution;
pub mod shell;
pub mod swap;
pub mod terminal;
pub mod theme;
pub mod uptime;
pub mod users;
pub mod wm;

pub use battery::BatteryModule;
pub use bios::BiosModule;
pub use boot::BootModule;
pub use bootloader::BootloaderModule;
pub use cpu::CpuModule;
pub use disk::DiskModule;
pub use display::DisplayModule;
pub use entropy::EntropyModule;
pub use font::FontModule;
pub use gpu::GpuModule;
pub use host::HostModule;
pub use icons::IconsModule;
pub use init::InitModule;
pub use kernel::KernelModule;
pub use locale::LocaleModule;
pub use memory::MemoryModule;
pub use mobo::MoboModule;
pub use network::NetworkModule;
pub use os::OsModule;
pub use packages::PackagesModule;
pub use processes::ProcessesModule;
pub use resolution::ResolutionModule;
pub use shell::ShellModule;
pub use swap::SwapModule;
pub use terminal::TerminalModule;
pub use theme::ThemeModule;
pub use uptime::UptimeModule;
pub use users::UsersModule;
pub use wm::WmModule;

// ─── Trait ───────────────────────────────────────────────────────────────────

pub trait InfoModule: Send + Sync {
    fn key(&self) -> &'static str;
    fn value(&self) -> anyhow::Result<String>;
    fn run_in_background(&self) -> bool {
        false
    }
}

// ─── Custom shell module ──────────────────────────────────────────────────────

struct ShellCmdModule {
    key: String,
    command: String,
    timeout: Duration,
}

impl InfoModule for ShellCmdModule {
    fn key(&self) -> &'static str {
        Box::leak(self.key.clone().into_boxed_str())
    }
    fn value(&self) -> anyhow::Result<String> {
        let out = run_command_stdout(
            {
                let mut cmd = Command::new("sh");
                cmd.args(["-c", &self.command]);
                cmd
            },
            self.timeout,
        )
        .unwrap_or_default();
        Ok(out.trim().to_string())
    }
    fn run_in_background(&self) -> bool {
        true
    }
}

// ─── Builder ─────────────────────────────────────────────────────────────────

pub fn build_modules(
    cfg: &ModulesConfig,
    theme: &Theme,
    de_value: &str,
    auto_hide_wm: bool,
    command_timeout: Duration,
) -> Vec<Box<dyn InfoModule>> {
    let wm_val = wm::detect_wm(command_timeout);
    let hide_wm = auto_hide_wm && !wm_val.is_empty() && wm_val == de_value;

    let all: Vec<(&'static str, bool, Box<dyn InfoModule>)> = vec![
        ("os", cfg.show_os, Box::new(OsModule::new())),
        (
            "host",
            cfg.show_host,
            Box::new(HostModule::new(command_timeout)),
        ),
        ("mobo", cfg.show_mobo, Box::new(MoboModule::new())),
        ("bios", cfg.show_bios, Box::new(BiosModule::new())),
        (
            "kernel",
            cfg.show_kernel,
            Box::new(KernelModule::new(command_timeout)),
        ),
        (
            "boot",
            cfg.show_boot,
            Box::new(BootModule::new(command_timeout)),
        ),
        (
            "bootloader",
            cfg.show_bootloader,
            Box::new(BootloaderModule::new()),
        ),
        ("init", cfg.show_init, Box::new(InitModule::new())),
        (
            "uptime",
            cfg.show_uptime,
            Box::new(UptimeModule::new(command_timeout)),
        ),
        (
            "processes",
            cfg.show_processes,
            Box::new(ProcessesModule::new()),
        ),
        ("users", cfg.show_users, Box::new(UsersModule::new())),
        (
            "cpu",
            cfg.show_cpu,
            Box::new(CpuModule::new(
                command_timeout,
                cfg.cpu_show_temp,
                cfg.cpu_show_cache,
            )),
        ),
        (
            "gpu",
            cfg.show_gpu,
            Box::new(GpuModule::new(
                command_timeout,
                cfg.gpu_show_temp,
                cfg.gpu_show_vram,
            )),
        ),
        (
            "memory",
            cfg.show_memory,
            Box::new(MemoryModule::new(theme.clone(), command_timeout)),
        ),
        (
            "swap",
            cfg.show_swap,
            Box::new(SwapModule::new(theme.clone(), command_timeout)),
        ),
        (
            "disk",
            cfg.show_disk,
            Box::new(DiskModule::new(cfg.show_all_disks, theme.clone())),
        ),
        (
            "battery",
            cfg.show_battery,
            Box::new(BatteryModule::new(theme.clone(), command_timeout)),
        ),
        (
            "network",
            cfg.show_network,
            Box::new(NetworkModule::new(command_timeout)),
        ),
        (
            "resolution",
            cfg.show_resolution,
            Box::new(ResolutionModule::new(command_timeout)),
        ),
        ("display", cfg.show_display, Box::new(DisplayModule::new())),
        ("theme", cfg.show_theme, Box::new(ThemeModule::new())),
        ("icons", cfg.show_icons, Box::new(IconsModule::new())),
        ("font", cfg.show_font, Box::new(FontModule::new())),
        (
            "shell",
            cfg.show_shell,
            Box::new(ShellModule::new(command_timeout)),
        ),
        (
            "terminal",
            cfg.show_terminal,
            Box::new(TerminalModule::new()),
        ),
        ("de", cfg.show_de, Box::new(WmModule::de(command_timeout))),
        (
            "wm",
            cfg.show_wm && !hide_wm,
            Box::new(WmModule::wm(command_timeout)),
        ),
        (
            "packages",
            cfg.show_packages,
            Box::new(PackagesModule::new(
                cfg.packages_extra.clone(),
                command_timeout,
            )),
        ),
        ("locale", cfg.show_locale, Box::new(LocaleModule::new())),
        ("entropy", cfg.show_entropy, Box::new(EntropyModule::new())),
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

    for cm in &cfg.custom {
        result.push(Box::new(ShellCmdModule {
            key: cm.key.clone(),
            command: cm.command.clone(),
            timeout: command_timeout,
        }));
    }

    result
}

// ─── Command helpers ─────────────────────────────────────────────────────────

pub fn run_command_timeout(mut cmd: Command, timeout: Duration) -> Option<Output> {
    use std::io::Read;

    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().ok()?;
    let start = Instant::now();

    loop {
        if let Some(status) = child.try_wait().ok()? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut out) = child.stdout.take() {
                let _ = out.read_to_end(&mut stdout);
            }
            if let Some(mut err) = child.stderr.take() {
                let _ = err.read_to_end(&mut stderr);
            }
            return Some(Output {
                status,
                stdout,
                stderr,
            });
        }
        if start.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        let remaining = timeout.saturating_sub(elapsed);
        if remaining > Duration::from_millis(2) {
            std::thread::sleep(Duration::from_millis(1));
        } else {
            std::thread::yield_now();
        }
    }
}

pub fn run_command_stdout(cmd: Command, timeout: Duration) -> Option<String> {
    run_command_timeout(cmd, timeout).and_then(|o| String::from_utf8(o.stdout).ok())
}
