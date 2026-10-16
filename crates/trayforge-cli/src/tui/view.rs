//! Rendering of the dashboard.
//!
//! Pure given a [`Dashboard`] and the [`AppConfig`]: it reads state and paints
//! it. No side effects happen here.

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, List, ListItem, ListState, Padding, Paragraph, Wrap},
};

use trayforge::config::AppConfig;

use super::dashboard::{Dashboard, Level};
use trayforge::supervisor::TrayState;

/// Colour palette, kept in one place so the look stays coherent.
mod palette {
    use ratatui::style::Color;

    /// Primary accent (periwinkle).
    pub const ACCENT: Color = Color::Rgb(0x7A, 0xA2, 0xF7);
    /// Background of the highlighted command row.
    pub const SELECTION: Color = Color::Rgb(0x2F, 0x81, 0xF7);
    /// De-emphasised text.
    pub const MUTED: Color = Color::Rgb(0x6C, 0x70, 0x86);
    /// Success indicator.
    pub const SUCCESS: Color = Color::Rgb(0x9E, 0xCE, 0x6A);
    /// Warning indicator.
    pub const WARN: Color = Color::Rgb(0xE0, 0xAF, 0x68);
    /// Failure indicator.
    pub const ERROR: Color = Color::Rgb(0xF7, 0x76, 0x8E);
    /// Body text.
    pub const TEXT: Color = Color::Rgb(0xD5, 0xD8, 0xE0);
}

/// Draw the whole dashboard into `frame`.
pub fn render(frame: &mut Frame, dashboard: &Dashboard, config: &AppConfig) {
    let chunks = Layout::vertical([
        Constraint::Length(3), // banner
        Constraint::Min(12),   // commands + status
        Constraint::Length(8), // activity log
        Constraint::Length(1), // footer
    ])
    .split(frame.area());

    render_banner(frame, chunks[0]);
    render_body(frame, chunks[1], dashboard, config);
    render_activity(frame, chunks[2], dashboard);
    render_footer(frame, chunks[3], dashboard);
}

/// The rounded banner at the top.
fn render_banner(frame: &mut Frame, area: Rect) {
    let block = panel().title(title_line(" trayforge "));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let line = Line::from(vec![
        Span::styled(
            "a terminal control panel for the tray agent",
            Style::new().fg(palette::TEXT),
        ),
        Span::styled(
            format!("   ·   v{}", env!("CARGO_PKG_VERSION")),
            Style::new().fg(palette::MUTED),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), inner);
}

/// The two-column middle: commands on the left, status on the right.
fn render_body(frame: &mut Frame, area: Rect, dashboard: &Dashboard, config: &AppConfig) {
    let columns =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    render_commands(frame, columns[0], dashboard);
    render_status(frame, columns[1], dashboard, config);
}

/// The selectable command list.
fn render_commands(frame: &mut Frame, area: Rect, dashboard: &Dashboard) {
    let items: Vec<ListItem> = super::command::COMMANDS
        .iter()
        .map(|command| {
            let title = Line::from(vec![
                Span::styled(
                    format!("{}  ", command.glyph),
                    Style::new().fg(palette::ACCENT),
                ),
                Span::styled(command.title, Style::new().fg(palette::TEXT).bold()),
            ]);
            let summary = Line::from(Span::styled(
                format!("   {}", command.summary),
                Style::new().fg(palette::MUTED),
            ));
            ListItem::new(vec![title, summary])
        })
        .collect();

    let list = List::new(items)
        .block(panel().title(title_line(" Commands ")))
        .highlight_symbol("▸ ")
        .highlight_style(
            Style::new()
                .bg(palette::SELECTION)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .repeat_highlight_symbol(false);

    let mut state = ListState::default();
    state.select(Some(dashboard.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

/// The key/value status panel.
fn render_status(frame: &mut Frame, area: Rect, dashboard: &Dashboard, config: &AppConfig) {
    let snapshot = &dashboard.snapshot;

    let auto_start = match snapshot.auto_start {
        Some(true) => (true, "enabled on login".to_string()),
        Some(false) => (false, "disabled".to_string()),
        None => (false, "unknown".to_string()),
    };
    let agent = match snapshot.tray {
        TrayState::Running { pid } => (true, format!("running · pid {pid}")),
        TrayState::Stopped => (false, "stopped".to_string()),
    };
    let primary = match snapshot.instance_available {
        Some(true) => (true, "available".to_string()),
        Some(false) => (false, "in use".to_string()),
        None => (false, "unknown".to_string()),
    };

    // Keep paths on one line by eliding their (redundant) prefix.
    let width = area.width.saturating_sub(4) as usize;

    let lines = vec![
        key_value("Auto-start", auto_start.0, &auto_start.1),
        key_value("Tray agent", agent.0, &agent.1),
        key_value("Instance lock", primary.0, &primary.1),
        Line::default(),
        Span::styled("Executable", Style::new().fg(palette::MUTED)).into(),
        Span::styled(
            elide_front(&config.executable_path_string(), width),
            Style::new().fg(palette::TEXT),
        )
        .into(),
        Line::default(),
        Span::styled("Log directory", Style::new().fg(palette::MUTED)).into(),
        Span::styled(
            elide_front(&config.log_dir.display().to_string(), width),
            Style::new().fg(palette::TEXT),
        )
        .into(),
        Line::default(),
        Span::styled(
            format!("Identity  {}", config.app_id),
            Style::new().fg(palette::MUTED),
        )
        .into(),
    ];

    let paragraph = Paragraph::new(lines)
        .block(panel().title(title_line(" Status ")))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

/// Shorten `text` to `max` columns by dropping its front and prefixing `…`.
///
/// This keeps the meaningful tail of a long path visible on one line.
fn elide_front(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }

    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max {
        return text.to_string();
    }

    let keep = max.saturating_sub(1);
    let tail: String = chars[chars.len() - keep..].iter().collect();
    format!("…{tail}")
}

/// The rolling activity log.
fn render_activity(frame: &mut Frame, area: Rect, dashboard: &Dashboard) {
    let block = panel().title(title_line(" Activity "));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Anchor the log to the bottom so the newest line is always at the last row.
    let visible = inner.height as usize;
    let start = dashboard.log.len().saturating_sub(visible);
    let shown = &dashboard.log[start..];
    let pad = visible.saturating_sub(shown.len());

    let mut lines: Vec<Line> = vec![Line::default(); pad];
    lines.extend(shown.iter().map(|entry| {
        let (mark, colour) = match entry.level {
            Level::Info => ("•", palette::ACCENT),
            Level::Success => ("✓", palette::SUCCESS),
            Level::Warn => ("!", palette::WARN),
            Level::Error => ("✕", palette::ERROR),
        };
        Line::from(vec![
            Span::styled(
                format!(
                    "[{:02}:{:02}]",
                    entry.at.as_secs() / 60,
                    entry.at.as_secs() % 60
                ),
                Style::new().fg(palette::MUTED),
            ),
            Span::raw("  "),
            Span::styled(mark, Style::new().fg(colour).bold()),
            Span::raw("  "),
            Span::styled(entry.text.clone(), Style::new().fg(palette::TEXT)),
        ])
    }));

    frame.render_widget(Paragraph::new(lines), inner);
}

/// The single-line footer with live indicator and key hints.
fn render_footer(frame: &mut Frame, area: Rect, dashboard: &Dashboard) {
    let columns = Layout::horizontal([Constraint::Length(14), Constraint::Min(0)]).split(area);

    let live = Line::from(vec![
        Span::styled(dashboard.spinner(), Style::new().fg(palette::ACCENT)),
        Span::styled(" live", Style::new().fg(palette::MUTED)),
    ]);
    frame.render_widget(Paragraph::new(live), columns[0]);

    let hints = Line::from(vec![
        key(" ↑/↓ "),
        Span::styled(" move   ", Style::new().fg(palette::MUTED)),
        key(" ⏎ "),
        Span::styled(" run   ", Style::new().fg(palette::MUTED)),
        key(" r "),
        Span::styled(" refresh   ", Style::new().fg(palette::MUTED)),
        key(" q "),
        Span::styled(" quit ", Style::new().fg(palette::MUTED)),
    ])
    .right_aligned();
    frame.render_widget(Paragraph::new(hints), columns[1]);
}

/// A rounded, muted-bordered panel with horizontal breathing room.
fn panel() -> Block<'static> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(palette::MUTED))
        .padding(Padding::horizontal(1))
}

/// A styled panel title line.
fn title_line(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(text, Style::new().fg(palette::ACCENT).bold()))
}

/// A "label  ● value" row.
fn key_value(label: &'static str, on: bool, value: &str) -> Line<'static> {
    let (dot, dot_colour) = if on {
        ("●", palette::SUCCESS)
    } else {
        ("○", palette::MUTED)
    };
    Line::from(vec![
        Span::styled(format!("{label:<14}"), Style::new().fg(palette::MUTED)),
        Span::styled(dot, Style::new().fg(dot_colour)),
        Span::styled(format!("  {value}"), Style::new().fg(palette::TEXT)),
    ])
}

/// A key-cap span for the footer.
fn key(text: &'static str) -> Span<'static> {
    Span::styled(
        text,
        Style::new()
            .fg(palette::ACCENT)
            .add_modifier(Modifier::BOLD),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

    use super::super::dashboard::{Level, Snapshot};

    /// Flatten a rendered buffer into newline-separated text.
    fn buffer_to_string(buffer: &Buffer) -> String {
        let width = buffer.area.width as usize;
        let mut out = String::new();
        for (i, cell) in buffer.content().iter().enumerate() {
            if i != 0 && i % width == 0 {
                out.push('\n');
            }
            out.push_str(cell.symbol());
        }
        out
    }

    fn sample(config: &AppConfig) -> Dashboard {
        let mut dashboard = Dashboard::new();
        dashboard.snapshot = Snapshot {
            auto_start: Some(true),
            instance_available: Some(true),
            tray: TrayState::Running { pid: 4242 },
        };
        dashboard.record(Level::Success, "tray agent started (pid 4242)");
        dashboard.record(Level::Info, "dashboard ready");
        let _ = config;
        dashboard
    }

    #[test]
    fn renders_every_section() {
        let config = AppConfig::new("trayforge", PathBuf::from("/opt/trayforge/bin/trayforge"));
        let dashboard = sample(&config);

        let mut terminal = Terminal::new(TestBackend::new(96, 30)).expect("terminal");
        terminal
            .draw(|frame| render(frame, &dashboard, &config))
            .expect("draw");

        let text = buffer_to_string(terminal.backend().buffer());
        println!("\n{text}\n");

        for needle in [
            "trayforge",
            "Commands",
            "Start tray agent",
            "Status",
            "Auto-start",
            "Activity",
            "pid 4242",
            "move",
            "quit",
        ] {
            assert!(text.contains(needle), "missing {needle:?} in:\n{text}");
        }
    }

    #[test]
    fn renders_at_a_small_size_without_panicking() {
        let config = AppConfig::new("trayforge", PathBuf::from("/opt/trayforge"));
        let dashboard = sample(&config);

        let mut terminal = Terminal::new(TestBackend::new(40, 12)).expect("terminal");
        terminal
            .draw(|frame| render(frame, &dashboard, &config))
            .expect("draw");
    }

    #[test]
    fn elide_front_keeps_the_tail() {
        assert_eq!(elide_front("short", 10), "short");
        assert_eq!(elide_front("abcdefghij", 5), "…ghij");
        assert_eq!(elide_front("abcdefghij", 0), "");
    }
}
