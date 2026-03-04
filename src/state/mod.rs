use crate::optimize::Optimization;
use anyhow::{Context, Result};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub path: PathBuf,
    pub timestamp: String,
    pub entries: Vec<SnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub target: String,
    pub previous_value: Option<String>,
}

pub fn snapshot(optimizations: &[Optimization]) -> Result<Snapshot> {
    let dir = dirs();
    fs::create_dir_all(&dir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let path = dir.join(format!("snapshot_{timestamp}.json"));

    let entries: Vec<SnapshotEntry> = optimizations
        .iter()
        .flat_map(|opt| {
            opt.changes.iter().map(|change| SnapshotEntry {
                target: change.target.clone(),
                previous_value: change.current_value.clone(),
            })
        })
        .collect();

    let snap = Snapshot {
        path: path.clone(),
        timestamp,
        entries,
    };

    let json = serde_json::to_string_pretty(&snap)?;
    fs::write(&path, json).with_context(|| format!("Failed to write snapshot to {}", path.display()))?;

    Ok(snap)
}

pub fn rollback(path: &PathBuf) -> Result<()> {
    let data = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snapshot from {}", path.display()))?;

    let snap: Snapshot = serde_json::from_str(&data)?;

    for entry in &snap.entries {
        if let Some(ref prev) = entry.previous_value {
            if entry.target.starts_with('/') {
                // Direct file path (e.g., /etc/modprobe.d/nvidia.conf)
                if std::path::Path::new(&entry.target).exists() {
                    fs::write(&entry.target, prev)
                        .with_context(|| format!("Failed to restore {}", entry.target))?;
                    println!("  Restored: {}", entry.target);
                }
            } else {
                // Sysctl key
                let sysctl_path = format!("/proc/sys/{}", entry.target.replace('.', "/"));
                if std::path::Path::new(&sysctl_path).exists() {
                    fs::write(&sysctl_path, prev)
                        .with_context(|| format!("Failed to restore {}", entry.target))?;
                    println!("  Restored: {} = {}", entry.target, prev);
                }
            }
        }
    }

    Ok(())
}

fn dirs() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".local/share/tuxtune/snapshots")
}
