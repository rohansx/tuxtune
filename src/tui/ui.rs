use super::app::{App, Tab};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph, Tabs, Wrap},
};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // tabs
        Constraint::Min(10),  // content
        Constraint::Length(3), // status bar
    ])
    .split(f.area());

    render_tabs(f, app, chunks[0]);

    match app.tab {
        Tab::System => render_system(f, app, chunks[1]),
        Tab::Optimizations => render_optimizations(f, app, chunks[1]),
        Tab::Profiles => render_profiles(f, app, chunks[1]),
    }

    render_status(f, app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<&str> = Tab::ALL.iter().map(|t| t.title()).collect();
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" tuxtune"),
        )
        .select(app.tab_index)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).bold());

    f.render_widget(tabs, area);
}

fn render_system(f: &mut Frame, app: &App, area: Rect) {
    let sys = &app.system;

    let content = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: CPU + Memory
    let left_sections = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(10),
        Constraint::Min(5),
    ])
    .split(content[0]);

    let cpu_text = vec![
        Line::from(format!("  Model: {}", sys.cpu.model)),
        Line::from(format!(
            "  Cores/Threads: {}/{}",
            sys.cpu.cores, sys.cpu.threads
        )),
        Line::from(format!("  Governor: {}", sys.cpu.governor)),
        Line::from(format!("  Scheduler: {}", sys.cpu.scheduler)),
        Line::from(format!(
            "  P-State: {}",
            sys.cpu
                .pstate_driver
                .as_deref()
                .unwrap_or("not available")
        )),
    ];
    let cpu_block = Paragraph::new(cpu_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" CPU ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(cpu_block, left_sections[0]);

    let mem = &sys.memory;
    let mem_text = vec![
        Line::from(format!(
            "  Total: {} MB ({:.1} GB)",
            mem.total_mb,
            mem.total_mb as f64 / 1024.0
        )),
        Line::from(format!("  Available: {} MB", mem.available_mb)),
        Line::from(format!("  Swap: {} MB", mem.swap_total_mb)),
        Line::from(format!(
            "  Zram: {}",
            if mem.zram_enabled {
                format!("{} MB ({})", mem.zram_size_mb, mem.zram_algorithm)
            } else {
                "disabled".into()
            }
        )),
        Line::from(format!("  Swappiness: {}", mem.swappiness)),
        Line::from(format!("  VFS cache pressure: {}", mem.vfs_cache_pressure)),
    ];
    let mem_block = Paragraph::new(mem_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Memory ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(mem_block, left_sections[1]);

    let kern_text = vec![
        Line::from(format!("  Kernel: {}", sys.kernel)),
        Line::from(format!("  Hostname: {}", sys.hostname)),
    ];
    let kern_block = Paragraph::new(kern_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" System ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(kern_block, left_sections[2]);

    // Right: Disks + Network
    let right_sections =
        Layout::vertical([Constraint::Min(8), Constraint::Length(10)]).split(content[1]);

    let mut disk_lines = vec![];
    for disk in &sys.disks {
        disk_lines.push(Line::from(
            format!(
                "  {} ({}, {} GB) -> {}",
                disk.name, disk.kind, disk.size_gb, disk.mount_point
            )
            .bold(),
        ));
        disk_lines.push(Line::from(format!(
            "    FS: {}  Sched: {}",
            disk.filesystem, disk.scheduler
        )));
        let opts_str = disk.mount_options.join(",");
        if opts_str.len() > 60 {
            disk_lines.push(Line::from(format!("    Opts: {}...", &opts_str[..57])));
        } else {
            disk_lines.push(Line::from(format!("    Opts: {opts_str}")));
        }
        disk_lines.push(Line::from(""));
    }
    let disk_block = Paragraph::new(disk_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Disks ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(disk_block, right_sections[0]);

    let net = &sys.network;
    let net_text = vec![
        Line::from(format!("  Congestion: {}", net.congestion_control)),
        Line::from(format!("  TCP Fast Open: {}", net.tcp_fastopen)),
        Line::from(format!(
            "  Buffers (r/w): {} / {}",
            format_bytes(net.rmem_max),
            format_bytes(net.wmem_max)
        )),
        Line::from(format!("  somaxconn: {}", net.somaxconn)),
        Line::from(format!(
            "  TIME_WAIT reuse: {}",
            if net.tw_reuse { "yes" } else { "no" }
        )),
        Line::from(format!("  Port range: {}", net.local_port_range)),
    ];
    let net_block = Paragraph::new(net_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Network ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(net_block, right_sections[1]);
}

fn render_optimizations(f: &mut Frame, app: &App, area: Rect) {
    let chunks =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).split(area);

    // Left: list
    let items: Vec<ListItem> = app
        .optimizations
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let check = if entry.enabled { "[x]" } else { "[ ]" };
            let style = if i == app.selected {
                Style::default().fg(Color::Cyan).bold()
            } else if entry.enabled {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            ListItem::new(format!(" {check} {}", entry.optimization.name)).style(style)
        })
        .collect();

    let profile_name = app.profiles[app.profile_index].0;
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Optimizations [{profile_name}] (p: cycle) "))
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(list, chunks[0]);

    // Right: detail of selected
    if let Some(entry) = app.optimizations.get(app.selected) {
        let opt = &entry.optimization;
        let mut lines = vec![
            Line::from(opt.name.clone()).bold(),
            Line::from(""),
            Line::from(opt.description.clone()),
            Line::from(""),
            Line::from("Why:").bold(),
            Line::from(opt.explanation.clone()),
            Line::from(""),
            Line::from("Changes:").bold(),
        ];

        for change in &opt.changes {
            match &change.current_value {
                Some(current) if current != &change.new_value => {
                    lines.push(Line::from(format!(
                        "  {} : {} -> {}",
                        change.target, current, change.new_value
                    )));
                }
                Some(current) => {
                    lines.push(
                        Line::from(format!("  {} = {} (already set)", change.target, current))
                            .style(Style::default().fg(Color::DarkGray)),
                    );
                }
                None => {
                    lines.push(Line::from(format!(
                        "  {} = {}",
                        change.target, change.new_value
                    )));
                }
            }
        }

        let detail = Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Detail ")
                    .padding(Padding::horizontal(1)),
            );
        f.render_widget(detail, chunks[1]);
    }
}

fn render_profiles(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .profiles
        .iter()
        .enumerate()
        .map(|(i, (name, desc))| {
            let marker = if i == app.profile_index { ">" } else { " " };
            let style = if i == app.selected {
                Style::default().fg(Color::Cyan).bold()
            } else if i == app.profile_index {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            ListItem::new(format!(" {marker} {name:<16} {desc}")).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Profiles (Enter to select) ")
            .padding(Padding::horizontal(1)),
    );
    f.render_widget(list, area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let status = Paragraph::new(format!(" {}", app.status_message))
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(status, area);
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
