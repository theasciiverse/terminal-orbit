use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::save::SaveSlot;

// Two separate art blocks so each can be centered at its own natural width.
const TERMINAL_ART: &str = r#" _____ _____  ____   __  __  ___  _   _     _    _
|_   _|| ____||  _ \ |  \/  ||_ _|| \ | |   / \  | |
  | |  |  _|  | |_) || |\/| | | | |  \| |  / _ \ | |
  | |  | |___ |  _ < | |  | | | | | |\  | / ___ \| |___
  |_|  |_____||_| \_\|_|  |_||___|_| \_|/_/   \_\|_____|"#;

const ORBIT_ART: &str = r#"   ___  ____  ____  ___  ____
  / _ \|  _ \| __ )|_ _||_   _|
 | | | | |_) |  _ \ | |   | |
 | |_| |  _ <| |_) || |   | |
  \___/|_| \_|____/|___|  |_|"#;

const SUBTITLE: &str = "Space Combat Simulator — lovingly translated to TUI\n  Originally by Steve Belczyk (1999)";

pub const TITLE_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

/// Pad all lines of an art block to the block's own max width, then
/// center the whole block.  Keeps internal letter-column alignment
/// while letting ratatui center the block in the terminal.
fn art_lines(art: &str, color: Color) -> Vec<Line<'static>> {
    let max_w = art.lines().map(|l| l.len()).max().unwrap_or(0);
    art.lines()
        .map(|l| {
            let padded = format!("{:<width$}", l, width = max_w);
            Line::from(Span::styled(padded, Style::default().fg(color)))
        })
        .collect()
}

pub fn render_title(frame: &mut Frame, sel: usize, saves: &[SaveSlot]) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" Terminal Orbit ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Heights for each section.
    let terminal_h: u16 = TERMINAL_ART.lines().count() as u16;  // 5
    let orbit_h:    u16 = ORBIT_ART.lines().count() as u16;     // 5
    let subtitle_h: u16 = 2;
    let menu_h:     u16 = TITLE_OPTIONS.len() as u16 + 3;       // blank + N opts + blank + hint

    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(terminal_h),
        Constraint::Length(1),            // blank between words
        Constraint::Length(orbit_h),
        Constraint::Length(1),            // blank before subtitle
        Constraint::Length(subtitle_h),
        Constraint::Length(menu_h),
        Constraint::Fill(1),
    ])
    .split(inner);

    // TERMINAL word — centered at its own width.
    frame.render_widget(
        Paragraph::new(art_lines(TERMINAL_ART, Color::Cyan)).alignment(Alignment::Center),
        chunks[1],
    );

    // ORBIT word — centered at its own (narrower) width.
    frame.render_widget(
        Paragraph::new(art_lines(ORBIT_ART, Color::Cyan)).alignment(Alignment::Center),
        chunks[3],
    );

    // Subtitle lines.
    let sub_lines: Vec<Line> = SUBTITLE
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::DarkGray))))
        .collect();
    frame.render_widget(
        Paragraph::new(sub_lines).alignment(Alignment::Center),
        chunks[5],
    );

    // Menu: blank line, options, blank line, hint.
    // All options padded to the same width. Use plain ">" so it is
    // always exactly 1 cell wide (► can render 2 cells in some terminals).
    let max_opt = TITLE_OPTIONS.iter().map(|o| o.len()).max().unwrap_or(8);
    let mut menu_lines: Vec<Line> = vec![Line::from("")];

    for (i, opt) in TITLE_OPTIONS.iter().enumerate() {
        let disabled = i == 1 && saves.is_empty();
        let padded = format!("{:<width$}", opt, width = max_opt);
        let line = if disabled {
            Line::from(Span::styled(
                format!("    {}  (no saves)", padded),
                Style::default().fg(Color::DarkGray),
            ))
        } else if i == sel {
            Line::from(Span::styled(
                format!("  > {}  ", padded),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                format!("    {}  ", padded),
                Style::default().fg(Color::White),
            ))
        };
        menu_lines.push(line);
    }

    menu_lines.push(Line::from(""));
    menu_lines.push(Line::from(Span::styled(
        "[↑/↓] Navigate   [Enter] Select   [Q] Quit",
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(
        Paragraph::new(menu_lines).alignment(Alignment::Center),
        chunks[6],
    );
}

pub fn render_load_menu(frame: &mut Frame, slots: &[SaveSlot], sel: usize) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" Load Game ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let mut lines: Vec<Line> = vec![
        Line::from(""),
        Line::from(Span::styled(
            "  Select a save slot:",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
    ];

    for (i, slot) in slots.iter().enumerate() {
        let row = format!(
            "  {}. {:30} {}",
            i,
            slot.mission,
            slot.display_time()
        );
        let style = if i == sel {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if i == sel { ">" } else { " " };
        lines.push(Line::from(Span::styled(
            format!("  {} {}", prefix, row.trim_start()),
            style,
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  [↑/↓] Navigate   [Enter] Load   [Q/Esc] Back",
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}
