# tuxtune

A developer-focused Linux system tuner with hardware detection, optimization profiles, and an interactive TUI — built in Rust.

Most Linux tuning tools focus on one narrow domain (CPU frequency, process priority, or power management). **tuxtune** combines system detection, sysctl tuning, network optimization, memory management, and I/O configuration into a single binary with developer-specific profiles.

## Why

- Your IDE keeps hitting inotify watch limits
- `git clone` and `docker pull` are slower than they should be
- You know there are sysctl tweaks that would help but don't want to research each one
- You want to apply optimizations safely with the ability to roll back

tuxtune detects your hardware, recommends optimizations for your workload, explains every change, and lets you undo everything with a single command.

## Features

- **Hardware detection** — reads CPU, memory, disk type (NVMe/SSD/HDD), network stack, and kernel config
- **Optimization profiles** — developer, compile, gaming, battery, server
- **Interactive TUI** — browse system state, toggle individual optimizations, switch profiles
- **Dry-run mode** — preview every change before applying
- **Snapshot & rollback** — automatic state capture before changes, one-command restore
- **Explains everything** — every optimization tells you *what* it changes and *why*
- **Hardware-aware** — adjusts recommendations based on your RAM, disk type, CPU vendor, and available kernel features

## Quick Start

```bash
# Clone and build
git clone https://github.com/rohansx/tuxtune.git
cd tuxtune
cargo build --release

# Launch the interactive TUI
./target/release/tuxtune

# Or use the CLI directly
./target/release/tuxtune scan              # detect your hardware
./target/release/tuxtune list              # show available profiles
./target/release/tuxtune apply developer --dry-run  # preview changes
sudo ./target/release/tuxtune apply developer       # apply optimizations
```

## Installation

### From source (any distro)

```bash
git clone https://github.com/rohansx/tuxtune.git
cd tuxtune
cargo build --release
sudo cp target/release/tuxtune /usr/local/bin/
```

### AUR (Arch Linux)

Coming soon.

## Usage

### Interactive TUI

Run `tuxtune` without arguments to launch the TUI:

```
tuxtune
```

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between tabs (System, Optimizations, Profiles) |
| `j` / `k` or arrows | Navigate up/down |
| `Space` | Toggle an optimization on/off |
| `p` | Cycle through profiles |
| `a` | Apply selected optimizations |
| `s` | Rescan system |
| `q` | Quit |

### CLI Commands

```bash
# Scan your system
tuxtune scan
```
```
System: omarchy (kernel 6.19.0-2-cachyos)

CPU: AMD Ryzen 7 7435HS
  Cores/Threads: 8/16
  Governor: powersave
  Scheduler: BORE (CachyOS)
Memory: 23706 MB total, 9716 MB available
  Swap: 12287 MB (zram: 4096 MB, zstd)
  Swappiness: 80
Network:
  Congestion control: cubic
  TCP Fast Open: 3
  Buffer max (r/w): 16 MB / 16 MB
```

```bash
# Preview what a profile would change (safe, no modifications)
tuxtune apply developer --dry-run
```
```
[Sysctl] Inotify watches
  Increase inotify limits for IDEs, file watchers, and dev servers
  Why: VS Code, JetBrains, webpack, vite, and other tools use inotify
       to watch files. The default 8192 is far too low for real projects.
    fs.inotify.max_user_watches : 8192 -> 1048576
    fs.inotify.max_user_instances : 128 -> 8192

[Sysctl] TCP buffer sizes
  Increase TCP buffer limits for high-bandwidth transfers
  Why: Default max of ~212KB bottlenecks large transfers (git clone,
       docker pull). 16MB allows full utilization of fast connections.
    net.core.rmem_max : 212992 -> 16777216
    net.core.wmem_max : 212992 -> 16777216
```

```bash
# Apply optimizations (creates a snapshot first)
sudo tuxtune apply developer
# Snapshot saved: ~/.local/share/tuxtune/snapshots/snapshot_20260224_143052.json
# Applying: Inotify watches ... Done
# Applying: TCP buffer sizes ... Done
# ...

# Roll back if needed
sudo tuxtune rollback ~/.local/share/tuxtune/snapshots/snapshot_20260224_143052.json
```

## Profiles

| Profile | Description |
|---------|-------------|
| `developer` | Balanced optimization for development workloads (IDE, Docker, builds) |
| `compile` | Maximum compilation performance (parallel builds, zram tuning) |
| `gaming` | Low-latency optimizations for gaming |
| `battery` | Power-saving optimizations for laptop use |
| `server` | Throughput-oriented server optimizations |

## What It Tunes

| Category | Examples |
|----------|---------|
| **Inotify** | Watch limits for IDEs and file watchers (1M watches) |
| **Network** | TCP buffer sizes, Fast Open, connection backlog, port range, BBR (when available) |
| **Memory** | Dirty page writeback, swap readahead, page cluster for zram |
| **VM** | Cache pressure, dirty bytes tuning based on RAM size |
| **I/O** | Scheduler detection per disk type (NVMe/SSD/HDD) |

Every optimization includes an explanation of what it does and why — visible in both the TUI detail panel and `--dry-run` output.

## Architecture

```
src/
├── main.rs           # CLI entry point
├── cli.rs            # Argument parsing (clap)
├── detect/           # Hardware detection
│   ├── cpu.rs        # CPU model, cores, governor, scheduler, pstate
│   ├── memory.rs     # RAM, swap, zram, swappiness, cache pressure
│   ├── disk.rs       # Block devices, filesystem, mount options, I/O scheduler
│   └── network.rs    # TCP stack configuration
├── optimize/         # Optimization engine
│   └── mod.rs        # Change types, apply logic, builder helpers
├── profile/          # Workload profiles
│   └── mod.rs        # developer, compile, gaming, battery, server
├── state/            # Snapshot & rollback
│   └── mod.rs        # JSON-based state capture and restore
└── tui/              # Interactive terminal UI
    ├── app.rs        # Application state and input handling
    ├── ui.rs         # Rendering (ratatui)
    └── mod.rs        # Terminal setup and event loop
```

## Roadmap

- [ ] Filesystem optimization (fstab detection and recommendations)
- [ ] Package detection (suggest missing dev tools like ccache, mold)
- [ ] Service management (enable/disable services)
- [ ] GPU detection (NVIDIA/AMD driver params)
- [ ] Export/import optimization configs
- [ ] AUR and crates.io publishing

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

## License

MIT
