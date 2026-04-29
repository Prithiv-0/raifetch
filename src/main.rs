mod ascii;
mod config;
mod error;
mod image;
mod info;
mod render;
mod theme;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::image::{auto_detect, BlockBackend, ImageBackend, KittyBackend, SixelBackend};
use crate::info::{build_modules, os, wm::detect_de};
use crate::render::render_side_by_side;
use crate::theme::{color_blocks, distro_auto_color, Theme};
use clap::Parser;

// ─── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "raifetch", about = "Fast system info fetch tool", version)]
struct Cli {
    #[arg(short, long)]
    image: Option<String>,
    #[arg(long)]
    no_image: bool,
    #[arg(long, value_name = "BACKEND")]
    backend: Option<String>,
    #[arg(long, value_name = "WHEN", default_value = "auto")]
    color: String,
    /// Use an alternate config file
    #[arg(long, value_name = "PATH")]
    config: Option<String>,
    #[arg(long)]
    config_path: bool,
    #[arg(long)]
    list_modules: bool,
    #[arg(long)]
    generate_config: bool,
    #[arg(long)]
    install: bool,
    /// Remove all cached image renders from /tmp
    #[arg(long)]
    clear_cache: bool,
    /// Print only one module (for status bars)
    #[arg(long, value_name = "MODULE")]
    module: Option<String>,
    /// With --module: print bare value only (no label/color/separator)
    #[arg(long)]
    raw: bool,
    /// Print per-module timings to stderr
    #[arg(long)]
    timings: bool,
}

// ─── Main ────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.color.as_str() {
        "always" => owo_colors::set_override(true),
        "never" => owo_colors::set_override(false),
        _ => {}
    }

    if cli.config_path {
        println!("{}", Config::path().display());
        return Ok(());
    }
    if cli.generate_config {
        println!("{}", Config::default_toml());
        return Ok(());
    }
    if cli.install {
        return do_install();
    }
    if cli.clear_cache {
        return do_clear_cache();
    }

    let cfg = match &cli.config {
        Some(p) => Config::load_from(std::path::Path::new(p))?,
        None => Config::load()?,
    };
    let command_timeout = Duration::from_millis(cfg.general.command_timeout_ms);

    if cli.list_modules {
        let all = [
            "os",
            "host",
            "mobo",
            "bios",
            "kernel",
            "boot",
            "bootloader",
            "init",
            "uptime",
            "processes",
            "users",
            "cpu",
            "gpu",
            "memory",
            "swap",
            "disk",
            "battery",
            "network",
            "resolution",
            "display",
            "theme",
            "icons",
            "font",
            "shell",
            "terminal",
            "de",
            "wm",
            "packages",
            "locale",
            "entropy",
            "colors",
        ];
        println!("Available modules:");
        for m in all {
            println!("  {m}");
        }
        if !cfg.modules.custom.is_empty() {
            println!("Custom modules:");
            for c in &cfg.modules.custom {
                println!("  {}", c.key);
            }
        }
        return Ok(());
    }

    // ── Resolve label color (auto = detect distro brand color) ────────────────
    let label_color = if cfg.theme.label_color == "auto" {
        distro_auto_color()
    } else {
        cfg.theme.label_color.clone()
    };

    // ── Theme ────────────────────────────────────────────────────────────────
    let theme = Theme {
        label_color,
        value_color: cfg.theme.value_color.clone(),
        separator: cfg.general.separator.clone(),
        bold_labels: cfg.theme.bold_labels,
        bar_width: cfg.theme.bar_width,
        bar_fill: cfg.theme.bar_fill,
        bar_empty: cfg.theme.bar_empty,
        icons: cfg.theme.icons,
        align_labels: cfg.theme.align_labels,
    };

    // ── Pre-detect DE for WM dedup ────────────────────────────────────────────
    let de_val = detect_de();

    // ── Build + collect modules ────────────────────────────────────────────────
    let modules = build_modules(
        &cfg.modules,
        &theme,
        &de_val,
        cfg.general.auto_hide_wm,
        command_timeout,
    );

    // ── --module: single module output for status bars ────────────────────────
    if let Some(target) = &cli.module {
        let target_lc = target.to_lowercase();
        for m in &modules {
            if m.key().to_lowercase() == target_lc {
                let start = Instant::now();
                if let Ok(v) = m.value() {
                    let stdout = io::stdout();
                    let mut out = io::BufWriter::new(stdout.lock());
                    if cli.raw {
                        writeln!(out, "{v}")?;
                    } else {
                        writeln!(out, "{}", theme.format_line(m.key(), &v))?;
                    }
                    out.flush()?;
                }
                if cli.timings {
                    eprintln!("{}: {} ms", m.key(), start.elapsed().as_millis());
                }
                return Ok(());
            }
        }
        eprintln!("raifetch: unknown module '{target}'. Run --list-modules to see available.");
        return Ok(());
    }

    // ── Collection ────────────────────────────────────────────────────────────
    let mut raw = collect_modules(&modules);
    raw.sort_unstable_by_key(|(i, _, _, _)| *i);
    let pairs: Vec<(String, String)> = raw
        .iter()
        .filter_map(|(_, k, v, _)| v.as_ref().map(|val| (k.clone(), val.clone())))
        .collect();
    let timings = if cli.timings {
        let mut t: Vec<(String, Duration, bool)> = raw
            .iter()
            .map(|(_, k, v, d)| (k.clone(), *d, v.is_some()))
            .collect();
        t.sort_by(|a, b| b.1.cmp(&a.1));
        Some(t)
    } else {
        None
    };

    // ── Label alignment: find max key width ───────────────────────────────────
    let key_width = if cfg.theme.align_labels {
        pairs
            .iter()
            .map(|(k, _)| {
                let icon_len = if cfg.theme.icons {
                    theme::icon_for(k).chars().count()
                } else {
                    0
                };
                k.len() + icon_len
            })
            .max()
            .unwrap_or(0)
    } else {
        0
    };

    // ── Logo resolution ───────────────────────────────────────────────────────
    let logo_type = if cli.no_image {
        "none".to_string()
    } else if cli.image.is_some() {
        "image".to_string()
    } else {
        cfg.image.logo_type.clone()
    };

    // ── Render ───────────────────────────────────────────────────────────────
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());
    writeln!(out)?;

    match logo_type.as_str() {
        "none" => {
            print_info_only(&mut out, &pairs, &theme, &cfg, key_width)?;
        }
        "ascii" => {
            let logo = resolve_ascii_logo(&cfg);
            print_ascii_sideby(&mut out, &logo, &pairs, &theme, &cfg, cfg.general.gap, key_width)?;
        }
        "image" | _ /* "auto" */ => {
            let path_opt = resolve_image_path(cli.image.as_deref(), cfg.image.path.as_deref());
            match path_opt {
                Some(img_path) => {
                    render_image_cached(
                        &mut out, &img_path, &cfg, &pairs, &theme, key_width, cli.backend.as_deref(),
                    )?;
                }
                None => {
                    let logo = resolve_ascii_logo(&cfg);
                    print_ascii_sideby(&mut out, &logo, &pairs, &theme, &cfg, cfg.general.gap, key_width)?;
                }
            }
        }
    }

    if cfg.modules.show_colors {
        writeln!(out)?;
        writeln!(out, "{}", color_blocks())?;
    }
    writeln!(out)?;
    out.flush()?;

    if let Some(timings) = timings {
        eprintln!("Module timings (ms):");
        for (name, dur, ok) in timings {
            if ok {
                eprintln!("  {name}: {}", dur.as_millis());
            } else {
                eprintln!("  {name}: {} (error)", dur.as_millis());
            }
        }
    }
    Ok(())
}

// ─── Render helpers ───────────────────────────────────────────────────────────

type ModuleResult = (usize, String, Option<String>, Duration);

fn collect_modules(modules: &[Box<dyn info::InfoModule>]) -> Vec<ModuleResult> {
    let mut raw: Vec<Option<ModuleResult>> = std::iter::repeat_with(|| None)
        .take(modules.len())
        .collect();

    std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for (i, module) in modules.iter().enumerate() {
            if module.run_in_background() {
                handles.push((
                    i,
                    scope.spawn(move || collect_one_module(i, module.as_ref())),
                ));
            } else {
                raw[i] = Some(collect_one_module(i, module.as_ref()));
            }
        }

        for (i, handle) in handles {
            raw[i] = Some(handle.join().unwrap_or_else(|_| {
                (
                    i,
                    modules[i].key().to_string(),
                    None,
                    Duration::from_millis(0),
                )
            }));
        }
    });

    raw.into_iter().flatten().collect()
}

fn collect_one_module(index: usize, module: &dyn info::InfoModule) -> ModuleResult {
    let start = Instant::now();
    let value = module.value().ok();
    let elapsed = start.elapsed();
    (index, module.key().to_string(), value, elapsed)
}

fn build_header(theme: &Theme, cfg: &Config) -> Vec<String> {
    if !cfg.general.show_header {
        return vec![];
    }
    let user = os::username();
    let host = os::hostname();
    vec![
        format!(
            "  {}@{}",
            theme.apply_label(&user),
            theme.apply_label(&host)
        ),
        format!(
            "  {}",
            theme.apply_label(&"─".repeat(user.len() + host.len() + 1))
        ),
    ]
}

fn format_pairs(pairs: &[(String, String)], theme: &Theme, key_width: usize) -> Vec<String> {
    pairs
        .iter()
        .map(|(k, v)| theme.format_line_aligned(k, v, key_width))
        .collect()
}

fn print_info_only(
    out: &mut impl Write,
    pairs: &[(String, String)],
    theme: &Theme,
    cfg: &Config,
    key_width: usize,
) -> anyhow::Result<()> {
    for line in build_header(theme, cfg) {
        writeln!(out, "{line}")?;
    }
    for line in format_pairs(pairs, theme, key_width) {
        writeln!(out, "{line}")?;
    }
    Ok(())
}

fn print_ascii_sideby(
    out: &mut impl Write,
    logo: &ascii::AsciiLogo,
    pairs: &[(String, String)],
    theme: &Theme,
    cfg: &Config,
    gap: usize,
    key_width: usize,
) -> anyhow::Result<()> {
    let mut info = build_header(theme, cfg);
    info.extend(format_pairs(pairs, theme, key_width));

    let pad = " ".repeat(logo.width);
    let gap_s = " ".repeat(gap);
    let total = logo.lines.len().max(info.len());
    for i in 0..total {
        let left = logo
            .lines
            .get(i)
            .map(String::as_str)
            .unwrap_or(pad.as_str());
        let right = info.get(i).cloned().unwrap_or_default();
        writeln!(out, "{left}{gap_s}{right}")?;
    }
    Ok(())
}

fn print_kitty_sideby(
    out: &mut impl Write,
    img_str: &str,
    img_cols: u16,
    img_rows: u16,
    pairs: &[(String, String)],
    theme: &Theme,
    cfg: &Config,
    gap: usize,
    key_width: usize,
) -> anyhow::Result<()> {
    let mut info = build_header(theme, cfg);
    info.extend(format_pairs(pairs, theme, key_width));

    write!(out, "{img_str}")?;
    out.flush()?;
    write!(out, "\x1b[{img_rows}A")?;

    let col = img_cols as usize + gap + 1;
    for (i, line) in info.iter().enumerate() {
        write!(out, "\x1b[{col}G{line}")?;
        if i + 1 < img_rows as usize {
            write!(out, "\x1b[1B")?;
        }
    }
    let remaining = (img_rows as usize).saturating_sub(info.len());
    if remaining > 0 {
        write!(out, "\x1b[{remaining}B")?;
    }
    write!(out, "\x1b[1G")?;
    writeln!(out)?;
    Ok(())
}

/// Render with cache: check cache first → if miss, load PNG → render → save cache.
fn render_image_cached(
    out: &mut impl Write,
    img_path: &std::path::Path,
    cfg: &Config,
    pairs: &[(String, String)],
    theme: &Theme,
    key_width: usize,
    backend_override: Option<&str>,
) -> anyhow::Result<()> {
    let backend = select_backend(backend_override);

    // 1. Try render cache (skips PNG decode on repeated runs)
    let render_result: anyhow::Result<(String, u16, u16)> = if let Some(cached) =
        image::cache::load(img_path, backend.name(), cfg.image.width, cfg.image.height)
    {
        Ok((cached.data, cached.cols, cached.rows))
    } else {
        // 2. Cache miss — decode PNG and render
        let img = ::image::open(img_path)
            .map_err(|e| anyhow::anyhow!("Cannot load image '{}': {e}", img_path.display()))?;
        let result = backend.render(&img, cfg.image.width, cfg.image.height)?;
        // 3. Save to cache for next run
        image::cache::save(
            img_path,
            backend.name(),
            cfg.image.width,
            cfg.image.height,
            &result.0,
            result.1,
            result.2,
        );
        Ok(result)
    };

    match render_result {
        Ok((s, cols, rows)) if backend.name() == "kitty" => print_kitty_sideby(
            out,
            &s,
            cols,
            rows,
            pairs,
            theme,
            cfg,
            cfg.general.gap,
            key_width,
        ),
        Ok((s, cols, rows)) => {
            let lines = render_side_by_side(s, cols, rows, false, pairs, theme, cfg.general.gap);
            for l in &lines {
                writeln!(out, "{l}")?;
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("raifetch: image error: {e}");
            let logo = resolve_ascii_logo(cfg);
            print_ascii_sideby(out, &logo, pairs, theme, cfg, cfg.general.gap, key_width)
        }
    }
}

// ─── Misc helpers ─────────────────────────────────────────────────────────────

fn resolve_ascii_logo(cfg: &Config) -> ascii::AsciiLogo {
    // Custom file takes priority
    if let Some(file) = &cfg.image.ascii_file {
        if let Some(logo) = ascii::load_from_file(file) {
            return logo;
        }
    }
    if cfg.image.ascii_distro == "auto" {
        ascii::auto_logo()
    } else {
        ascii::get_logo(&cfg.image.ascii_distro)
    }
}

/// Resolve the image path from CLI override or config, expanding `~`.
fn resolve_image_path(
    cli_path: Option<&str>,
    cfg_path: Option<&str>,
) -> Option<std::path::PathBuf> {
    let p = cli_path.or(cfg_path).filter(|s| !s.is_empty())?;
    Some(Config::expand_path(p))
}

fn select_backend(name: Option<&str>) -> Box<dyn ImageBackend> {
    match name {
        Some("kitty") => Box::new(KittyBackend::new()),
        Some("sixel") => Box::new(SixelBackend::new()),
        Some("block") => Box::new(BlockBackend::new()),
        _ => auto_detect(),
    }
}

fn do_install() -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let dest = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home dir"))?
        .join(".local/bin/raifetch");
    std::fs::create_dir_all(dest.parent().unwrap())?;
    std::fs::copy(&exe, &dest)?;
    println!("Installed to {}", dest.display());
    println!("Make sure ~/.local/bin is in your PATH.");
    Ok(())
}

fn do_clear_cache() -> anyhow::Result<()> {
    let mut count = 0usize;
    for entry in std::fs::read_dir("/tmp")?.flatten() {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with("raifetch_") {
            if std::fs::remove_file(entry.path()).is_ok() {
                count += 1;
            }
        }
    }
    println!("Cleared {count} cached image render(s) from /tmp.");
    Ok(())
}
