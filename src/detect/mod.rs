mod cpu;
mod disk;
pub mod gpu;
mod memory;
mod network;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub cpu: cpu::CpuInfo,
    pub memory: memory::MemoryInfo,
    pub disks: Vec<disk::DiskInfo>,
    pub network: network::NetworkInfo,
    pub gpu: Option<gpu::GpuInfo>,
    pub kernel: String,
    pub hostname: String,
}

impl fmt::Display for SystemInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "System: {} (kernel {})", self.hostname, self.kernel)?;
        writeln!(f)?;
        writeln!(f, "{}", self.cpu)?;
        writeln!(f, "{}", self.memory)?;
        if let Some(ref gpu) = self.gpu {
            writeln!(f, "{gpu}")?;
        }
        for disk in &self.disks {
            writeln!(f, "{disk}")?;
        }
        write!(f, "{}", self.network)
    }
}

pub fn scan_system() -> Result<SystemInfo> {
    Ok(SystemInfo {
        cpu: cpu::detect()?,
        memory: memory::detect()?,
        disks: disk::detect()?,
        network: network::detect()?,
        gpu: gpu::detect(),
        kernel: read_file_trimmed("/proc/version")
            .map(|v| v.split_whitespace().nth(2).unwrap_or("unknown").to_string())
            .unwrap_or_else(|_| "unknown".into()),
        hostname: read_file_trimmed("/etc/hostname").unwrap_or_else(|_| "unknown".into()),
    })
}

pub fn read_file_trimmed(path: &str) -> Result<String> {
    Ok(std::fs::read_to_string(path)?.trim().to_string())
}

pub fn read_sysctl(key: &str) -> Result<String> {
    let path = format!("/proc/sys/{}", key.replace('.', "/"));
    read_file_trimmed(&path)
}
