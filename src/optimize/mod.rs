use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;

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
            ChangeKind::EnableService | ChangeKind::InstallPackage => {
                // These require elevated privileges, handled by the TUI/CLI wrapper
                eprintln!(
                    "    Note: {} requires elevated privileges (skipped in direct mode)",
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
