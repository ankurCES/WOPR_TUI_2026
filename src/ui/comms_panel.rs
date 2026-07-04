use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::game::comms::{garble, priority_label};
use crate::game::types::{CommMessage, CommPriority, Country};

fn priority_style(p: &CommPriority) -> Style {
    match p {
        CommPriority::Flash => Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD),
        CommPriority::Immediate => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        CommPriority::Priority => Style::default().fg(Color::Yellow),
        CommPriority::Routine => Style::default().fg(Color::Green),
    }
}

fn country_flag(c: &Country) -> &'static str {
    match c {
        Country::Russia => "RU",
        Country::China => "CN",
        Country::USA => "US",
        Country::NATO => "NATO",
        Country::DPRK => "DPRK",
        Country::Iran => "IR",
        Country::India => "IN",
        Country::Pakistan => "PK",
        Country::Unknown => "??",
    }
}

pub struct CommsPanel<'a> {
    messages: &'a [CommMessage],
    scroll: usize,
}

impl<'a> CommsPanel<'a> {
    pub fn new(messages: &'a [CommMessage], scroll: usize) -> Self {
        Self { messages, scroll }
    }
}

impl Widget for CommsPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default().borders(Borders::ALL).title(" COMMS ");
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<Line<'_>> = Vec::new();

        for msg in self.messages.iter().skip(self.scroll) {
            // line 1: [PRIORITY] [COUNTRY] native_text
            let (display_text, _) = if msg.garbled_mask.is_empty() {
                (msg.native_text.clone(), Vec::new())
            } else {
                garble(&msg.native_text, 0.0) // already garbled at creation
            };

            let header = format!("[{}] [{}] ", priority_label(&msg.priority), country_flag(&msg.origin));
            lines.push(Line::from(vec![
                Span::styled(header, priority_style(&msg.priority)),
                Span::styled(
                    if msg.garbled_mask.iter().any(|&m| m) {
                        apply_garble(&msg.native_text, &msg.garbled_mask)
                    } else {
                        display_text
                    },
                    Style::default().fg(Color::White),
                ),
            ]));

            // line 2: english translation (dim, indented)
            lines.push(Line::from(Span::styled(
                format!("  {}", msg.english_translation),
                Style::default().fg(Color::DarkGray),
            )));

            // spacer
            lines.push(Line::from(""));
        }

        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .render(inner, buf);
    }
}

fn apply_garble(text: &str, mask: &[bool]) -> String {
    text.chars()
        .zip(mask.iter().chain(std::iter::repeat(&false)))
        .map(|(c, &m)| if m && !c.is_whitespace() { '\u{2592}' } else { c })
        .collect()
}
