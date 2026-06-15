use ratatui::{
    layout::{Alignment, Constraint, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::save::SaveSlot;

// Raw string avoids escaping all the backslashes in the ASCII art.
const LOGO: &str = r#" _____ _____  ____   __  __  ___  _   _     _    _
|_   _|| ____||  _ \ |  \/  ||_ _|| \ | |   / \  | |
  | |  |  _|  | |_) || |\/| | | | |  \| |  / _ \ | |
  | |  | |___ |  _ < | |  | | | | | |\  | / ___ \| |___
  |_|  |_____||_| \_\|_|  |_||___|_| \_|/_/   \_\|_____|

   ___  ____  ____  ___  ____
  / _ \|  _ \| __ )|_ _||_   _|
 | | | | |_) |  _ \ | |   | |
 | |_| |  _ <| |_) || |   | |
  \___/|_| \_|____/|___|  |_|

   Space Combat Simulator
   (C) 1999 Steve Belczyk       "#;

pub const TITLE_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

pub fn render_title(frame: &mut Frame, sel: usize, saves: &[SaveSlot]) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" ORBIT — Terminal Edition ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Vertical centering: Fill spacers push the content block to the middle.
    // Logo: 5 art rows + 1 blank + 2 subtitle = 8 rows + 1 top padding = 9
    // Menu: 1 blank + N options + 1 blank + 1 hint = N + 3
    let logo_height: u16 = 14;
    let menu_height: u16 = TITLE_OPTIONS.len() as u16 + 3;
    let chunks = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(logo_height),
        Constraint::Length(menu_height),
        Constraint::Fill(1),
    ])
    .split(inner);

    // Pad every logo row to the same width so Alignment::Center keeps a
    // consistent left edge across rows of different visual widths.
    let logo_max_w = LOGO.lines().map(|l| l.len()).max().unwrap_or(0);
    let logo_lines: Vec<Line> = LOGO.lines()
        .map(|l| {
            let padded = format!("{:<width$}", l, width = logo_max_w);
            Line::from(Span::styled(padded, Style::default().fg(Color::Cyan)))
        })
        .collect();
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Center),
        chunks[1],
    );

    // Menu: blank line, options, blank line, hint.
    // Pad all options to the same width so Alignment::Center keeps the
    // cursor column and text column stable regardless of which item is selected.
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
                format!("  ► {}  ", padded),
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
        chunks[2],
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
        let prefix = if i == sel { "►" } else { " " };
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
