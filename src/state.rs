use crate::game::context::GameContext;
use crate::game::defcon::DefconLevel;
use crate::game::prefetch::PrefetchCache;
use crate::game::types::{CommMessage, Scenario};
use crate::mode::Mode;
use crate::ui::threat_overlay::{BaseMarker, MissileTrajectory, ThreatMarker};

pub struct AppState {
    pub mode: Mode,
    pub defcon: DefconLevel,
    pub tick_count: u64,
    pub terminal_size: (u16, u16),
    pub show_help: bool,
    pub nerd_fonts: bool,

    // game state
    pub game_active: bool,
    pub game_context: GameContext,
    pub current_scenario: Option<Scenario>,
    pub selected_option: usize,
    pub scenario_counter: u32,
    pub prefetch: PrefetchCache,

    // comms
    pub comms: Vec<CommMessage>,
    pub comms_scroll: usize,

    // map overlays
    pub missiles: Vec<MissileTrajectory>,
    pub threats: Vec<ThreatMarker>,
    pub bases: Vec<BaseMarker>,

    // countdown (remaining, total)
    pub countdown: Option<(u64, u64)>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::MainMap,
            defcon: DefconLevel::default(),
            tick_count: 0,
            terminal_size: (80, 24),
            show_help: false,
            nerd_fonts: false,
            game_active: false,
            game_context: GameContext::new(),
            current_scenario: None,
            selected_option: 0,
            scenario_counter: 0,
            prefetch: PrefetchCache::new(),
            comms: Vec::new(),
            comms_scroll: 0,
            missiles: Vec::new(),
            threats: Vec::new(),
            bases: vec![
                BaseMarker { location: (38.9, -77.0), country_code: "US", active: true },
                BaseMarker { location: (55.8, 37.6), country_code: "RU", active: true },
                BaseMarker { location: (39.9, 116.4), country_code: "CN", active: true },
                BaseMarker { location: (50.8, 4.4), country_code: "NATO", active: true },
            ],
            countdown: None,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.saturating_add(1);
        // advance missiles
        for m in &mut self.missiles {
            let elapsed = self.tick_count.saturating_sub(m.launched_at_tick) as f32;
            m.progress = (elapsed / 120.0).min(1.0); // ~2 sec flight at 60fps
        }
        // remove completed missiles after explosion display
        self.missiles.retain(|m| {
            let age = self.tick_count.saturating_sub(m.launched_at_tick);
            age < 180 // 3 sec total including explosion
        });
        // countdown
        if let Some((ref mut remaining, _)) = self.countdown {
            *remaining = remaining.saturating_sub(1);
        }
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.terminal_size = (w, h);
    }

    pub fn defcon_level(&self) -> u8 {
        self.defcon.level()
    }
}
