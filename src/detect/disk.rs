use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub kind: DiskKind,
    pub filesystem: String,
    pub mount_point: String,
    pub scheduler: String,
    pub mount_options: Vec<String>,
    pub size_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DiskKind {
    Nvme,
    Ssd,
    Hdd,
    Unknown,
}

impl fmt::Display for DiskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskKind::Nvme => write!(f, "NVMe"),
            DiskKind::Ssd => write!(f, "SSD"),
            DiskKind::Hdd => write!(f, "HDD"),
            DiskKind::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for DiskInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Disk: {} ({}, {} GB, {})",
            self.name, self.kind, self.size_gb, self.filesystem
        )?;
        writeln!(f, "  Mount: {} [{}]", self.mount_point, self.scheduler)?;
        write!(f, "  Options: {}", self.mount_options.join(","))
    }
}

pub fn detect() -> Result<Vec<DiskInfo>> {
    let mounts = fs::read_to_string("/proc/mounts")?;
    let mut disks = Vec::new();

    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let device = parts[0];
        let mount = parts[1];
        let fstype = parts[2];
        let options = parts[3];

        // Only care about real block devices
        if !device.starts_with("/dev/") {
            continue;
        }

        // Skip snap, loop, etc
        if device.contains("loop") || device.contains("snap") {
            continue;
        }

        let block_name = extract_block_device(device);
        let kind = detect_disk_kind(&block_name);
        let scheduler = detect_io_scheduler(&block_name);
        let size_gb = detect_disk_size(&block_name);

        disks.push(DiskInfo {
            name: block_name,
            kind,
            filesystem: fstype.to_string(),
            mount_point: mount.to_string(),
            scheduler,
            mount_options: options.split(',').map(String::from).collect(),
            size_gb,
        });
    }

    // Deduplicate by mount point
    disks.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    disks.dedup_by(|a, b| a.mount_point == b.mount_point);

    Ok(disks)
}

fn extract_block_device(device: &str) -> String {
    let dev = device
        .trim_start_matches("/dev/")
        .trim_start_matches("mapper/");

    // nvme0n1p1 -> nvme0n1
    if dev.contains("nvme") {
        if let Some(pos) = dev.rfind('p') {
            if dev[pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                return dev[..pos].to_string();
            }
        }
    }

    // sda1 -> sda, dm-0 stays as is
    dev.trim_end_matches(|c: char| c.is_ascii_digit())
        .to_string()
}

fn detect_disk_kind(block_name: &str) -> DiskKind {
    if block_name.starts_with("nvme") {
        return DiskKind::Nvme;
    }

    let rotational = fs::read_to_string(format!("/sys/block/{block_name}/queue/rotational"))
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok());

    match rotational {
        Some(0) => DiskKind::Ssd,
        Some(1) => DiskKind::Hdd,
        _ => DiskKind::Unknown,
    }
}

fn detect_io_scheduler(block_name: &str) -> String {
    fs::read_to_string(format!("/sys/block/{block_name}/queue/scheduler"))
        .ok()
        .map(|s| {
            s.trim()
                .split_whitespace()
                .find(|w| w.starts_with('['))
                .map(|w| w.trim_matches(|c| c == '[' || c == ']').to_string())
                .unwrap_or_else(|| s.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

fn detect_disk_size(block_name: &str) -> u64 {
    fs::read_to_string(format!("/sys/block/{block_name}/size"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|sectors| sectors * 512 / 1024 / 1024 / 1024)
        .unwrap_or(0)
}
