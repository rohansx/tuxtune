use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: usize,
    pub threads: usize,
    pub vendor: CpuVendor,
    pub governor: String,
    pub pstate_driver: Option<String>,
    pub scheduler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CpuVendor {
    Amd,
    Intel,
    Other(String),
}

impl fmt::Display for CpuInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CPU: {}", self.model)?;
        writeln!(f, "  Cores/Threads: {}/{}", self.cores, self.threads)?;
        writeln!(f, "  Governor: {}", self.governor)?;
        if let Some(ref driver) = self.pstate_driver {
            writeln!(f, "  P-State driver: {driver}")?;
        }
        write!(f, "  Scheduler: {}", self.scheduler)
    }
}

pub fn detect() -> Result<CpuInfo> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo")?;

    let model = cpuinfo
        .lines()
        .find(|l| l.starts_with("model name"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "Unknown".into());

    let vendor_str = cpuinfo
        .lines()
        .find(|l| l.starts_with("vendor_id"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let vendor = match vendor_str.as_str() {
        "AuthenticAMD" => CpuVendor::Amd,
        "GenuineIntel" => CpuVendor::Intel,
        other => CpuVendor::Other(other.to_string()),
    };

    let threads = cpuinfo
        .lines()
        .filter(|l| l.starts_with("processor"))
        .count();

    let cores = cpuinfo
        .lines()
        .find(|l| l.starts_with("cpu cores"))
        .and_then(|l| l.split(':').nth(1))
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(threads);

    let governor = fs::read_to_string("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "unknown".into());

    let pstate_driver = fs::read_to_string("/sys/devices/system/cpu/amd_pstate/status")
        .or_else(|_| fs::read_to_string("/sys/devices/system/cpu/intel_pstate/status"))
        .map(|s| s.trim().to_string())
        .ok();

    let scheduler = detect_scheduler();

    Ok(CpuInfo {
        model,
        cores,
        threads,
        vendor,
        governor,
        pstate_driver,
        scheduler,
    })
}

fn detect_scheduler() -> String {
    // Check sched_ext first
    if let Ok(s) = fs::read_to_string("/sys/kernel/sched_ext/root/ops") {
        let s = s.trim();
        if !s.is_empty() {
            return format!("sched_ext ({s})");
        }
    }

    // Check kernel config for BORE/EEVDF
    if let Ok(version) = fs::read_to_string("/proc/version") {
        if version.contains("bore") || version.contains("BORE") {
            return "BORE".into();
        }
        if version.contains("cachyos") || version.contains("CachyOS") {
            return "BORE (CachyOS)".into();
        }
    }

    "CFS/EEVDF".into()
}
