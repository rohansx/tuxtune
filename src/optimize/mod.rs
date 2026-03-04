use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::process::Command;

use crate::detect::gpu::{GpuInfo, GpuVendor};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Optimization {
    pub name: String,
    pub category: Category,
    pub description: String,
    pub explanation: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    Sysctl,
    IoScheduler,
    Filesystem,
    Packages,
    Service,
    Gpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub kind: ChangeKind,
    pub target: String,
    pub current_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeKind {
    SysctlSet,
    WriteFile,
    EnableService,
    InstallPackage,
}

impl fmt::Display for Optimization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "[{:?}] {}", self.category, self.name)?;
        writeln!(f, "  {}", self.description)?;
        writeln!(f, "  Why: {}", self.explanation)?;
        for change in &self.changes {
            match &change.current_value {
                Some(current) => {
                    writeln!(
                        f,
                        "    {} : {} -> {}",
                        change.target, current, change.new_value
                    )?;
                }
                None => {
                    writeln!(f, "    {} = {}", change.target, change.new_value)?;
                }
            }
        }
        Ok(())
    }
}

pub fn apply(opt: &Optimization) -> Result<()> {
    for change in &opt.changes {
        match change.kind {
            ChangeKind::SysctlSet => {
                let path = format!("/proc/sys/{}", change.target.replace('.', "/"));
                fs::write(&path, &change.new_value)
                    .with_context(|| format!("Failed to write sysctl {}", change.target))?;
            }
            ChangeKind::WriteFile => {
                fs::write(&change.target, &change.new_value)
                    .with_context(|| format!("Failed to write {}", change.target))?;
            }
            ChangeKind::EnableService => {
                Command::new("systemctl")
                    .args(["enable", &change.target])
                    .output()
                    .with_context(|| format!("Failed to enable service {}", change.target))?;
            }
            ChangeKind::InstallPackage => {
                eprintln!(
                    "    Note: {} requires manual installation (skipped)",
                    change.target
                );
            }
        }
    }
    Ok(())
}

// Builder helpers for creating optimizations

pub fn sysctl_opt(
    name: &str,
    description: &str,
    explanation: &str,
    params: Vec<(&str, &str)>,
) -> Optimization {
    let changes = params
        .into_iter()
        .map(|(key, value)| {
            let current = crate::detect::read_sysctl(key).ok();
            Change {
                kind: ChangeKind::SysctlSet,
                target: key.to_string(),
                current_value: current,
                new_value: value.to_string(),
            }
        })
        .collect();

    Optimization {
        name: name.to_string(),
        category: Category::Sysctl,
        description: description.to_string(),
        explanation: explanation.to_string(),
        changes,
    }
}

const NVIDIA_MODPROBE_CONTENT: &str = "\
options nvidia_drm modeset=1
options nvidia NVreg_PreserveVideoMemoryAllocations=1
options nvidia NVreg_TemporaryFilePath=/var/tmp
";

const NVIDIA_MODPROBE_PATH: &str = "/etc/modprobe.d/nvidia.conf";

pub fn nvidia_suspend_opt(gpu: &GpuInfo) -> Option<Optimization> {
    if gpu.vendor != GpuVendor::Nvidia {
        return None;
    }

    // Skip if already fully configured
    if gpu.preserve_vram && gpu.suspend_services_enabled {
        return None;
    }

    let mut changes = Vec::new();

    if !gpu.preserve_vram {
        let current = fs::read_to_string(NVIDIA_MODPROBE_PATH).ok();
        changes.push(Change {
            kind: ChangeKind::WriteFile,
            target: NVIDIA_MODPROBE_PATH.to_string(),
            current_value: current,
            new_value: NVIDIA_MODPROBE_CONTENT.to_string(),
        });
    }

    if !gpu.suspend_services_enabled {
        for service in ["nvidia-suspend", "nvidia-resume", "nvidia-hibernate"] {
            let enabled = Command::new("systemctl")
                .args(["is-enabled", service])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "enabled")
                .unwrap_or(false);

            if !enabled {
                changes.push(Change {
                    kind: ChangeKind::EnableService,
                    target: service.to_string(),
                    current_value: Some("disabled".to_string()),
                    new_value: "enabled".to_string(),
                });
            }
        }
    }

    if changes.is_empty() {
        return None;
    }

    Some(Optimization {
        name: "NVIDIA GPU suspend fix".to_string(),
        category: Category::Gpu,
        description: "Fix NVIDIA GPU freeze on suspend/resume by preserving VRAM and enabling suspend services".to_string(),
        explanation: "NVIDIA GPUs lose VRAM contents during Linux suspend, causing a frozen display on wake. \
            NVreg_PreserveVideoMemoryAllocations=1 saves VRAM to disk before sleep. \
            The nvidia-suspend/resume/hibernate systemd services coordinate the save/restore cycle. \
            Requires initramfs rebuild (mkinitcpio -P) and reboot after applying.".to_string(),
        changes,
    })
}
