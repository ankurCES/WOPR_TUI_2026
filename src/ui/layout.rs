use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::mode::Mode;
use crate::state::AppState;
use crate::view::ViewState;

use super::comms_panel::CommsPanel;
use super::country_select::CountrySelectScreen;
use super::decision::DecisionPanel;
use super::loading::LoadingOverlay;
use super::threat_overlay::ThreatOverlay;
use super::world_map::WorldMap;

pub fn render(frame: &mut Frame, view: &ViewState, state: &AppState) {
    if state.show_splash {
        super::splash::render_splash(frame, frame.area(), state);
        return;
    }
    if state.show_country_select {
        let area = centered_rect(70, 90, frame.area());
        frame.render_widget(CountrySelectScreen { selected: state.country_select_index }, area);
        return;
    }
    render_status_bar(frame, view, state);
    render_content(frame, view, state);
    render_bottom_panel(frame, view, state);
    render_input_bar(frame, view, state);
    if state.show_help {
        render_help(frame, view);
    }
    if state.llm_loading {
        frame.render_widget(
            LoadingOverlay { tick: state.tick_count, start_tick: state.llm_loading_start_tick },
            view.top_content,
        );
    }
}

fn render_status_bar(frame: &mut Frame, view: &ViewState, state: &AppState) {
    let dl = state.defcon.level();
    let color = state.defcon.color();
    let label = state.defcon.label();
    let defcon_style = if state.defcon_flash_remaining > 0 && state.tick_count % 10 < 5 {
        Style::default().fg(color).bg(Color::Black).add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().fg(Color::Black).bg(color).add_modifier(Modifier::BOLD)
    };
    let w = view.status_bar.width;
    let mut spans = vec![
        Span::styled(format!(" DEFCON {} ", dl), defcon_style),
        Span::styled(format!(" {} ", label), Style::default().fg(color)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", state.mode), Style::default().fg(Color::White)),
        Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
    ];
    if w >= 80 {
        spans.push(Span::styled(format!("T{:06}", state.tick_count), Style::default().fg(Color::DarkGray)));
    }
    spans.push(Span::styled(
        format!("  TURN {}", state.game_context.turn_number),
        Style::default().fg(Color::Yellow),
    ));
    if w >= 60 {
        spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        spans.push(tension_gauge("MIL", state.game_context.world_state.military_tension));
        spans.push(Span::raw(" "));
        spans.push(tension_gauge("ECO", 1.0 - state.game_context.world_state.economic_stability));
        spans.push(Span::raw(" "));
        spans.push(tension_gauge("POL", state.game_context.world_state.political_unrest));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)), view.status_bar);
}

fn render_content(frame: &mut Frame, view: &ViewState, state: &AppState) {
    match &state.mode {
        Mode::MainMap | Mode::Scenario => {
            if view.top_content.width < 100 {
                let rows = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                    .split(view.top_content);
                render_main_map(frame, rows[0], state);
                render_comms_stream(frame, rows[1], state);
            } else {
                let cols = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(view.top_content);
                render_main_map(frame, cols[0], state);
                render_comms_stream(frame, cols[1], state);
            }
        }
        Mode::Comms => render_comms(frame, view.top_content, state),
        Mode::Defcon => render_defcon(frame, view.top_content, state),
        Mode::Settings => render_settings(frame, view.top_content, state),
        Mode::About => super::about::render_about(frame, view.top_content),
    }
}

fn render_bottom_panel(frame: &mut Frame, view: &ViewState, state: &AppState) {
    if state.game_over {
        render_endgame(frame, view.bottom_panel, state);
        return;
    }
    match &state.current_scenario {
        Some(scenario) => {
            let mut panel = DecisionPanel::new(scenario, state.selected_option);
            if let Some((remaining, total)) = state.countdown {
                panel = panel.with_countdown(remaining, total);
            }
            frame.render_widget(panel, view.bottom_panel);
        }
        None => {
            let block = Block::default().borders(Borders::ALL).title(" DECISION CENTER ");
            let inner = block.inner(view.bottom_panel);
            frame.render_widget(block, view.bottom_panel);
            frame.render_widget(
                Paragraph::new("AWAITING ORDERS...")
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow)),
                inner,
            );
        }
    }
}

fn render_endgame(frame: &mut Frame, area: Rect, state: &AppState) {
    let is_victory = state.game_outcome_message.as_deref()
        .map(|m| m.contains("AVERTED") || m.contains("PEACE"))
        .unwrap_or(false);
    let (title, color) = if is_victory {
        (" VICTORY ", Color::Green)
    } else {
        (" DEFEAT ", Color::Red)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let turns = state.game_context.turn_number;
    let decisions = state.game_context.player_decisions.len();

    let art = if is_victory {
        crate::game::endgame::VICTORY_ART
    } else {
        crate::game::endgame::DEFEAT_ART
    };

    let mut lines: Vec<Line> = art.lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(color))))
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("Turns: {}  Decisions: {}  Final DEFCON: {}", turns, decisions, state.defcon.level()),
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  [R] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled("Play Again    ", Style::default().fg(Color::White)),
        Span::styled("[Q] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Quit", Style::default().fg(Color::White)),
    ]));

    frame.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center),
        inner,
    );
}

fn render_main_map(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title(" GLOBAL STRATEGIC MAP ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(WorldMap {
        missiles: &state.missiles,
        threats: &state.threats,
        comms: &state.comms,
        player_country: state.player_country,
        tick: state.tick_count,
    }, inner);
    frame.render_widget(
        ThreatOverlay {
            missiles: &state.missiles,
            threats: &state.threats,
            bases: &state.bases,
            tick: state.tick_count,
            nerd_fonts: state.nerd_fonts,
        },
        inner,
    );
}

fn render_comms(frame: &mut Frame, area: Rect, state: &AppState) {
    frame.render_widget(
        CommsPanel::new(&state.comms, state.comms_scroll),
        area,
    );
}

fn render_comms_stream(frame: &mut Frame, area: Rect, state: &AppState) {
    // ponytail: auto-scroll to bottom so latest comms visible = streaming feel
    let lines_per_msg = 3; // header + translation + spacer
    let inner_height = area.height.saturating_sub(2) as usize; // border
    let total_lines = state.comms.len() * lines_per_msg;
    let auto_scroll = if total_lines > inner_height {
        (total_lines - inner_height) / lines_per_msg
    } else {
        0
    };
    frame.render_widget(
        CommsPanel::new(&state.comms, auto_scroll),
        area,
    );
}

fn render_defcon(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title(" DEFCON STATUS ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let dl = state.defcon.level();
    let color = state.defcon.color();

    let mut lines = Vec::new();
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  DEFCON {}", dl),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        format!("  {}", state.defcon.label()),
        Style::default().fg(color),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  {}", state.defcon.description()),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    // defcon gauge
    for level in (1..=5).rev() {
        let is_current = level == dl;
        let lc = match level {
            5 => Color::Green,
            4 => Color::Cyan,
            3 => Color::Yellow,
            2 => Color::Magenta,
            _ => Color::Red,
        };
        let bar = if is_current { "████████████████" } else { "░░░░░░░░░░░░░░░░" };
        let marker = if is_current { " ◀" } else { "" };
        lines.push(Line::from(vec![
            Span::styled(format!("  DEFCON {} ", level), Style::default().fg(lc)),
            Span::styled(bar, Style::default().fg(if is_current { lc } else { Color::DarkGray })),
            Span::styled(marker, Style::default().fg(lc)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!("  Turn: {}  Decisions: {}  Threats: {}",
            state.game_context.turn_number,
            state.game_context.player_decisions.len(),
            state.threats.len(),
        ),
        Style::default().fg(Color::DarkGray),
    )));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn render_settings(frame: &mut Frame, area: Rect, _state: &AppState) {
    let block = Block::default().borders(Borders::ALL).title(" SETTINGS ");
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(
        Paragraph::new("Settings configured via ~/.wopr/settings.json")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        inner,
    );
}

fn render_input_bar(frame: &mut Frame, view: &ViewState, state: &AppState) {
    let hints = if state.game_over {
        " [R] Play Again  [Q] Quit"
    } else if state.current_scenario.is_some() {
        " [1-4] Select  [Enter] Confirm  [Tab] Mode  [q] Quit"
    } else {
        " [Tab] Next  [Shift+Tab] Prev  [?] Help  [q] Quit"
    };
    frame.render_widget(
        Paragraph::new(hints).style(Style::default().fg(Color::DarkGray)),
        view.input_bar,
    );
}

fn render_help(frame: &mut Frame, view: &ViewState) {
    let (px, py) = if view.top_content.width < 100 { (80, 60) } else { (50, 50) };
    let popup = centered_rect(px, py, view.top_content);
    let block = Block::default().borders(Borders::ALL).title(" HELP ");
    let inner = block.inner(popup);
    frame.render_widget(Clear, popup);
    frame.render_widget(block, popup);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from("  [Tab]         Next mode"),
            Line::from("  [Shift+Tab]   Prev mode"),
            Line::from("  [1-4]         Select option"),
            Line::from("  [Up/Down]     Navigate options"),
            Line::from("  [Enter]       Confirm decision"),
            Line::from("  [?]           Toggle help"),
            Line::from("  [q]           Quit"),
            Line::from(""),
            Line::from(Span::styled(
                "  MODES: Map │ Comms │ About │ Settings │ Defcon",
                Style::default().fg(Color::Yellow),
            )),
            Line::from(Span::styled(
                "  (Decision panel always visible below)",
                Style::default().fg(Color::Yellow),
            )),
        ])
        .wrap(Wrap { trim: false }),
        inner,
    );
}

fn tension_gauge<'a>(label: &'a str, value: f32) -> Span<'a> {
    let filled = (value * 4.0).round() as usize;
    let bar: String = "▓".repeat(filled.min(4)) + &"░".repeat(4_usize.saturating_sub(filled));
    let color = if value > 0.7 { Color::Red } else if value > 0.4 { Color::Yellow } else { Color::Green };
    Span::styled(format!("{}:{}", label, bar), Style::default().fg(color))
}

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(vert[1])[1]
}
