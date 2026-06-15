use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::scores::ScoreEntry;
use crate::types::World;

pub fn render(frame: &mut Frame, world: &World, top_scores: &[ScoreEntry]) {
    let area = frame.area();
    frame.render_widget(Clear, area);

    let title = format!(" Mission Briefing: {} ", world.mission_file);
    let outer = Block::default().borders(Borders::ALL).title(title);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Split: briefing text | scores sidebar (if scores exist)
    let has_scores = !top_scores.is_empty();
    let chunks = if has_scores {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(28)])
            .split(inner)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(inner)
    };

    // Briefing text
    let text = world.briefing.replace("\\\\", "\n").replace('\\', "\n");
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(chunks[0]);

    let lines: Vec<Line> = text
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(Color::White))))
        .collect();
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::NONE)),
        body_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "  Press any key to begin...",
                Style::default().fg(Color::DarkGray),
            )),
        ]),
        body_chunks[1],
    );

    // High scores sidebar
    if has_scores {
        let mut score_lines: Vec<Line> = vec![
            Line::from(Span::styled(
                " Top Scores",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ];
        for (i, entry) in top_scores.iter().enumerate() {
            score_lines.push(Line::from(vec![
                Span::styled(
                    format!(" {:1}. ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("{:<12}", truncate(&entry.player_name, 12)),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" {:>6}", entry.score),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }
        frame.render_widget(
            Paragraph::new(score_lines)
                .block(Block::default().borders(Borders::LEFT).title(" Records ")),
            chunks[1],
        );
    }
}

fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        &s[..max]
    }
}
