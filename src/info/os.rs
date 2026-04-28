use super::InfoModule;

pub struct OsModule;

impl OsModule {
    pub fn new() -> Self { Self }
}

impl InfoModule for OsModule {
    fn key(&self) -> &'static str { "OS" }

    fn value(&self) -> anyhow::Result<String> {
        let distro = whoami::distro();
        let arch   = std::env::consts::ARCH;
        Ok(format!("{distro} ({arch})"))
    }
}
