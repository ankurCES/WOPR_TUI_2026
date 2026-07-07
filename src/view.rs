use ratatui::layout::{Constraint, Direction, Layout, Rect};

use crate::state::AppState;

pub struct ViewState {
    pub status_bar: Rect,
    pub top_content: Rect,
    pub bottom_panel: Rect,
    pub input_bar: Rect,
}

pub fn compute_view(_state: &AppState, area: Rect) -> ViewState {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let (top_pct, bot_pct) = if area.height <= 20 {
        (40, 60)
    } else if area.height <= 30 {
        (50, 50)
    } else {
        (65, 35)
    };
    let middle = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(top_pct),
            Constraint::Percentage(bot_pct),
        ])
        .split(outer[1]);

    ViewState {
        status_bar: outer[0],
        top_content: middle[0],
        bottom_panel: middle[1],
        input_bar: outer[2],
    }
}
