use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_mb: u64,
    pub available_mb: u64,
    pub swap_total_mb: u64,
    pub zram_enabled: bool,
    pub zram_size_mb: u64,
    pub zram_algorithm: String,
    pub swappiness: u32,
    pub vfs_cache_pressure: u32,
    pub dirty_bytes: u64,
    pub dirty_background_bytes: u64,
}

impl fmt::Display for MemoryInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Memory: {} MB total, {} MB available",
            self.total_mb, self.available_mb
        )?;
        writeln!(
            f,
            "  Swap: {} MB (zram: {})",
            self.swap_total_mb,
            if self.zram_enabled {
                format!("{} MB, {}", self.zram_size_mb, self.zram_algorithm)
            } else {
                "disabled".into()
            }
        )?;
        writeln!(f, "  Swappiness: {}", self.swappiness)?;
        write!(f, "  VFS cache pressure: {}", self.vfs_cache_pressure)
    }
}

pub fn detect() -> Result<MemoryInfo> {
    let meminfo = fs::read_to_string("/proc/meminfo")?;

    let total_kb = parse_meminfo_field(&meminfo, "MemTotal").unwrap_or(0);
    let available_kb = parse_meminfo_field(&meminfo, "MemAvailable").unwrap_or(0);
    let swap_total_kb = parse_meminfo_field(&meminfo, "SwapTotal").unwrap_or(0);

    let (zram_enabled, zram_size_mb, zram_algorithm) = detect_zram();

    let swappiness = super::read_sysctl("vm.swappiness")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);

    let vfs_cache_pressure = super::read_sysctl("vm.vfs_cache_pressure")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let dirty_bytes = super::read_sysctl("vm.dirty_bytes")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let dirty_background_bytes = super::read_sysctl("vm.dirty_background_bytes")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Ok(MemoryInfo {
        total_mb: total_kb / 1024,
        available_mb: available_kb / 1024,
        swap_total_mb: swap_total_kb / 1024,
        zram_enabled,
        zram_size_mb,
        zram_algorithm,
        swappiness,
        vfs_cache_pressure,
        dirty_bytes,
        dirty_background_bytes,
    })
}

fn parse_meminfo_field(meminfo: &str, field: &str) -> Option<u64> {
    meminfo
        .lines()
        .find(|l| l.starts_with(field))
        .and_then(|l| {
            l.split_whitespace()
                .nth(1)
                .and_then(|v| v.parse::<u64>().ok())
        })
}

fn detect_zram() -> (bool, u64, String) {
    let disksize = fs::read_to_string("/sys/block/zram0/disksize")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());

    let algorithm = fs::read_to_string("/sys/block/zram0/comp_algorithm")
        .ok()
        .map(|s| {
            // Format: "lzo lzo-rle lz4 lz4hc 842 zstd [lz4]" -- bracketed is active
            s.trim()
                .split_whitespace()
                .find(|w| w.starts_with('['))
                .map(|w| w.trim_matches(|c| c == '[' || c == ']').to_string())
                .unwrap_or_else(|| s.trim().to_string())
        })
        .unwrap_or_else(|| "none".into());

    match disksize {
        Some(size) if size > 0 => (true, size / 1024 / 1024, algorithm),
        _ => (false, 0, "none".into()),
    }
}
