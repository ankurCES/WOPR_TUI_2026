use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefconLevel(u8);

impl DefconLevel {
    pub fn new(level: u8) -> Self {
        Self(level.clamp(1, 5))
    }

    pub fn level(&self) -> u8 {
        self.0
    }

    pub fn escalate(&self) -> Self {
        Self::new(self.0.saturating_sub(1))
    }

    pub fn deescalate(&self) -> Self {
        Self::new(self.0.saturating_add(1))
    }

    // ponytail: ±1 per turn enforced here, caller just says direction
    pub fn try_change(&self, target: u8) -> Self {
        let target = target.clamp(1, 5);
        if target < self.0 {
            self.escalate()
        } else if target > self.0 {
            self.deescalate()
        } else {
            *self
        }
    }

    pub fn color(&self) -> Color {
        match self.0 {
            5 => Color::Green,
            4 => Color::Cyan,
            3 => Color::Yellow,
            2 => Color::Magenta,
            _ => Color::Red,
        }
    }

    pub fn label(&self) -> &'static str {
        match self.0 {
            5 => "FADE OUT",
            4 => "DOUBLE TAKE",
            3 => "ROUND HOUSE",
            2 => "FAST PACE",
            _ => "COCKED PISTOL",
        }
    }

    pub fn description(&self) -> &'static str {
        match self.0 {
            5 => "PEACE — Normal readiness",
            4 => "INCREASED READINESS — Intelligence watch",
            3 => "AIR FORCE READY — Forces on standby",
            2 => "ARMED FORCES READY — Mobilization imminent",
            _ => "NUCLEAR WAR IMMINENT — Maximum force readiness",
        }
    }
}

impl Default for DefconLevel {
    fn default() -> Self {
        Self(5)
    }
}
