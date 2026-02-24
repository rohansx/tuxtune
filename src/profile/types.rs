use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProfileToml {
    pub profile: ProfileMeta,
    #[serde(default)]
    pub optimization: Vec<OptimizationToml>,
}

#[derive(Debug, Deserialize)]
pub struct ProfileMeta {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub extends: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizationToml {
    pub name: String,
    pub description: String,
    pub explanation: String,
    #[serde(default)]
    pub requires: Vec<String>,
    pub min_ram_mb: Option<u64>,
    #[serde(default)]
    pub sysctl: Vec<SysctlEntry>,
}

#[derive(Debug, Deserialize)]
pub struct SysctlEntry {
    pub key: String,
    pub value: String,
}
