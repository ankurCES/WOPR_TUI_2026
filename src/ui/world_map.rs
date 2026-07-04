use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Widget,
    },
};

use crate::game::types::ThreatLevel;
use crate::ui::threat_overlay::{MissileTrajectory, ThreatMarker};

pub struct Location {
    pub name: &'static str,
    pub lat: f32,
    pub lon: f32,
}

pub const LOCATIONS: &[Location] = &[
    Location { name: "Washington", lat: 38.9, lon: -77.0 },
    Location { name: "Moscow",     lat: 55.8, lon:  37.6 },
    Location { name: "Beijing",    lat: 39.9, lon: 116.4 },
    Location { name: "London",     lat: 51.5, lon:  -0.1 },
    Location { name: "Paris",      lat: 48.9, lon:   2.3 },
    Location { name: "New Delhi",  lat: 28.6, lon:  77.2 },
    Location { name: "Pyongyang",  lat: 39.0, lon: 125.7 },
    Location { name: "Tehran",     lat: 35.7, lon:  51.4 },
    Location { name: "Islamabad",  lat: 33.7, lon:  73.0 },
];

// ponytail: ~20-30 point polylines per continent, (lat, lon) pairs
// canvas x=lon, y=lat with y_bounds [-90,90] so north is up
const CONTINENTS: &[&[(f32, f32)]] = &[
    // North America
    &[(70.0,-170.0),(72.0,-140.0),(70.0,-100.0),(65.0,-90.0),(60.0,-75.0),(50.0,-65.0),
      (45.0,-65.0),(40.0,-75.0),(30.0,-82.0),(25.0,-80.0),(30.0,-90.0),(28.0,-97.0),
      (20.0,-105.0),(15.0,-90.0),(15.0,-83.0),(10.0,-78.0),(8.0,-77.0)],
    // South America
    &[(12.0,-72.0),(10.0,-65.0),(5.0,-52.0),(0.0,-50.0),(-5.0,-35.0),(-15.0,-40.0),
      (-23.0,-42.0),(-33.0,-52.0),(-40.0,-63.0),(-50.0,-73.0),(-55.0,-68.0),(-55.0,-65.0),
      (-45.0,-65.0),(-35.0,-72.0),(-20.0,-70.0),(-5.0,-80.0),(5.0,-77.0)],
    // Europe
    &[(71.0,25.0),(70.0,30.0),(65.0,28.0),(60.0,30.0),(55.0,20.0),(54.0,14.0),
      (48.0,0.0),(43.0,-10.0),(36.0,-8.0),(36.0,0.0),(40.0,5.0),(43.0,15.0),
      (40.0,20.0),(38.0,25.0),(41.0,29.0),(45.0,30.0),(50.0,40.0),(55.0,40.0),
      (60.0,45.0),(65.0,40.0),(70.0,32.0)],
    // Africa
    &[(37.0,10.0),(35.0,0.0),(30.0,-10.0),(20.0,-17.0),(15.0,-17.0),(5.0,-5.0),
      (5.0,10.0),(0.0,10.0),(-5.0,12.0),(-15.0,15.0),(-25.0,30.0),(-35.0,25.0),
      (-35.0,18.0),(-30.0,30.0),(-15.0,40.0),(0.0,42.0),(10.0,50.0),(15.0,42.0),
      (20.0,40.0),(30.0,32.0),(32.0,35.0),(37.0,10.0)],
    // Asia
    &[(55.0,40.0),(50.0,55.0),(45.0,60.0),(40.0,70.0),(35.0,75.0),(25.0,65.0),
      (10.0,78.0),(8.0,80.0),(20.0,90.0),(22.0,97.0),(10.0,105.0),(1.0,104.0),
      (20.0,110.0),(30.0,120.0),(35.0,130.0),(40.0,130.0),(45.0,135.0),(50.0,140.0),
      (55.0,135.0),(60.0,150.0),(65.0,170.0),(70.0,180.0),(72.0,140.0),(70.0,100.0),
      (65.0,70.0),(60.0,50.0)],
    // Australia
    &[(-12.0,130.0),(-15.0,125.0),(-20.0,115.0),(-25.0,113.0),(-32.0,115.0),
      (-35.0,118.0),(-38.0,145.0),(-35.0,150.0),(-28.0,153.0),(-20.0,148.0),
      (-15.0,145.0),(-12.0,142.0),(-15.0,140.0),(-12.0,135.0)],
];

// kept for threat_overlay.rs which renders directly to Buffer
pub fn latlon_to_cell(lat: f32, lon: f32, area: Rect) -> (u16, u16) {
    let x_frac = (lon + 180.0) / 360.0;
    let y_frac = (90.0 - lat) / 180.0;
    let x = area.x + (x_frac * area.width as f32).clamp(0.0, (area.width - 1) as f32) as u16;
    let y = area.y + (y_frac * area.height as f32).clamp(0.0, (area.height - 1) as f32) as u16;
    (x, y)
}

fn threat_color(severity: ThreatLevel) -> Color {
    match severity {
        ThreatLevel::Low => Color::Cyan,       // radar ping
        ThreatLevel::Medium => Color::Yellow,  // threat
        ThreatLevel::High => Color::Red,       // attack
        ThreatLevel::Critical => Color::LightRed,
    }
}

pub struct WorldMap<'a> {
    pub missiles: &'a [MissileTrajectory],
    pub threats: &'a [ThreatMarker],
    pub tick: u64,
}

impl Widget for WorldMap<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let blink = (self.tick / 15) % 2 == 0;

        Canvas::default()
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                // continent outlines
                for continent in CONTINENTS {
                    for pair in continent.windows(2) {
                        ctx.draw(&CanvasLine {
                            x1: pair[0].1 as f64,
                            y1: pair[0].0 as f64,
                            x2: pair[1].1 as f64,
                            y2: pair[1].0 as f64,
                            color: Color::Green,
                        });
                    }
                }

                // city markers
                for loc in LOCATIONS {
                    let coords = [(loc.lon as f64, loc.lat as f64)];
                    ctx.draw(&Points { coords: &coords, color: Color::LightGreen });
                    ctx.print(
                        loc.lon as f64 + 2.0,
                        loc.lat as f64,
                        Span::styled(loc.name, Style::default().fg(Color::DarkGray)),
                    );
                }

                // missile trajectory lines + moving dot
                for m in self.missiles {
                    let p = m.progress.clamp(0.0, 1.0) as f64;
                    let (ox, oy) = (m.origin.1 as f64, m.origin.0 as f64);
                    let (tx, ty) = (m.target.1 as f64, m.target.0 as f64);

                    // full path line in dark gray
                    ctx.draw(&CanvasLine { x1: ox, y1: oy, x2: tx, y2: ty, color: Color::DarkGray });

                    // moving dot along lerp path
                    let cx = ox + (tx - ox) * p;
                    let cy = oy + (ty - oy) * p;
                    let dot_color = if p >= 1.0 { Color::LightRed } else { Color::Red };
                    ctx.draw(&Points { coords: &[(cx, cy)], color: dot_color });
                }

                // blinking threat icons: High/Critical blink, others steady
                for t in self.threats {
                    let visible = match t.severity {
                        ThreatLevel::Critical | ThreatLevel::High => blink,
                        _ => true,
                    };
                    if visible {
                        let color = threat_color(t.severity);
                        ctx.draw(&Points {
                            coords: &[(t.location.1 as f64, t.location.0 as f64)],
                            color,
                        });
                    }
                }
            })
            .render(area, buf);
    }
}
