mod cli;
mod detect;
mod optimize;
mod profile;
mod state;
mod tui;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Scan) => {
            let system = detect::scan_system()?;
            println!("{system}");
        }
        Some(Command::Apply { profile, dry_run }) => {
            let system = detect::scan_system()?;
            let optimizations = profile::resolve(&profile, &system);

            if dry_run {
                println!("Dry run -- the following changes would be applied:\n");
                for opt in &optimizations {
                    println!("{opt}");
                }
            } else {
                let snapshot = state::snapshot(&optimizations)?;
                println!("Snapshot saved: {}", snapshot.path.display());
                println!();

                for opt in &optimizations {
                    println!("Applying: {}", opt.name);
                    if let Err(e) = optimize::apply(opt) {
                        eprintln!("  Failed: {e}");
                    } else {
                        println!("  Done");
                    }
                }

                println!("\nAll optimizations applied.");
                println!(
                    "Rollback with: tuxtune rollback {}",
                    snapshot.path.display()
                );
            }
        }
        Some(Command::Rollback { snapshot }) => {
            state::rollback(&snapshot)?;
            println!("Rolled back to snapshot: {}", snapshot.display());
        }
        Some(Command::List) => {
            for (name, desc) in profile::list_all() {
                println!("  {name:<16} {desc}");
            }
        }
        None => {
            tui::run()?;
        }
    }

    Ok(())
}
