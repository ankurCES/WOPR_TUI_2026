use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

const STEPS: [&str; 4] = [
    "ENCRYPTING SECURE CHANNEL",
    "TRANSMITTING TO NORAD",
    "DECODING RESPONSE",
    "ANALYZING THREAT DATA",
];
const BAR_LEN: usize = 8;
const TICKS_PER_STEP: u64 = 25;

pub struct LoadingOverlay {
    pub tick: u64,
    pub start_tick: u64,
}

impl Widget for LoadingOverlay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pw = area.width.saturating_sub(4).min(52);
        let ph = area.height.saturating_sub(4).min(12);
        if pw < 30 || ph < 6 {
            return;
        }
        let popup = Rect {
            x: area.x + (area.width - pw) / 2,
            y: area.y + (area.height - ph) / 2,
            width: pw,
            height: ph,
        };

        Clear.render(popup, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .title(Span::styled(
                " ◈ WOPR PROCESSING ◈ ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(Color::Black));
        let inner = block.inner(popup);
        block.render(popup, buf);

        let elapsed = self.tick.saturating_sub(self.start_tick);
        let cycle = TICKS_PER_STEP * STEPS.len() as u64;
        let phase = elapsed % cycle;
        let current_step = (phase / TICKS_PER_STEP) as usize;
        let step_progress = phase % TICKS_PER_STEP;

        // Color cycles Green→Yellow→Red→White every 30 ticks
        const CYCLE_COLORS: [Color; 4] = [Color::Green, Color::Yellow, Color::Red, Color::White];
        let accent = CYCLE_COLORS[(self.tick / 30) as usize % CYCLE_COLORS.len()];

        let bar_len = (BAR_LEN * pw as usize / 52).max(2);
        let label_width = (pw.saturating_sub(2) as usize).saturating_sub(1 + bar_len).min(27);

        let mut lines = vec![Line::from("")];

        for (i, label) in STEPS.iter().enumerate() {
            let (filled, label_style, bar_color) = if i < current_step {
                (
                    bar_len,
                    Style::default().fg(Color::Green),
                    Color::Green,
                )
            } else if i == current_step {
                let f = (step_progress as usize * bar_len) / TICKS_PER_STEP as usize;
                (
                    f,
                    Style::default().fg(accent).add_modifier(Modifier::BOLD),
                    accent,
                )
            } else {
                (
                    0,
                    Style::default().fg(Color::DarkGray),
                    Color::DarkGray,
                )
            };

            let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_len - filled));
            lines.push(Line::from(vec![
                Span::styled(format!(" {:<width$}", label, width = label_width), label_style),
                Span::styled(bar, Style::default().fg(bar_color)),
            ]));
        }

        lines.push(Line::from(""));

        let blink = if (elapsed / 8) % 2 == 0 { "█" } else { " " };
        lines.push(Line::from(Span::styled(
            format!(" AWAITING SECURE RESPONSE {}", blink),
            Style::default().fg(Color::Green).add_modifier(Modifier::DIM),
        )));

        Paragraph::new(lines)
            .style(Style::default().fg(Color::Green).bg(Color::Black))
            .render(inner, buf);
    }
}
