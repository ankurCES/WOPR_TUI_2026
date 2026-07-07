use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::game::types::Country;

pub const COUNTRIES: &[(Country, &str)] = &[
    (Country::USA, "NATO leader, largest nuclear arsenal, global force projection"),
    (Country::Russia, "Nuclear superpower, vast land force, resurgent global ambitions"),
    (Country::China, "Emerging superpower, economic leverage, growing blue-water navy"),
    (Country::UnitedKingdom, "NATO member, independent nuclear deterrent, special intelligence ties"),
    (Country::France, "Independent nuclear force, EU anchor, non-aligned NATO doctrine"),
    (Country::India, "Nuclear-armed, largest standing army, regional hegemon"),
    (Country::Pakistan, "Nuclear state, ISI asymmetric capability, South Asia flashpoint"),
    (Country::DPRK, "Rogue nuclear state, ICBM capability, unpredictable doctrine"),
];

pub struct CountrySelectScreen {
    pub selected: usize,
}

impl Widget for CountrySelectScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" SELECT YOUR NATION ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
        let inner = block.inner(area);
        block.render(area, buf);

        let mut lines: Vec<Line> = vec![Line::from("")];
        for (i, (country, posture)) in COUNTRIES.iter().enumerate() {
            let is_selected = i == self.selected;
            let (name_style, desc_style) = if is_selected {
                (
                    Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                    Style::default().fg(Color::Black).bg(Color::Green),
                )
            } else {
                (
                    Style::default().fg(Color::Green),
                    Style::default().fg(Color::DarkGray),
                )
            };
            let arrow = if is_selected { "▶ " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!("  {}{}", arrow, country.full_name()),
                name_style,
            )));
            lines.push(Line::from(Span::styled(
                format!("      {}", posture),
                desc_style,
            )));
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            "  [↑↓] Navigate   [Enter] Confirm",
            Style::default().fg(Color::DarkGray),
        )));

        // Scroll to keep selected item visible
        let lines_per_item: u16 = 3;
        let selected_top = 1 + self.selected as u16 * lines_per_item;
        let visible = inner.height;
        let scroll_y = if selected_top + lines_per_item > visible {
            selected_top + lines_per_item - visible
        } else {
            0
        };

        Paragraph::new(lines)
            .wrap(Wrap { trim: true })
            .scroll((scroll_y, 0))
            .render(inner, buf);
    }
}
