use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::game::types::Scenario;

pub struct DecisionPanel<'a> {
    scenario: &'a Scenario,
    selected: usize,
    countdown: Option<(u64, u64)>, // (remaining_ticks, total_ticks)
}

impl<'a> DecisionPanel<'a> {
    pub fn new(scenario: &'a Scenario, selected: usize) -> Self {
        Self { scenario, selected, countdown: None }
    }

    pub fn with_countdown(mut self, remaining: u64, total: u64) -> Self {
        self.countdown = Some((remaining, total));
        self
    }
}

impl Widget for DecisionPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.scenario.title))
            .style(Style::default().fg(Color::Green));
        let inner = block.inner(area);
        Clear.render(area, buf);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(2),        // description
                Constraint::Length(1),     // separator
                Constraint::Min(self.scenario.player_options.len() as u16 + 1), // options
                Constraint::Length(if self.countdown.is_some() { 2 } else { 0 }), // countdown
            ])
            .split(inner);

        // description
        Paragraph::new(self.scenario.description.as_str())
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White))
            .render(chunks[0], buf);

        // separator
        let sep = "─".repeat(chunks[1].width as usize);
        Paragraph::new(sep)
            .style(Style::default().fg(Color::DarkGray))
            .render(chunks[1], buf);

        // options
        let mut option_lines = vec![Line::from(Span::styled(
            "SELECT RESPONSE:",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ))];
        for (i, opt) in self.scenario.player_options.iter().enumerate() {
            let style = if i == self.selected {
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            };
            let prefix = if i == self.selected { "▶ " } else { "  " };
            option_lines.push(Line::from(Span::styled(
                format!("{prefix}{}. {} — {}", i + 1, opt.label, opt.description),
                style,
            )));
        }
        Paragraph::new(option_lines).wrap(Wrap { trim: true }).render(chunks[2], buf);

        // countdown bar
        if let Some((remaining, total)) = self.countdown {
            let pct = if total > 0 { remaining as f64 / total as f64 } else { 0.0 };
            let bar_width = chunks[3].width.saturating_sub(2) as usize;
            let filled = (pct * bar_width as f64) as usize;
            let bar_color = if pct > 0.5 { Color::Yellow } else { Color::Red };

            let bar: String = format!(
                "[{}{}]",
                "█".repeat(filled),
                "░".repeat(bar_width.saturating_sub(filled)),
            );
            Paragraph::new(Line::from(Span::styled(bar, Style::default().fg(bar_color))))
                .alignment(Alignment::Center)
                .render(chunks[3], buf);
        }
    }
}

pub struct EndScreen<'a> {
    pub art: &'a str,
    pub summary_lines: Vec<String>,
}

impl Widget for EndScreen<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::default().borders(Borders::ALL).title(" GAME OVER ");
        let inner = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        Paragraph::new(self.art)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Red))
            .render(chunks[0], buf);

        let lines: Vec<Line<'_>> = self
            .summary_lines
            .iter()
            .map(|s| Line::from(Span::styled(s.as_str(), Style::default().fg(Color::Green))))
            .collect();
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .render(chunks[1], buf);
    }
}
