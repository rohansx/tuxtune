use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vendor: GpuVendor,
    pub model: String,
    pub driver_version: String,
    pub preserve_vram: bool,
    pub suspend_services_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Other(String),
}

impl fmt::Display for GpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "GPU: {}", self.model)?;
        writeln!(f, "  Driver: {}", self.driver_version)?;
        if self.vendor == GpuVendor::Nvidia {
            writeln!(
                f,
                "  VRAM preservation: {}",
                if self.preserve_vram {
                    "enabled"
                } else {
                    "DISABLED"
                }
            )?;
            write!(
                f,
                "  Suspend services: {}",
                if self.suspend_services_enabled {
                    "enabled"
                } else {
                    "DISABLED"
                }
            )?;
        }
        Ok(())
    }
}

pub fn detect() -> Option<GpuInfo> {
    detect_nvidia()
}

fn detect_nvidia() -> Option<GpuInfo> {
    let version = fs::read_to_string("/proc/driver/nvidia/version").ok()?;
    let driver_version = version
        .lines()
        .find(|l| l.contains("Kernel Module"))
        .and_then(|l| {
            // Format: "NVRM version: NVIDIA UNIX Open Kernel Module ... 590.48.01 ..."
            l.split_whitespace()
                .find(|w| w.chars().next().is_some_and(|c| c.is_ascii_digit()) && w.contains('.'))
        })
        .unwrap_or("unknown")
        .to_string();

    let model = detect_nvidia_model().unwrap_or_else(|| "NVIDIA GPU".into());

    let preserve_vram = fs::read_to_string("/proc/driver/nvidia/params")
        .map(|s| s.contains("PreserveVideoMemoryAllocations: 1"))
        .unwrap_or(false);

    let suspend_services_enabled = check_service_enabled("nvidia-suspend")
        && check_service_enabled("nvidia-resume")
        && check_service_enabled("nvidia-hibernate");

    Some(GpuInfo {
        vendor: GpuVendor::Nvidia,
        model,
        driver_version,
        preserve_vram,
        suspend_services_enabled,
    })
}

fn detect_nvidia_model() -> Option<String> {
    let gpus_dir = fs::read_dir("/proc/driver/nvidia/gpus/").ok()?;
    for entry in gpus_dir.flatten() {
        let info_path = entry.path().join("information");
        if let Ok(info) = fs::read_to_string(&info_path) {
            if let Some(line) = info.lines().find(|l| l.starts_with("Model:")) {
                return Some(line.split(':').nth(1)?.trim().to_string());
            }
        }
    }
    None
}

fn check_service_enabled(name: &str) -> bool {
    Command::new("systemctl")
        .args(["is-enabled", name])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
        .unwrap_or(false)
}
