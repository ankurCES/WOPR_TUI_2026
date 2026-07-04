use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct Location {
    pub name: &'static str,
    pub lat: f32,
    pub lon: f32,
}

pub const LOCATIONS: &[Location] = &[
    Location { name: "Washington", lat: 38.9, lon: -77.0 },
    Location { name: "Moscow", lat: 55.8, lon: 37.6 },
    Location { name: "Beijing", lat: 39.9, lon: 116.4 },
    Location { name: "London", lat: 51.5, lon: -0.1 },
    Location { name: "Pyongyang", lat: 39.0, lon: 125.7 },
    Location { name: "Tehran", lat: 35.7, lon: 51.4 },
    Location { name: "Brussels", lat: 50.8, lon: 4.4 },
    Location { name: "New Delhi", lat: 28.6, lon: 77.2 },
    Location { name: "Islamabad", lat: 33.7, lon: 73.0 },
];

pub fn latlon_to_cell(lat: f32, lon: f32, area: Rect) -> (u16, u16) {
    let x_frac = (lon + 180.0) / 360.0;
    let y_frac = (90.0 - lat) / 180.0;
    let x = area.x + (x_frac * area.width as f32).clamp(0.0, (area.width - 1) as f32) as u16;
    let y = area.y + (y_frac * area.height as f32).clamp(0.0, (area.height - 1) as f32) as u16;
    (x, y)
}

// ponytail: simplified continental outlines as (lat, lon) polylines
// each continent is a separate slice — rough but recognizable at 80x40
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

pub struct WorldMap;

impl Widget for WorldMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let map_style = Style::default().fg(Color::Green);
        let marker_style = Style::default().fg(Color::LightGreen);
        let label_style = Style::default().fg(Color::DarkGray);

        // draw continents
        for continent in CONTINENTS {
            for pair in continent.windows(2) {
                let (x1, y1) = latlon_to_cell(pair[0].0, pair[0].1, area);
                let (x2, y2) = latlon_to_cell(pair[1].0, pair[1].1, area);
                draw_line(buf, x1, y1, x2, y2, '.', map_style, area);
            }
        }

        // draw locations
        for loc in LOCATIONS {
            let (x, y) = latlon_to_cell(loc.lat, loc.lon, area);
            if area.contains((x, y).into()) {
                buf[(x, y)].set_char('●').set_style(marker_style);
                // label to the right if room
                let label_x = x + 1;
                if label_x + loc.name.len() as u16 <= area.right() {
                    buf.set_string(label_x, y, loc.name, label_style);
                }
            }
        }
    }
}

fn draw_line(buf: &mut Buffer, x1: u16, y1: u16, x2: u16, y2: u16, ch: char, style: Style, clip: Rect) {
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();
    let sx: i32 = if x1 < x2 { 1 } else { -1 };
    let sy: i32 = if y1 < y2 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x1 as i32;
    let mut cy = y1 as i32;

    loop {
        if cx >= clip.x as i32
            && cx < clip.right() as i32
            && cy >= clip.y as i32
            && cy < clip.bottom() as i32
        {
            buf[(cx as u16, cy as u16)].set_char(ch).set_style(style);
        }
        if cx == x2 as i32 && cy == y2 as i32 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
}
