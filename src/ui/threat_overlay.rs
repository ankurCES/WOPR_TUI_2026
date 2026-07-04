use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::game::types::ThreatLevel;
use crate::ui::anim::pulse_style;
use crate::ui::icons;
use crate::ui::world_map::latlon_to_cell;

pub struct MissileTrajectory {
    pub origin: (f32, f32),
    pub target: (f32, f32),
    pub progress: f32,
    pub launched_at_tick: u64,
}

pub struct ThreatMarker {
    pub location: (f32, f32),
    pub severity: ThreatLevel,
}

pub struct BaseMarker {
    pub location: (f32, f32),
    pub country_code: &'static str,
    pub active: bool,
}

pub struct ThreatOverlay<'a> {
    pub missiles: &'a [MissileTrajectory],
    pub threats: &'a [ThreatMarker],
    pub bases: &'a [BaseMarker],
    pub tick: u64,
    pub nerd_fonts: bool,
}

impl Widget for ThreatOverlay<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ic = icons::icons(self.nerd_fonts);

        // bases
        for base in self.bases {
            let (x, y) = latlon_to_cell(base.location.0, base.location.1, area);
            if area.contains((x, y).into()) {
                let style = if base.active {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                buf[(x, y)].set_symbol(ic.base).set_style(style);
                if x + 1 + base.country_code.len() as u16 <= area.right() {
                    buf.set_string(x + 1, y, base.country_code, style);
                }
            }
        }

        // threats
        for threat in self.threats {
            let (x, y) = latlon_to_cell(threat.location.0, threat.location.1, area);
            if area.contains((x, y).into()) {
                let style = match threat.severity {
                    ThreatLevel::Low => Style::default().fg(Color::Green),
                    ThreatLevel::Medium => Style::default().fg(Color::Yellow),
                    ThreatLevel::High => Style::default().fg(Color::Red),
                    ThreatLevel::Critical => pulse_style(self.tick, 15, Color::LightRed, Color::Red),
                };
                buf[(x, y)].set_symbol(ic.threat).set_style(style);
            }
        }

        // missiles
        for missile in self.missiles {
            let (ox, oy) = latlon_to_cell(missile.origin.0, missile.origin.1, area);
            let (tx, ty) = latlon_to_cell(missile.target.0, missile.target.1, area);
            let p = missile.progress.clamp(0.0, 1.0);

            // parabolic arc: lerp x, lerp y with arc height
            let cx = ox as f32 + (tx as f32 - ox as f32) * p;
            let arc_height = 3.0 * (1.0 - (2.0 * p - 1.0).powi(2));
            let cy = oy as f32 + (ty as f32 - oy as f32) * p - arc_height;

            let mx = cx.clamp(area.x as f32, (area.right() - 1) as f32) as u16;
            let my = cy.clamp(area.y as f32, (area.bottom() - 1) as f32) as u16;

            if area.contains((mx, my).into()) {
                if p >= 1.0 {
                    // explosion
                    let boom_chars = ['*', '+', 'x', '*'];
                    let frame = ((self.tick - missile.launched_at_tick) % 4) as usize;
                    buf[(mx, my)]
                        .set_char(boom_chars[frame])
                        .set_style(Style::default().fg(Color::LightRed));
                } else {
                    buf[(mx, my)]
                        .set_symbol(ic.missile)
                        .set_style(Style::default().fg(Color::Red));
                }
            }

            // trail dots
            for i in 1..=3 {
                let tp = (p - i as f32 * 0.05).max(0.0);
                let trail_x = ox as f32 + (tx as f32 - ox as f32) * tp;
                let trail_arc = 3.0 * (1.0 - (2.0 * tp - 1.0).powi(2));
                let trail_y = oy as f32 + (ty as f32 - oy as f32) * tp - trail_arc;
                let trail_px = trail_x.clamp(area.x as f32, (area.right() - 1) as f32) as u16;
                let trail_py = trail_y.clamp(area.y as f32, (area.bottom() - 1) as f32) as u16;
                if area.contains((trail_px, trail_py).into()) {
                    buf[(trail_px, trail_py)]
                        .set_char('·')
                        .set_style(Style::default().fg(Color::DarkGray));
                }
            }
        }
    }
}
