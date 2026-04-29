/// Bundled ASCII distro logos + file-based custom logos.

pub struct AsciiLogo {
    pub lines: Vec<String>,
    pub width: usize,
}

/// Get a logo by distro name hint (case-insensitive substring match).
pub fn get_logo(distro_hint: &str) -> AsciiLogo {
    let d = distro_hint.to_lowercase();
    if d.contains("endeavour") {
        endeavour()
    } else if d.contains("arch") {
        arch()
    } else if d.contains("nixos") || d.contains("nix") {
        nixos()
    } else if d.contains("fedora") {
        fedora()
    } else if d.contains("manjaro") {
        manjaro()
    } else if d.contains("opensuse") || d.contains("suse") {
        opensuse()
    } else if d.contains("pop") {
        pop()
    } else if d.contains("mint") {
        mint()
    } else if d.contains("ubuntu") {
        ubuntu()
    } else if d.contains("debian") {
        debian()
    } else {
        generic()
    }
}

/// Auto-detect distro from /etc/os-release and return its logo.
pub fn auto_logo() -> AsciiLogo {
    let id = read_os_id();
    get_logo(&id)
}

/// Load a custom ASCII art logo from a file path (supports ~ expansion).
pub fn load_from_file(path: &str) -> Option<AsciiLogo> {
    let expanded = crate::config::Config::expand_path(path);
    let content = std::fs::read_to_string(expanded).ok()?;
    let lines: Vec<String> = content.lines().map(String::from).collect();
    let width = lines.iter().map(|l| visible_len(l)).max().unwrap_or(0);
    Some(AsciiLogo { lines, width })
}

/// Strip ANSI escape codes to count visible characters only.
fn visible_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c == 'm' {
                in_escape = false;
            }
            continue;
        }
        len += 1;
    }
    len
}

fn read_os_id() -> String {
    std::fs::read_to_string("/etc/os-release")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("ID="))
        .and_then(|l| l.splitn(2, '=').nth(1))
        .map(|s| s.trim_matches('"').to_lowercase())
        .unwrap_or_else(|| "linux".to_string())
}

const R: &str = "\x1b[0m"; // reset

// ─── Logos ────────────────────────────────────────────────────────────────────

fn arch() -> AsciiLogo {
    let b = "\x1b[96m";
    AsciiLogo {
        lines: vec![
            format!("{b}        /\\{R}"),
            format!("{b}       /  \\{R}"),
            format!("{b}      / /\\ \\{R}"),
            format!("{b}     / /  \\ \\{R}"),
            format!("{b}    / / /\\ \\ \\{R}"),
            format!("{b}   /_/_/  \\_\\_\\{R}"),
            format!("{b}     Arch Linux{R}"),
        ],
        width: 17,
    }
}

fn endeavour() -> AsciiLogo {
    let p = "\x1b[35m";
    let r2 = "\x1b[31m";
    let w = "\x1b[97m";
    AsciiLogo {
        lines: vec![
            format!("{p}        /\\{R}"),
            format!("{p}       /  \\{R}"),
            format!("{r2}      / {p}\\  \\{R}"),
            format!("{r2}     /  {p}\\  \\{R}"),
            format!("{r2}    / {w}/\\{p}\\  \\{R}"),
            format!("{r2}   / {w}/  \\{p}\\  \\{R}"),
            format!("{r2}  /_/{w}____\\{p}\\__\\{R}"),
            format!("{p}  EndeavourOS{R}"),
        ],
        width: 18,
    }
}

fn ubuntu() -> AsciiLogo {
    let o = "\x1b[33m";
    AsciiLogo {
        lines: vec![
            format!("{o}         .-.{R}"),
            format!("{o}      .-'   '-.{R}"),
            format!("{o}    .' .-''-. '.{R}"),
            format!("{o}   /  /      \\  \\{R}"),
            format!("{o}  |  |  .--.  |  |{R}"),
            format!("{o}  |  |  '--'  |  |{R}"),
            format!("{o}   \\  \\.____./  /{R}"),
            format!("{o}    '-.______.-'{R}"),
            format!("{o}      Ubuntu{R}"),
        ],
        width: 20,
    }
}

fn debian() -> AsciiLogo {
    let r2 = "\x1b[31m";
    AsciiLogo {
        lines: vec![
            format!("{r2}    ____  {R}"),
            format!("{r2}   /    \\ {R}"),
            format!("{r2}  |  ()  |{R}"),
            format!("{r2}  |  __/ {R}"),
            format!("{r2}  | |    {R}"),
            format!("{r2}  | |    {R}"),
            format!("{r2}  |_|    {R}"),
            format!("{r2}  Debian {R}"),
        ],
        width: 10,
    }
}

fn nixos() -> AsciiLogo {
    let b = "\x1b[94m";
    let c = "\x1b[96m";
    AsciiLogo {
        lines: vec![
            format!("{b}  \\\\  / /  {R}"),
            format!("{b}   \\\\/  /   {R}"),
            format!("{c} -- /\\ --  {R}"),
            format!("{c}  / /\\ \\   {R}"),
            format!("{b} / /  \\ \\  {R}"),
            format!("{b}/_/    \\_\\ {R}"),
            format!("{b}  NixOS   {R}"),
        ],
        width: 12,
    }
}

fn fedora() -> AsciiLogo {
    let b = "\x1b[34m";
    let w = "\x1b[97m";
    AsciiLogo {
        lines: vec![
            format!("{b}    ___  {R}"),
            format!("{b}   /   \\ {R}"),
            format!("{b}  | {w}|{b}-- |{R}"),
            format!("{b}  | {w}|{b}  | {R}"),
            format!("{b}  | {w}|__/ {R}"),
            format!("{b}   \\___/ {R}"),
            format!("{b}  Fedora {R}"),
        ],
        width: 9,
    }
}

fn manjaro() -> AsciiLogo {
    let g = "\x1b[92m";
    AsciiLogo {
        lines: vec![
            format!("{g}  |||||||  {R}"),
            format!("{g}  |||||||  {R}"),
            format!("{g}  |||      {R}"),
            format!("{g}  ||| |||  {R}"),
            format!("{g}  ||| |||  {R}"),
            format!("{g}  ||| |||  {R}"),
            format!("{g}  Manjaro  {R}"),
        ],
        width: 11,
    }
}

fn opensuse() -> AsciiLogo {
    let g = "\x1b[32m";
    AsciiLogo {
        lines: vec![
            format!("{g}   _______ {R}"),
            format!("{g}  /       \\{R}"),
            format!("{g} |  .-.   |{R}"),
            format!("{g} |  | |   |{R}"),
            format!("{g}  \\ '-'  / {R}"),
            format!("{g}   \\____/  {R}"),
            format!("{g}  openSUSE {R}"),
        ],
        width: 11,
    }
}

fn pop() -> AsciiLogo {
    let c = "\x1b[96m";
    let y = "\x1b[93m";
    AsciiLogo {
        lines: vec![
            format!("{c}  ______  {R}"),
            format!("{c} /      \\ {R}"),
            format!("{c}|  {y}Pop!{c}  |{R}"),
            format!("{c}|   _OS  |{R}"),
            format!("{c} \\      / {R}"),
            format!("{c}  \\____/  {R}"),
            format!("{c}  Pop!_OS {R}"),
        ],
        width: 10,
    }
}

fn mint() -> AsciiLogo {
    let g = "\x1b[92m";
    let d = "\x1b[32m";
    AsciiLogo {
        lines: vec![
            format!("{d} _________ {R}"),
            format!("{d}|         |{R}"),
            format!("{d}| {g}Mint{d}     |{R}"),
            format!("{d}|   ___   |{R}"),
            format!("{d}|  |   |  |{R}"),
            format!("{d}|  |___|  |{R}"),
            format!("{d}|_________|{R}"),
            format!("{g} Linux Mint{R}"),
        ],
        width: 11,
    }
}

fn generic() -> AsciiLogo {
    let g = "\x1b[92m";
    AsciiLogo {
        lines: vec![
            format!("{g}    .--.   {R}"),
            format!("{g}   |o_o |  {R}"),
            format!("{g}   |:_/ |  {R}"),
            format!("{g}  //   \\ \\ {R}"),
            format!("{g} (|     | ){R}"),
            format!("{g}/'\\_   _/`\\{R}"),
            format!("{g}\\___)=(___/{R}"),
            format!("{g}   Linux   {R}"),
        ],
        width: 12,
    }
}
