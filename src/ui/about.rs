use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn render_about(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ABOUT ")
        .style(Style::default().fg(Color::Green));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let green = Style::default().fg(Color::Green);
    let bold_green = green.add_modifier(Modifier::BOLD);
    let yellow = Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC);
    let dim = Style::default().fg(Color::DarkGray);
    let white = Style::default().fg(Color::White);

    let sep_width = (inner.width.saturating_sub(4) as usize).min(51);
    let sep = "═".repeat(sep_width);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(sep.clone(), dim)),
        Line::from(Span::styled("W A R G A M E S   (1983)", bold_green)),
        Line::from(Span::styled(sep.clone(), dim)),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Directed by .............. ", dim),
            Span::styled("John Badham", white),
        ]),
        Line::from(vec![
            Span::styled("  Written by ............... ", dim),
            Span::styled("Lawrence Lasker", white),
        ]),
        Line::from(vec![
            Span::styled("                               ", dim),
            Span::styled("Walter F. Parkes", white),
        ]),
        Line::from(""),
        Line::from(Span::styled("  CAST", bold_green)),
        Line::from(vec![
            Span::styled("  David Lightman ........... ", dim),
            Span::styled("Matthew Broderick", white),
        ]),
        Line::from(vec![
            Span::styled("  Jennifer Mack ............ ", dim),
            Span::styled("Ally Sheedy", white),
        ]),
        Line::from(vec![
            Span::styled("  Dr. Stephen Falken ....... ", dim),
            Span::styled("John Wood", white),
        ]),
        Line::from(vec![
            Span::styled("  Dr. John McKittrick ...... ", dim),
            Span::styled("Dabney Coleman", white),
        ]),
        Line::from(""),
        Line::from(Span::styled("  THE WOPR", bold_green)),
        Line::from(Span::styled("  War Operation Plan Response — a NORAD supercomputer", green)),
        Line::from(Span::styled("  designed to run nuclear war simulations and predict", green)),
        Line::from(Span::styled("  outcomes of global thermonuclear conflict.", green)),
        Line::from(""),
        Line::from(Span::styled(sep.clone(), dim)),
        Line::from(""),
        Line::from(Span::styled("    \"The only winning move is not to play.\"", yellow)),
        Line::from(Span::styled("                              — WOPR / Joshua", dim)),
        Line::from(""),
        Line::from(Span::styled(sep.clone(), dim)),
        Line::from(""),
        Line::from(Span::styled("  WOPR TUI 2026 — A loving tribute to the film that", green)),
        Line::from(Span::styled("  taught a generation that some games cannot be won.", green)),
        Line::from(""),
        Line::from(Span::styled("  Built with Rust, ratatui, and a healthy fear of", dim)),
        Line::from(Span::styled("  mutually assured destruction.", dim)),
    ];

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).wrap(Wrap { trim: false }),
        inner,
    );
}
