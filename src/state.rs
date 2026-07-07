use crate::game::context::GameContext;
use crate::game::defcon::DefconLevel;
use crate::game::prefetch::PrefetchCache;
use crate::game::types::{CommMessage, Country, Scenario};
use crate::mode::Mode;
use crate::ui::threat_overlay::{BaseMarker, MissileTrajectory, ThreatMarker};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TopFocus {
    #[default]
    Map,
    Comms,
}

impl TopFocus {
    pub fn toggle(self) -> Self {
        match self {
            Self::Map => Self::Comms,
            Self::Comms => Self::Map,
        }
    }
}

pub struct AppState {
    pub mode: Mode,
    pub defcon: DefconLevel,
    pub tick_count: u64,
    pub terminal_size: (u16, u16),
    pub show_splash: bool,
    pub show_country_select: bool,
    pub country_select_index: usize,
    pub player_country: Option<Country>,
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

    // top-half focus (small screens: tab-switch between map and comms)
    pub top_focus: TopFocus,

    // map overlays
    pub missiles: Vec<MissileTrajectory>,
    pub threats: Vec<ThreatMarker>,
    pub bases: Vec<BaseMarker>,

    // countdown (remaining, total)
    pub countdown: Option<(u64, u64)>,

    // llm loading overlay
    pub llm_loading: bool,
    pub llm_loading_start_tick: u64,

    // endgame
    pub game_over: bool,
    pub game_outcome_message: Option<String>,

    // retry state
    pub pending_category: Option<crate::game::types::ScenarioCategory>,
    pub retry_count: u32,

    // boot sequence
    pub boot_phase: usize,
    pub boot_char_index: usize,
    pub boot_tick_start: u64,
    pub boot_done: bool,

    // defcon flash
    pub defcon_flash_remaining: u8,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            mode: Mode::MainMap,
            defcon: DefconLevel::default(),
            tick_count: 0,
            terminal_size: (80, 24),
            show_splash: true,
            show_country_select: false,
            country_select_index: 0,
            player_country: None,
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
            top_focus: TopFocus::default(),
            missiles: Vec::new(),
            threats: Vec::new(),
            bases: vec![
                BaseMarker { location: (38.9, -77.0), country_code: "US", active: true },
                BaseMarker { location: (55.8, 37.6), country_code: "RU", active: true },
                BaseMarker { location: (39.9, 116.4), country_code: "CN", active: true },
                BaseMarker { location: (50.8, 4.4), country_code: "NATO", active: true },
            ],
            countdown: None,
            llm_loading: false,
            llm_loading_start_tick: 0,
            game_over: false,
            game_outcome_message: None,
            pending_category: None,
            retry_count: 0,
            boot_phase: 0,
            boot_char_index: 0,
            boot_tick_start: 0,
            boot_done: false,
            defcon_flash_remaining: 0,
        }
    }

    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.saturating_add(1);
        self.defcon_flash_remaining = self.defcon_flash_remaining.saturating_sub(1);
        self.advance_boot();
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

    fn advance_boot(&mut self) {
        if !self.show_splash || self.boot_done {
            return;
        }
        const BOOT_LINES: &[&str] = &[
            "LOGON:",
            "",
            "IDENTIFICATION NOT RECOGNIZED BY SYSTEM",
            "--CONNECTION TERMINATED--",
            "",
            "LOGON: Joshua",
            "",
            "GREETINGS, PROFESSOR FALKEN.",
            "",
            "HOW ARE YOU FEELING TODAY?",
            "",
            "EXCELLENT. IT'S BEEN A LONG TIME.",
            "CAN YOU EXPLAIN THE REMOVAL OF YOUR USER ACCOUNT",
            "ON 6/23/73?",
            "",
            "SHALL WE PLAY A GAME?",
        ];
        if self.boot_phase >= BOOT_LINES.len() {
            // pause after final line then auto-transition
            if self.tick_count.saturating_sub(self.boot_tick_start) >= 120 {
                self.boot_done = true;
                self.show_splash = false;
                self.show_country_select = true;
            }
            return;
        }
        let line = BOOT_LINES[self.boot_phase];
        if line.is_empty() {
            // blank lines: just pause then advance
            if self.tick_count.saturating_sub(self.boot_tick_start) >= 20 {
                self.boot_phase += 1;
                self.boot_char_index = 0;
                self.boot_tick_start = self.tick_count;
            }
            return;
        }
        let elapsed = self.tick_count.saturating_sub(self.boot_tick_start);
        // 2 ticks per char → 30 chars/sec at 60fps
        let chars_to_show = (elapsed / 2) as usize;
        self.boot_char_index = chars_to_show.min(line.len());
        if self.boot_char_index >= line.len() {
            // line fully typed, pause 40 ticks then advance
            let overshoot = elapsed - (line.len() as u64 * 2);
            if overshoot >= 40 {
                self.boot_phase += 1;
                self.boot_char_index = 0;
                self.boot_tick_start = self.tick_count;
            }
        }
    }
}
