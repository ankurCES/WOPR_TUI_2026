use ratatui::style::{Color, Style};

const BRAILLE_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

pub fn braille_spinner(tick: u64) -> char {
    let idx = (tick / 8) as usize % BRAILLE_FRAMES.len();
    BRAILLE_FRAMES[idx]
}

pub struct TypewriterState {
    pub char_index: usize,
    pub complete: bool,
}

impl TypewriterState {
    pub fn new() -> Self {
        Self { char_index: 0, complete: false }
    }

    pub fn advance(&mut self, text: &str, chars_per_tick: usize) {
        let total = text.chars().count();
        self.char_index = (self.char_index + chars_per_tick).min(total);
        self.complete = self.char_index >= total;
    }

    pub fn visible_slice<'a>(&self, text: &'a str) -> &'a str {
        let byte_end = text
            .char_indices()
            .nth(self.char_index)
            .map(|(i, _)| i)
            .unwrap_or(text.len());
        &text[..byte_end]
    }

    pub fn reset(&mut self) {
        self.char_index = 0;
        self.complete = false;
    }
}

pub fn pulse_style(tick: u64, period: u64, bright: Color, dim: Color) -> Style {
    if (tick / period) % 2 == 0 {
        Style::default().fg(bright)
    } else {
        Style::default().fg(dim)
    }
}

pub struct RadarSweep {
    pub angle: f32,
    pub speed: f32,
}

impl RadarSweep {
    pub fn new(speed: f32) -> Self {
        Self { angle: 0.0, speed }
    }

    pub fn advance(&mut self) {
        self.angle = (self.angle + self.speed) % 360.0;
    }

    // ponytail: returns (dx, dy) offset for sweep line endpoint, scaled to radius
    pub fn endpoint(&self, radius: f32) -> (f32, f32) {
        let rad = self.angle.to_radians();
        (rad.cos() * radius, rad.sin() * radius)
    }
}
