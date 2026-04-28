use std::process::Command;
use super::InfoModule;

pub struct ShellModule;

impl ShellModule {
    pub fn new() -> Self { Self }
}

impl InfoModule for ShellModule {
    fn key(&self) -> &'static str { "Shell" }

    fn value(&self) -> anyhow::Result<String> {
        let shell_path = std::env::var("SHELL")
            .unwrap_or_else(|_| "unknown".to_string());

        let name = shell_path
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();

        // Get version: first line of `shell --version`
        let version = Command::new(&name)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.lines().next().unwrap_or("").trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unknown version".to_string());

        // Most shells (zsh, bash) include their name in --version output already.
        // Avoid "zsh (zsh 5.9 ...)" by only prepending name when it's absent.
        if version.to_lowercase().starts_with(name.to_lowercase().as_str()) {
            Ok(version)
        } else {
            Ok(format!("{name} {version}"))
        }
    }
}
