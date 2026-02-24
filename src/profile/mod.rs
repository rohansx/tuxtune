use crate::detect::SystemInfo;
use crate::optimize::{self, Optimization};

pub fn list_all() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "developer",
            "Balanced optimization for development workloads (IDE, Docker, builds)",
        ),
        (
            "compile",
            "Maximum compilation performance (parallel builds, ccache, tmpfs)",
        ),
        (
            "gaming",
            "Low-latency optimizations for gaming",
        ),
        (
            "battery",
            "Power-saving optimizations for laptop use",
        ),
        (
            "server",
            "Throughput-oriented server optimizations",
        ),
    ]
}

pub fn resolve(name: &str, system: &SystemInfo) -> Vec<Optimization> {
    match name {
        "developer" => developer_profile(system),
        "compile" => compile_profile(system),
        "gaming" => gaming_profile(system),
        "battery" => battery_profile(system),
        "server" => server_profile(system),
        _ => {
            eprintln!("Unknown profile: {name}, using 'developer'");
            developer_profile(system)
        }
    }
}

fn developer_profile(system: &SystemInfo) -> Vec<Optimization> {
    let mut opts = vec![];

    // Inotify watches for IDEs and file watchers
    opts.push(optimize::sysctl_opt(
        "Inotify watches",
        "Increase inotify limits for IDEs, file watchers, and dev servers",
        "VS Code, JetBrains, webpack, vite, and other tools use inotify to watch files. \
         The default 8192 is far too low for real projects. Each watch costs ~1KB kernel memory.",
        vec![
            ("fs.inotify.max_user_watches", "1048576"),
            ("fs.inotify.max_user_instances", "8192"),
            ("fs.inotify.max_queued_events", "32768"),
        ],
    ));

    // TCP BBR (only if kernel supports it)
    if bbr_available() {
        opts.push(optimize::sysctl_opt(
            "TCP BBR congestion control",
            "Switch to Google's BBR congestion algorithm",
            "BBR uses bandwidth estimation instead of packet loss, giving better throughput \
             and lower latency for git operations, Docker pulls, and npm installs.",
            vec![
                ("net.core.default_qdisc", "fq"),
                ("net.ipv4.tcp_congestion_control", "bbr"),
            ],
        ));
    }

    // TCP buffers
    opts.push(optimize::sysctl_opt(
        "TCP buffer sizes",
        "Increase TCP buffer limits for high-bandwidth transfers",
        "Default max of ~212KB bottlenecks large transfers (git clone, docker pull). \
         16MB allows full utilization of fast connections.",
        vec![
            ("net.core.rmem_max", "16777216"),
            ("net.core.wmem_max", "16777216"),
            ("net.ipv4.tcp_rmem", "4096 131072 16777216"),
            ("net.ipv4.tcp_wmem", "4096 131072 16777216"),
        ],
    ));

    // TCP Fast Open
    opts.push(optimize::sysctl_opt(
        "TCP Fast Open",
        "Enable TCP Fast Open for client and server",
        "Sends data in the SYN packet, saving one round-trip on new connections. \
         Reduces latency for API calls and HTTP requests from dev servers.",
        vec![("net.ipv4.tcp_fastopen", "3")],
    ));

    // Connection handling
    opts.push(optimize::sysctl_opt(
        "Connection handling",
        "Optimize connection backlog and port reuse for dev servers",
        "Higher somaxconn prevents connection drops under load. TIME_WAIT reuse \
         and wider port range help when running many local services and containers.",
        vec![
            ("net.core.somaxconn", "8192"),
            ("net.core.netdev_max_backlog", "8192"),
            ("net.ipv4.tcp_tw_reuse", "1"),
            ("net.ipv4.ip_local_port_range", "1024 65535"),
        ],
    ));

    // VM tuning based on RAM
    if system.memory.total_mb > 16384 {
        opts.push(optimize::sysctl_opt(
            "VM dirty page tuning",
            "Optimize write-back behavior for NVMe",
            "Caps dirty page buffers to prevent large write stalls. \
             Optimal for NVMe drives where small, frequent flushes beat large batches.",
            vec![
                ("vm.dirty_bytes", "268435456"),
                ("vm.dirty_background_bytes", "67108864"),
                ("vm.dirty_writeback_centisecs", "1500"),
            ],
        ));
    }

    opts
}

fn compile_profile(system: &SystemInfo) -> Vec<Optimization> {
    let mut opts = developer_profile(system);

    // More aggressive page cluster for compilation
    opts.push(optimize::sysctl_opt(
        "Page cluster (compilation)",
        "Disable swap readahead for zram",
        "With zram swap, readahead is counterproductive since data is already in RAM. \
         Setting to 0 reduces unnecessary decompression during heavy compilation.",
        vec![("vm.page-cluster", "0")],
    ));

    opts
}

fn gaming_profile(_system: &SystemInfo) -> Vec<Optimization> {
    let mut opts = vec![optimize::sysctl_opt(
        "Gaming network latency",
        "Minimize network latency for gaming",
        "Enables TCP Fast Open for lower-latency network communication in online games.",
        vec![("net.ipv4.tcp_fastopen", "3")],
    )];

    if bbr_available() {
        opts.push(optimize::sysctl_opt(
            "BBR congestion control",
            "Switch to BBR for better gaming network performance",
            "BBR provides lower latency and better throughput than CUBIC.",
            vec![
                ("net.core.default_qdisc", "fq"),
                ("net.ipv4.tcp_congestion_control", "bbr"),
            ],
        ));
    }

    opts
}

fn battery_profile(_system: &SystemInfo) -> Vec<Optimization> {
    vec![optimize::sysctl_opt(
        "Battery-saving VM tuning",
        "Reduce disk writes to save power",
        "Increases dirty page writeback interval, allowing the disk to stay idle longer. \
         Combined with laptop-mode, this significantly reduces power consumption.",
        vec![
            ("vm.dirty_writeback_centisecs", "6000"),
            ("vm.laptop_mode", "5"),
        ],
    )]
}

fn bbr_available() -> bool {
    std::fs::read_to_string("/proc/sys/net/ipv4/tcp_available_congestion_control")
        .map(|s| s.contains("bbr"))
        .unwrap_or(false)
}

fn server_profile(system: &SystemInfo) -> Vec<Optimization> {
    let mut opts = developer_profile(system);

    opts.push(optimize::sysctl_opt(
        "Server connection limits",
        "Increase connection limits for server workloads",
        "Higher file descriptor limits and connection backlog for handling \
         many concurrent connections in server environments.",
        vec![
            ("net.core.somaxconn", "65535"),
            ("net.core.netdev_max_backlog", "65535"),
            ("fs.file-max", "2097152"),
        ],
    ));

    opts
}
