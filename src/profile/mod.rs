mod types;

use crate::detect::SystemInfo;
use crate::optimize::{self, Optimization};
use std::collections::HashSet;
use types::ProfileToml;

// Embedded default profiles — binary works standalone
const DEVELOPER_TOML: &str = include_str!("../../profiles/developer.toml");
const COMPILE_TOML: &str = include_str!("../../profiles/compile.toml");
const GAMING_TOML: &str = include_str!("../../profiles/gaming.toml");
const BATTERY_TOML: &str = include_str!("../../profiles/battery.toml");
const SERVER_TOML: &str = include_str!("../../profiles/server.toml");

fn builtin_profiles() -> Vec<(&'static str, &'static str)> {
    vec![
        ("developer", DEVELOPER_TOML),
        ("compile", COMPILE_TOML),
        ("gaming", GAMING_TOML),
        ("battery", BATTERY_TOML),
        ("server", SERVER_TOML),
    ]
}

fn load_all_profiles() -> Vec<(String, ProfileToml)> {
    let mut profiles = Vec::new();

    for (name, toml_str) in builtin_profiles() {
        match toml::from_str::<ProfileToml>(toml_str) {
            Ok(p) => profiles.push((name.to_string(), p)),
            Err(e) => eprintln!("Failed to parse builtin profile {name}: {e}"),
        }
    }

    // Load user profiles from ~/.config/tuxtune/profiles/*.toml
    if let Ok(home) = std::env::var("HOME") {
        let user_dir = std::path::PathBuf::from(home).join(".config/tuxtune/profiles");
        if user_dir.is_dir()
            && let Ok(entries) = std::fs::read_dir(&user_dir)
        {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => match toml::from_str::<ProfileToml>(&content) {
                            Ok(p) => {
                                let name = p.profile.name.clone();
                                // User profiles override builtins by name
                                profiles.retain(|(n, _)| n != &name);
                                profiles.push((name, p));
                            }
                            Err(e) => {
                                eprintln!("Failed to parse {}: {e}", path.display());
                            }
                        },
                        Err(e) => eprintln!("Failed to read {}: {e}", path.display()),
                    }
                }
            }
        }
    }

    profiles
}

pub fn list_all() -> Vec<(String, String)> {
    load_all_profiles()
        .into_iter()
        .map(|(name, p)| (name, p.profile.description))
        .collect()
}

pub fn resolve(name: &str, system: &SystemInfo) -> Vec<Optimization> {
    let all = load_all_profiles();
    let caps = detect_capabilities();
    let mut opts = resolve_profile(name, system, &all, &caps, &mut HashSet::new());

    // Append hardware-specific optimizations detected from the system
    if let Some(ref gpu) = system.gpu {
        if let Some(nvidia_opt) = optimize::nvidia_suspend_opt(gpu) {
            opts.push(nvidia_opt);
        }
    }

    opts
}

fn resolve_profile(
    name: &str,
    system: &SystemInfo,
    all: &[(String, ProfileToml)],
    caps: &HashSet<String>,
    visited: &mut HashSet<String>,
) -> Vec<Optimization> {
    if !visited.insert(name.to_string()) {
        eprintln!("Circular extends detected for profile: {name}");
        return vec![];
    }

    let profile = match all.iter().find(|(n, _)| n == name) {
        Some((_, p)) => p,
        None => {
            eprintln!("Unknown profile: {name}, using 'developer'");
            match all.iter().find(|(n, _)| n == "developer") {
                Some((_, p)) => p,
                None => return vec![],
            }
        }
    };

    // Resolve parent profiles first
    let mut opts = Vec::new();
    for parent in &profile.profile.extends {
        opts.extend(resolve_profile(parent, system, all, caps, visited));
    }

    // Process this profile's optimizations
    for opt_toml in &profile.optimization {
        // Check capability requirements
        if !opt_toml.requires.iter().all(|r| caps.contains(r)) {
            continue;
        }

        // Check minimum RAM requirement
        if let Some(min_ram) = opt_toml.min_ram_mb
            && system.memory.total_mb < min_ram
        {
            continue;
        }

        let params: Vec<(&str, &str)> = opt_toml
            .sysctl
            .iter()
            .map(|s| (s.key.as_str(), s.value.as_str()))
            .collect();

        opts.push(optimize::sysctl_opt(
            &opt_toml.name,
            &opt_toml.description,
            &opt_toml.explanation,
            params,
        ));
    }

    opts
}

fn detect_capabilities() -> HashSet<String> {
    let mut caps = HashSet::new();

    if std::fs::read_to_string("/proc/sys/net/ipv4/tcp_available_congestion_control")
        .map(|s| s.contains("bbr"))
        .unwrap_or(false)
    {
        caps.insert("bbr".to_string());
    }

    caps
}
