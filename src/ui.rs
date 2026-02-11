use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;
use crate::probe::CANDIDATES;
use crate::protocol::ChannelStatus;

/// Color for a given channel status.
fn status_color(status: ChannelStatus) -> Color {
    match status {
        ChannelStatus::Inactive => Color::DarkGray,
        ChannelStatus::Running => Color::Yellow,
        ChannelStatus::Captured => Color::Cyan,
        ChannelStatus::Confirmed => Color::Green,
        ChannelStatus::Unknown(_) => Color::Red,
    }
}

/// Render the full dashboard.
pub fn draw(f: &mut Frame, app: &App) {
    let mut constraints = vec![
        Constraint::Length(3), // Header bar
        Constraint::Min(8),   // Channel grid
    ];

    if app.probe_active {
        constraints.push(Constraint::Length(8)); // Probe panel
    }

    constraints.push(Constraint::Length(3)); // Raw hex
    constraints.push(Constraint::Length(3)); // Status bar

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(f.area());

    let mut idx = 0;
    draw_header(f, app, outer[idx]);
    idx += 1;
    draw_channels(f, app, outer[idx]);
    idx += 1;

    if app.probe_active {
        draw_probe_panel(f, app, outer[idx]);
        idx += 1;
    }

    draw_raw_hex(f, app, outer[idx]);
    idx += 1;
    draw_status_bar(f, app, outer[idx]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let connection_status = if app.connected { "CONNECTED" } else { "DISCONNECTED" };
    let conn_color = if app.connected { Color::Green } else { Color::Red };

    let spans = vec![
        Span::styled("  Device: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.device_mode),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Lane: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.lane),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   State: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.state_flag),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("   FPS: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}", app.frames_per_second),
            Style::default().fg(Color::White),
        ),
        Span::styled("   ", Style::default()),
        Span::styled(connection_status, Style::default().fg(conn_color)),
    ];

    let header = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" TRV Kocab Sports Timer ")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
    );
    f.render_widget(header, area);
}

fn draw_channels(f: &mut Frame, app: &App, area: Rect) {
    // Split into 2 rows
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Each row has 2 columns
    let top_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[0]);

    let bot_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(rows[1]);

    let areas = [top_cols[0], top_cols[1], bot_cols[0], bot_cols[1]];

    for (i, &channel_area) in areas.iter().enumerate() {
        draw_channel_box(f, &app.channels[i], i + 1, channel_area);
    }
}

fn draw_channel_box(f: &mut Frame, ch: &crate::protocol::TimeChannel, num: usize, area: Rect) {
    let color = status_color(ch.status);
    let time_str = ch.format_time();
    let status_str = format!("{}", ch.status);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("   {}", time_str),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("   {}", status_str),
            Style::default().fg(color),
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Channel {} ", num))
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD))
        .border_style(Style::default().fg(color));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn draw_probe_panel(f: &mut Frame, app: &App, area: Rect) {
    // Split into two halves: candidate selector (left) and log (right)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    draw_probe_selector(f, app, cols[0]);
    draw_probe_log(f, app, cols[1]);
}

fn draw_probe_selector(f: &mut Frame, app: &App, area: Rect) {
    let candidate = &CANDIDATES[app.probe_index];
    let hex = candidate
        .bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");

    let auto_indicator = if app.probe_auto_running { " AUTO" } else { "" };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  < ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                candidate.label,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" >", Style::default().fg(Color::DarkGray)),
            Span::styled(auto_indicator, Style::default().fg(Color::Magenta)),
        ]),
        Line::from(Span::styled(
            format!("  Hex: {}", hex),
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "  [Enter]=send  [A]=all  [\u{2190}\u{2192}]=nav",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let title = format!(
        " Probe [{}/{}] ",
        app.probe_index + 1,
        CANDIDATES.len()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn draw_probe_log(f: &mut Frame, app: &App, area: Rect) {
    let inner_height = area.height.saturating_sub(2) as usize; // subtract borders
    let log = &app.probe_log;

    // Show the most recent entries that fit
    let start = if log.len() > inner_height {
        log.len() - inner_height
    } else {
        0
    };

    let lines: Vec<Line> = log[start..]
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let num = start + i + 1;
            let status = if entry.success { "OK" } else { "ERR" };
            let status_color = if entry.success { Color::Green } else { Color::Red };

            let mode_before = format!("{}", entry.device_mode_before);
            let mode_transition = match entry.device_mode_after {
                Some(after) => {
                    let after_str = format!("{}", after);
                    if after_str == mode_before {
                        mode_before.clone()
                    } else {
                        format!("{}->{}", mode_before, after_str)
                    }
                }
                None => format!("{}->?", mode_before),
            };

            Line::from(vec![
                Span::styled(
                    format!(" #{:<2} ", num),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<14} ", entry.label),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!("{:<3} ", status),
                    Style::default().fg(status_color),
                ),
                Span::styled(mode_transition, Style::default().fg(Color::Gray)),
            ])
        })
        .collect();

    let title = format!(" Log ({} sent) ", log.len());

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
        .border_style(Style::default().fg(Color::Magenta));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

fn draw_raw_hex(f: &mut Frame, app: &App, area: Rect) {
    let hex = match &app.latest_frame {
        Some(frame) => frame.raw_hex_string(),
        None => "No data".to_string(),
    };

    // Truncate to fit
    let max_len = area.width.saturating_sub(4) as usize;
    let display = if hex.len() > max_len {
        format!("{} ...", &hex[..max_len.saturating_sub(4)])
    } else {
        hex
    };

    let paragraph = Paragraph::new(Line::from(Span::styled(
        format!("  {}", display),
        Style::default().fg(Color::DarkGray),
    )))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Raw ")
            .title_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(paragraph, area);
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let error_text = match &app.last_error {
        Some(e) => format!("  Last: {}", e),
        None => String::new(),
    };

    let mut spans = vec![
        Span::styled("  Frames: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.frame_count),
            Style::default().fg(Color::White),
        ),
        Span::styled("  Errors: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.error_count),
            Style::default().fg(if app.error_count > 0 {
                Color::Red
            } else {
                Color::White
            }),
        ),
        Span::styled(error_text, Style::default().fg(Color::Red)),
    ];

    // Build help text based on available features
    let mut help_parts = Vec::new();
    if app.probe_available() {
        help_parts.push("p=probe");
    }
    help_parts.push("q=quit");

    spans.push(Span::styled(
        format!("  |  {}", help_parts.join("  ")),
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    f.render_widget(paragraph, area);
}
