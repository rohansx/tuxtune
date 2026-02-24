use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tuxtune",
    about = "A developer-focused Linux system tuner",
    long_about = "Detect hardware, apply optimizations, and tune your Linux system \
                  for development workloads. Run without arguments for interactive TUI."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Scan system hardware and current configuration
    Scan,

    /// Apply an optimization profile
    Apply {
        /// Profile name (developer, compile, gaming, battery, server)
        #[arg(default_value = "developer")]
        profile: String,

        /// Show what would change without applying
        #[arg(long)]
        dry_run: bool,
    },

    /// Rollback to a previous snapshot
    Rollback {
        /// Path to the snapshot file
        snapshot: PathBuf,
    },

    /// List available profiles
    List,
}
