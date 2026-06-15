use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::save::SaveSlot;

const LOGO: &str = concat!(
    "   ___  ____  ____  ___  ____  \n",
    "  / _ \\|  _ \\| __ )|_ _||_   _|\n",
    " | | | | |_) |  _ \\ | |   | |  \n",
    " | |_| |  _ <| |_) || |   | |  \n",
    "  \\___/|_| \\_|____/|___|  |_|  \n",
    "\n",
    "   Space Combat Simulator       \n",
    "   (C) 1999 Steve Belczyk       ",
);

pub const TITLE_OPTIONS: &[&str] = &["New Game", "Load Game", "Quit"];

pub fn render_title(frame: &mut Frame, sel: usize, saves: &[SaveSlot]) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(" ORBIT — Terminal Edition ");
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(5),
        ])
        .split(inner);

    // Logo
    let logo_lines: Vec<Line> = LOGO.lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::Cyan))))
        .collect();
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Center),
        chunks[0],
    );

    // Menu + hints
    let mut menu_lines: Vec<Line> = Vec::new();
    menu_lines.push(Line::from(""));

    for (i, opt) in TITLE_OPTIONS.iter().enumerate() {
        let disabled = i == 1 && saves.is_empty();
        let line = if disabled {
            Line::from(Span::styled(
                format!("    {}  (no saves)", opt),
                Style::default().fg(Color::DarkGray),
            ))
        } else if i == sel {
            Line::from(Span::styled(
                format!("  ► {}  ", opt),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            Line::from(Span::styled(
                format!("    {}  ", opt),
                Style::default().fg(Color::White),
            ))
        };
        menu_lines.push(line);
    }

    menu_lines.push(Line::from(""));
    menu_lines.push(Line::from(Span::styled(
        "  [↑/↓] Navigate   [Enter] Select   [Q] Quit",
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(
        Paragraph::new(menu_lines).alignment(Alignment::Center),
        chunks[1],
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
