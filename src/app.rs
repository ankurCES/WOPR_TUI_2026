use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{Event as CEvent, EventStream, KeyCode, KeyEventKind};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::config::Settings;
use crate::event::{create_channel, AppEvent};
use crate::game::comms::make_comm;
use crate::game::consequence::map_decision_to_events;
use crate::game::endgame::check_endgame;
use crate::game::events::GameEvent;
use crate::game::prompts::{scenario_prompt, WOPR_SYSTEM_PROMPT};
use crate::game::scenario::parse_scenario;
use crate::game::types::{CommPriority, Country, ScenarioCategory};
use crate::llm;
use crate::llm::types::LlmRequest;
use crate::state::AppState;
use crate::ui::country_select::COUNTRIES;
use crate::ui::render;
use crate::ui::threat_overlay::ThreatMarker;
use crate::view::compute_view;

pub struct App {
    state: AppState,
    tx: mpsc::Sender<AppEvent>,
    rx: mpsc::Receiver<AppEvent>,
    settings: Settings,
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = create_channel();
        let settings = Settings::load();
        Self { state: AppState::new(), tx, rx, settings }
    }

    pub async fn run(mut self) -> io::Result<()> {
        let mut terminal = crate::terminal::init_terminal()?;
        // detect nerd fonts
        let caps = crate::terminal::detect_capabilities();
        self.state.nerd_fonts = caps.nerd_fonts_likely;
        let result = self.event_loop(&mut terminal).await;
        crate::terminal::restore_terminal();
        result
    }

    async fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        let mut ticker = tokio::time::interval(Duration::from_millis(16));
        let mut events = EventStream::new();

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    let mut drained = 0;
                    while drained < 64 {
                        match self.rx.try_recv() {
                            Ok(AppEvent::Quit) => return Ok(()),
                            Ok(ev) => { self.handle_event(ev); drained += 1; }
                            Err(_) => break,
                        }
                    }
                    self.state.tick();
                    terminal.draw(|f| {
                        let view = compute_view(&self.state, f.area());
                        render(f, &view, &self.state);
                    })?;
                }
                maybe = events.next() => {
                    match maybe {
                        Some(Ok(CEvent::Key(k))) if k.kind == KeyEventKind::Press => {
                            if self.state.show_splash {
                                self.state.show_splash = false;
                                self.state.show_country_select = true;
                            } else if self.state.show_country_select {
                                match k.code {
                                    KeyCode::Up => {
                                        self.state.country_select_index =
                                            self.state.country_select_index.saturating_sub(1);
                                    }
                                    KeyCode::Down => {
                                        self.state.country_select_index =
                                            (self.state.country_select_index + 1).min(COUNTRIES.len() - 1);
                                    }
                                    KeyCode::Enter => {
                                        let (country, _) = COUNTRIES[self.state.country_select_index];
                                        self.state.player_country = Some(country);
                                        self.state.game_context.player_country = Some(country);
                                        self.state.show_country_select = false;
                                        self.start_game();
                                    }
                                    KeyCode::Char('q') => {
                                        let _ = self.tx.send(AppEvent::Quit).await;
                                    }
                                    _ => {}
                                }
                            } else if self.state.game_over {
                                match k.code {
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        self.restart_game();
                                    }
                                    KeyCode::Char('q') => {
                                        let _ = self.tx.send(AppEvent::Quit).await;
                                    }
                                    _ => {}
                                }
                            } else { match k.code {
                                KeyCode::Char('q') => {
                                    let _ = self.tx.send(AppEvent::Quit).await;
                                }
                                KeyCode::Enter if self.state.current_scenario.is_some() => {
                                    self.submit_decision();
                                }
                                KeyCode::Char(c @ '1'..='4') if self.state.current_scenario.is_some() => {
                                    let idx = (c as usize) - ('1' as usize);
                                    if let Some(ref s) = self.state.current_scenario {
                                        if idx < s.player_options.len() {
                                            self.state.selected_option = idx;
                                        }
                                    }
                                }
                                KeyCode::Up if self.state.current_scenario.is_some() => {
                                    self.state.selected_option = self.state.selected_option.saturating_sub(1);
                                }
                                KeyCode::Down if self.state.current_scenario.is_some() => {
                                    if let Some(ref s) = self.state.current_scenario {
                                        self.state.selected_option = (self.state.selected_option + 1).min(s.player_options.len().saturating_sub(1));
                                    }
                                }
                                _ => {
                                    let _ = self.tx.send(AppEvent::Input(k)).await;
                                }
                            } }
                        }
                        Some(Ok(CEvent::Resize(w, h))) => {
                            let _ = self.tx.send(AppEvent::Resize(w, h)).await;
                        }
                        None => return Ok(()),
                        _ => {}
                    }
                }
            }
        }
    }

    fn restart_game(&mut self) {
        let nerd_fonts = self.state.nerd_fonts;
        self.state = AppState::new();
        self.state.nerd_fonts = nerd_fonts;
        self.state.show_country_select = true;
    }

    fn start_game(&mut self) {
        self.state.game_active = true;
        self.state.game_context.defcon_level = 5;

        let player_country = self.state.player_country.unwrap_or(Country::USA);
        let nation = player_country.full_name();
        self.state.comms.push(make_comm(
            player_country,
            format!("WOPR ONLINE. ADVISING {} COMMAND. SIMULATION INITIATED.", nation),
            format!("WOPR ONLINE. ADVISING {} COMMAND. SIMULATION INITIATED.", nation),
            CommPriority::Flash,
            self.state.tick_count,
            0.0,
        ));

        self.state.llm_loading = true;
        self.state.llm_loading_start_tick = self.state.tick_count;
        self.spawn_scenario(ScenarioCategory::MilitaryConfrontation);
    }

    fn spawn_scenario(&mut self, category: ScenarioCategory) {
        self.state.pending_category = Some(category);
        self.state.retry_count = 0;
        self.do_spawn_scenario(category);
    }

    fn retry_scenario(&mut self) {
        let category = match self.state.pending_category {
            Some(c) => c,
            None => return,
        };
        self.state.retry_count += 1;
        if self.state.retry_count > 3 {
            self.state.llm_loading = false;
            self.state.comms.push(make_comm(
                Country::Unknown,
                "▒▒▒ COMMS FAILURE — ALL RETRIES EXHAUSTED ▒▒▒",
                "LLM provider unreachable after 4 attempts",
                CommPriority::Flash,
                self.state.tick_count,
                0.0,
            ));
            return;
        }
        self.state.llm_loading = true;
        self.state.llm_loading_start_tick = self.state.tick_count;
        self.do_spawn_scenario(category);
    }

    fn do_spawn_scenario(&self, category: ScenarioCategory) {
        let tx = self.tx.clone();
        let settings = self.settings.clone();
        let context = self.state.game_context.clone();
        let retry = self.state.retry_count;
        tokio::spawn(async move {
            // exponential backoff: 0s, 1s, 2s, 4s
            if retry > 0 {
                tokio::time::sleep(Duration::from_secs(1 << (retry - 1))).await;
            }
            let provider = llm::create_provider(&settings);
            let request = LlmRequest {
                system_prompt: WOPR_SYSTEM_PROMPT.to_string(),
                user_prompt: scenario_prompt(&context, category),
                context_json: context.to_llm_context(),
                temperature: settings.temperature,
                max_tokens: settings.max_tokens,
            };
            match provider.generate_boxed(&request).await {
                Ok(response) => { let _ = tx.send(AppEvent::ScenarioReady(response.content)).await; }
                Err(_) => { let _ = tx.send(AppEvent::ScenarioFailed).await; }
            }
        });
    }

    fn submit_decision(&mut self) {
        let scenario = match self.state.current_scenario.take() {
            Some(s) => s,
            None => return,
        };

        let option_idx = self.state.selected_option;
        let option_label = scenario
            .player_options
            .get(option_idx)
            .map(|o| o.label.clone())
            .unwrap_or_default();

        // record decision
        self.state.game_context.record_decision(scenario.id, option_idx, option_label.clone());
        self.state.game_context.add_timeline(format!(
            "DEFCON {} — Player chose: {}",
            self.state.defcon.level(),
            option_label
        ));

        // generate consequences
        let events = map_decision_to_events(&scenario, option_idx, &self.state.game_context);
        for event in events {
            self.apply_game_event(event);
        }

        self.state.game_context.advance_turn();
        self.state.game_context.defcon_level = self.state.defcon.level();

        // check endgame
        if let Some(outcome) = check_endgame(&self.state.game_context) {
            let msg = match &outcome {
                crate::game::types::GameOutcome::Victory(m) => m.clone(),
                crate::game::types::GameOutcome::Defeat(m) => m.clone(),
            };
            self.state.game_active = false;
            self.state.game_over = true;
            self.state.game_outcome_message = Some(msg.clone());
            self.state.comms.push(make_comm(
                Country::USA,
                &msg,
                "GAME OVER",
                CommPriority::Flash,
                self.state.tick_count,
                0.0,
            ));
            return;
        }

        // next scenario — cycle categories
        let categories = [
            ScenarioCategory::CyberWarfare,
            ScenarioCategory::DiplomaticCrisis,
            ScenarioCategory::NuclearBrinksmanship,
            ScenarioCategory::EconomicWarfare,
            ScenarioCategory::IntelligenceOps,
            ScenarioCategory::MilitaryConfrontation,
        ];
        let cat = categories[self.state.game_context.turn_number as usize % categories.len()];
        self.state.llm_loading = true;
        self.state.llm_loading_start_tick = self.state.tick_count;
        self.spawn_scenario(cat);
    }

    fn apply_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::DefconChange(level) => {
                self.state.defcon = self.state.defcon.try_change(level);
                self.state.defcon_flash_remaining = 30;
            }
            GameEvent::CommReceived(mut comm) => {
                comm.timestamp = self.state.tick_count;
                self.state.comms.push(comm);
            }
            GameEvent::ThreatDetected { location, severity } => {
                self.state.threats.push(ThreatMarker { location, severity });
            }
            GameEvent::MissileLaunch { origin, target } => {
                self.state.missiles.push(crate::ui::threat_overlay::MissileTrajectory {
                    origin,
                    target,
                    progress: 0.0,
                    launched_at_tick: self.state.tick_count,
                });
            }
            GameEvent::PlayerDecision { .. } => {}
            GameEvent::DiplomaticAction { country, action } => {
                let status = match action {
                    crate::game::types::DiplAction::Treaty | crate::game::types::DiplAction::DeEscalate => {
                        crate::game::types::RelationStatus::Neutral
                    }
                    crate::game::types::DiplAction::Threaten | crate::game::types::DiplAction::Mobilize => {
                        crate::game::types::RelationStatus::Hostile
                    }
                    crate::game::types::DiplAction::Sanction | crate::game::types::DiplAction::Expel => {
                        crate::game::types::RelationStatus::Tense
                    }
                };
                self.state.game_context.diplomatic_status.insert(country.full_name().to_string(), status);
                self.state.game_context.add_timeline(format!("{} — {:?}", country, action));
            }
            GameEvent::GameOver(_) => {
                self.state.game_active = false;
            }
            GameEvent::BudgetWarning(pct) => {
                self.state.comms.push(make_comm(
                    Country::USA,
                    format!("⚠ COMMS BUDGET {}% USED", pct),
                    format!("Token budget at {}% — approaching limit", pct),
                    CommPriority::Immediate,
                    self.state.tick_count,
                    0.0,
                ));
            }
            GameEvent::WorldStateUpdate { economic, military, political } => {
                let ws = &mut self.state.game_context.world_state;
                ws.economic_stability = (ws.economic_stability + economic).clamp(0.0, 1.0);
                ws.military_tension = (ws.military_tension + military).clamp(0.0, 1.0);
                ws.political_unrest = (ws.political_unrest + political).clamp(0.0, 1.0);
            }
            GameEvent::ScenarioUpdate(_) => {}
        }
    }

    fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Tick => self.state.tick(),
            AppEvent::Resize(w, h) => self.state.resize(w, h),
            AppEvent::Input(_) if self.state.show_help => self.state.show_help = false,
            AppEvent::Input(k) => match k.code {
                KeyCode::Tab => self.state.mode = self.state.mode.next(),
                KeyCode::BackTab => self.state.mode = self.state.mode.prev(),
                KeyCode::Char('?') => self.state.show_help = true,
                KeyCode::Left | KeyCode::Right => {
                    self.state.top_focus = self.state.top_focus.toggle();
                }
                KeyCode::Char('k') | KeyCode::Up
                    if self.state.mode == crate::mode::Mode::Comms
                        || self.state.top_focus == crate::state::TopFocus::Comms =>
                {
                    self.state.comms_scroll = self.state.comms_scroll.saturating_sub(1);
                }
                KeyCode::Char('j') | KeyCode::Down
                    if self.state.mode == crate::mode::Mode::Comms
                        || self.state.top_focus == crate::state::TopFocus::Comms =>
                {
                    self.state.comms_scroll = (self.state.comms_scroll + 1)
                        .min(self.state.comms.len().saturating_sub(1));
                }
                _ => {}
            },
            AppEvent::ScenarioReady(content) => {
                self.state.scenario_counter += 1;
                match parse_scenario(&content, self.state.scenario_counter) {
                    Ok(scenario) => {
                        self.state.llm_loading = false;
                        self.state.pending_category = None;
                        self.state.retry_count = 0;
                        for comm in &scenario.comms {
                            let mut c = comm.clone();
                            c.timestamp = self.state.tick_count;
                            self.state.comms.push(c);
                        }
                        for region in &scenario.affected_regions {
                            self.state.threats.push(ThreatMarker {
                                location: crate::game::consequence::region_to_coords(region),
                                severity: scenario.threat_level,
                            });
                        }
                        self.state.selected_option = 0;
                        self.state.current_scenario = Some(scenario);
                        self.state.mode = crate::mode::Mode::Scenario;
                    }
                    Err(_) => {
                        self.state.comms.push(make_comm(
                            Country::Unknown,
                            "▒▒▒ GARBLED SIGNAL — RETRYING ▒▒▒",
                            "Scenario parse failed, retrying...",
                            CommPriority::Immediate,
                            self.state.tick_count,
                            0.3,
                        ));
                        self.retry_scenario();
                    }
                }
            }
            AppEvent::ScenarioFailed => {
                self.state.comms.push(make_comm(
                    Country::Unknown,
                    "▒▒▒ SIGNAL LOST — RETRYING ▒▒▒",
                    "LLM provider error, retrying...",
                    CommPriority::Immediate,
                    self.state.tick_count,
                    0.0,
                ));
                self.retry_scenario();
            }
            AppEvent::GameEvent(_ge) => {}
            AppEvent::Quit => {}
        }
    }
}
