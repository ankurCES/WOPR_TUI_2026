use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

const TITLE: &str = r#" ██╗    ██╗  ·  ██████╗   ·  ██████╗   ·  ██████╗
 ██║    ██║  ·  ██╔══██╗  ·  ██╔══██╗  ·  ██╔══██╗
 ██║ █╗ ██║  ·  ██║  ██║  ·  ██████╔╝  ·  ██████╔╝
 ██║███╗██║  ·  ██║  ██║  ·  ██╔═══╝   ·  ██╔══██╗
 ╚███╔███╔╝  ·  ╚██████╔╝  ·  ██║       ·  ██║  ██║
  ╚══╝╚══╝   ·   ╚═════╝   ·  ╚═╝       ·  ╚═╝  ╚═╝"#;

fn hazard_line<'a>() -> Line<'a> {
    Line::from(vec![
        Span::styled("▓▓▓▓", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled("░░░░", Style::default().fg(Color::Yellow).bg(Color::Black)),
        Span::styled(" ☢ ", Style::default().fg(Color::Yellow)),
        Span::styled("▓▓▓▓", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled("░░░░", Style::default().fg(Color::Yellow).bg(Color::Black)),
        Span::styled(" ☢ ", Style::default().fg(Color::Yellow)),
        Span::styled("▓▓▓▓", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled("░░░░", Style::default().fg(Color::Yellow).bg(Color::Black)),
        Span::styled(" ☢ ", Style::default().fg(Color::Yellow)),
        Span::styled("▓▓▓▓", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled("░░░░", Style::default().fg(Color::Yellow).bg(Color::Black)),
    ])
}

pub fn render_splash(frame: &mut Frame, area: Rect, tick: u64) {
    frame.render_widget(Clear, area);

    let title_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
    let green_bold = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(""));
    lines.push(hazard_line());
    lines.push(Line::from(""));
    for l in TITLE.lines() {
        lines.push(Line::from(Span::styled(l, title_style)));
    }
    lines.push(Line::from(""));
    lines.push(hazard_line());
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("SHALL WE PLAY A GAME?", green_bold)));
    lines.push(Line::from(""));

    // ponytail: tick % 60 gives ~0.5s blink at 60fps
    let blink = if tick % 60 < 30 { "PRESS ANY KEY TO CONTINUE" } else { "" };
    lines.push(Line::from(Span::styled(blink, Style::default().fg(Color::Green))));

    let content_h = lines.len() as u16;
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(content_h),
            Constraint::Fill(1),
        ])
        .split(area);

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        vert[1],
    );
}
