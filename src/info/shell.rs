use super::{run_command_stdout, InfoModule};
use std::process::Command;
use std::time::Duration;

pub struct ShellModule {
    timeout: Duration,
}

impl ShellModule {
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }
}

impl InfoModule for ShellModule {
    fn key(&self) -> &'static str {
        "Shell"
    }

    fn run_in_background(&self) -> bool {
        true
    }

    fn value(&self) -> anyhow::Result<String> {
        let shell_path = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());

        let name = shell_path
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string();

        if let Some(version) = shell_version_from_env(&name) {
            Ok(version)
        } else {
            Ok(name)
        }
    }
}

fn shell_version_from_env(name: &str) -> Option<String> {
    let version = match name {
        "bash" => std::env::var("BASH_VERSION").ok(),
        "zsh" => std::env::var("ZSH_VERSION").ok(),
        "fish" => std::env::var("FISH_VERSION").ok(),
        _ => None,
    }?;
    if version.is_empty() {
        None
    } else {
        Some(format!("{name} {version}"))
    }
}
