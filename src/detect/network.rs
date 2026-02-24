use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub congestion_control: String,
    pub tcp_fastopen: u32,
    pub rmem_max: u64,
    pub wmem_max: u64,
    pub somaxconn: u32,
    pub tw_reuse: bool,
    pub local_port_range: String,
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Network:")?;
        writeln!(f, "  Congestion control: {}", self.congestion_control)?;
        writeln!(f, "  TCP Fast Open: {}", self.tcp_fastopen)?;
        writeln!(
            f,
            "  Buffer max (r/w): {} / {}",
            format_bytes(self.rmem_max),
            format_bytes(self.wmem_max)
        )?;
        writeln!(f, "  somaxconn: {}", self.somaxconn)?;
        write!(f, "  Local port range: {}", self.local_port_range)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{} MB", bytes / 1024 / 1024)
    } else if bytes >= 1024 {
        format!("{} KB", bytes / 1024)
    } else {
        format!("{bytes} B")
    }
}

pub fn detect() -> Result<NetworkInfo> {
    Ok(NetworkInfo {
        congestion_control: super::read_sysctl("net.ipv4.tcp_congestion_control")
            .unwrap_or_else(|_| "unknown".into()),
        tcp_fastopen: super::read_sysctl("net.ipv4.tcp_fastopen")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        rmem_max: super::read_sysctl("net.core.rmem_max")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        wmem_max: super::read_sysctl("net.core.wmem_max")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        somaxconn: super::read_sysctl("net.core.somaxconn")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        tw_reuse: super::read_sysctl("net.ipv4.tcp_tw_reuse")
            .ok()
            .map(|s| s == "1" || s == "2")
            .unwrap_or(false),
        local_port_range: super::read_sysctl("net.ipv4.ip_local_port_range")
            .unwrap_or_else(|_| "unknown".into()),
    })
}
