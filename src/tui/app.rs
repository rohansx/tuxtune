use crate::detect::{self, SystemInfo};
use crate::optimize::{self, Optimization};
use crate::profile;
use crate::state;
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    System,
    Optimizations,
    Profiles,
}

impl Tab {
    pub const ALL: [Tab; 3] = [Tab::System, Tab::Optimizations, Tab::Profiles];

    pub fn title(&self) -> &str {
        match self {
            Tab::System => "System",
            Tab::Optimizations => "Optimizations",
            Tab::Profiles => "Profiles",
        }
    }
}

pub struct OptEntry {
    pub optimization: Optimization,
    pub enabled: bool,
}

pub struct App {
    pub system: SystemInfo,
    pub tab: Tab,
    pub tab_index: usize,
    pub selected: usize,
    pub optimizations: Vec<OptEntry>,
    pub profile_index: usize,
    pub profiles: Vec<(String, String)>,
    pub status_message: String,
}

impl App {
    pub fn new() -> Result<Self> {
        let system = detect::scan_system()?;
        let profiles = profile::list_all();
        let opts = profile::resolve(&profiles[0].0, &system);

        let optimizations = opts
            .into_iter()
            .map(|o| OptEntry {
                optimization: o,
                enabled: true,
            })
            .collect();

        Ok(Self {
            system,
            tab: Tab::System,
            tab_index: 0,
            selected: 0,
            optimizations,
            profile_index: 0,
            profiles,
            status_message: "Press 'a' to apply, Space to toggle, Tab to switch views, 'q' to quit"
                .into(),
        })
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % Tab::ALL.len();
        self.tab = Tab::ALL[self.tab_index];
        self.selected = 0;
    }

    pub fn prev_tab(&mut self) {
        self.tab_index = if self.tab_index == 0 {
            Tab::ALL.len() - 1
        } else {
            self.tab_index - 1
        };
        self.tab = Tab::ALL[self.tab_index];
        self.selected = 0;
    }

    pub fn scroll_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        let max = match self.tab {
            Tab::Optimizations => self.optimizations.len().saturating_sub(1),
            Tab::Profiles => self.profiles.len().saturating_sub(1),
            Tab::System => 0,
        };
        if self.selected < max {
            self.selected += 1;
        }
    }

    pub fn toggle_selected(&mut self) {
        if self.tab == Tab::Optimizations && self.selected < self.optimizations.len() {
            self.optimizations[self.selected].enabled = !self.optimizations[self.selected].enabled;
        }
    }

    pub fn apply_selected(&mut self) -> Result<()> {
        let enabled: Vec<Optimization> = self
            .optimizations
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.optimization.clone())
            .collect();

        if enabled.is_empty() {
            self.status_message = "No optimizations selected".into();
            return Ok(());
        }

        match state::snapshot(&enabled) {
            Ok(snap) => {
                let mut applied = 0;
                let mut failed = 0;

                for opt in &enabled {
                    if optimize::apply(opt).is_ok() {
                        applied += 1;
                    } else {
                        failed += 1;
                    }
                }

                self.status_message = format!(
                    "Applied {applied} optimizations ({failed} failed). Snapshot: {}",
                    snap.path.display()
                );

                // Rescan to reflect new state
                self.system = detect::scan_system()?;
            }
            Err(e) => {
                self.status_message = format!("Snapshot failed: {e}");
            }
        }

        Ok(())
    }

    pub fn cycle_profile(&mut self) {
        self.profile_index = (self.profile_index + 1) % self.profiles.len();
        let name = &self.profiles[self.profile_index].0;
        let opts = profile::resolve(name, &self.system);
        self.optimizations = opts
            .into_iter()
            .map(|o| OptEntry {
                optimization: o,
                enabled: true,
            })
            .collect();
        self.selected = 0;
        self.status_message = format!("Profile: {name}");
    }

    pub fn rescan(&mut self) -> Result<()> {
        self.system = detect::scan_system()?;
        self.status_message = "System rescanned".into();
        Ok(())
    }
}
