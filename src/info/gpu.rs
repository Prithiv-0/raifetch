use super::InfoModule;

pub struct GpuModule;

impl GpuModule {
    pub fn new() -> Self { Self }
}

impl InfoModule for GpuModule {
    fn key(&self) -> &'static str { "GPU" }

    fn value(&self) -> anyhow::Result<String> {
        let gpus = get_gpus();
        if gpus.is_empty() {
            return Ok("Unknown".to_string());
        }
        Ok(gpus.join(", "))
    }
}

#[cfg(target_os = "linux")]
fn get_gpus() -> Vec<String> {
    let mut gpus = Vec::new();
    if let Ok(out) = std::process::Command::new("lspci").arg("-mm").output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("vga compatible controller") || lower.contains("3d controller") || lower.contains("display controller") {
                let mut parts = Vec::new();
                let mut current = String::new();
                let mut in_quote = false;
                for c in line.chars() {
                    if c == '"' {
                        in_quote = !in_quote;
                    } else if c == ' ' && !in_quote {
                        if !current.is_empty() {
                            parts.push(current.clone());
                            current.clear();
                        }
                    } else {
                        current.push(c);
                    }
                }
                if !current.is_empty() { parts.push(current); }

                if parts.len() >= 4 {
                    let mut vendor = parts[2]
                        .replace(" Corporation", "")
                        .replace(" Inc.", "")
                        .replace(" Micro Devices", "")
                        .replace(" Technology", "");
                    
                    // Handle "Advanced Micro Devices, Inc. [AMD/ATI]"
                    if vendor.contains("Advanced Micro Devices") {
                        vendor = "AMD".to_string();
                    }

                    // Extract exact name from brackets if present, e.g. "Alder Lake-S [UHD Graphics]" -> "UHD Graphics"
                    let mut model = parts[3].clone();
                    if let Some(start) = model.find('[') {
                        if let Some(end) = model.rfind(']') {
                            if end > start {
                                model = model[start+1..end].to_string();
                            }
                        }
                    }
                    
                    // Further clean model name
                    model = model.replace(" Series", "");

                    gpus.push(format!("{} {}", vendor, model));
                }
            }
        }
    }
    gpus
}

#[cfg(target_os = "macos")]
fn get_gpus() -> Vec<String> {
    let mut gpus = Vec::new();
    if let Ok(out) = std::process::Command::new("system_profiler").args(["SPDisplaysDataType"]).output() {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("Chipset Model:") {
                gpus.push(rest.trim().to_string());
            }
        }
    }
    gpus
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
fn get_gpus() -> Vec<String> {
    Vec::new()
}
