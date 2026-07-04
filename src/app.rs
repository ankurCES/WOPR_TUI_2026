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
                                        self.start_game().await;
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
                                    self.submit_decision().await;
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

    async fn start_game(&mut self) {
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
        self.generate_scenario(ScenarioCategory::MilitaryConfrontation).await;
        self.state.llm_loading = false;
    }

    async fn generate_scenario(&mut self, category: ScenarioCategory) {
        let provider = llm::create_provider(&self.settings);

        let request = LlmRequest {
            system_prompt: WOPR_SYSTEM_PROMPT.to_string(),
            user_prompt: scenario_prompt(&self.state.game_context, category),
            context_json: self.state.game_context.to_llm_context(),
            temperature: self.settings.temperature,
            max_tokens: self.settings.max_tokens,
        };

        match provider.generate_boxed(&request).await {
            Ok(response) => {
                self.state.scenario_counter += 1;
                if let Ok(scenario) = parse_scenario(&response.content, self.state.scenario_counter) {
                    // add scenario comms
                    for comm in &scenario.comms {
                        let mut c = comm.clone();
                        c.timestamp = self.state.tick_count;
                        self.state.comms.push(c);
                    }

                    // add threats
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
            }
            Err(_) => {
                self.state.comms.push(make_comm(
                    Country::Unknown,
                    "▒▒▒ SIGNAL LOST ▒▒▒",
                    "COMMS ERROR — LLM provider unreachable",
                    CommPriority::Flash,
                    self.state.tick_count,
                    0.0,
                ));
            }
        }
    }

    async fn submit_decision(&mut self) {
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
            self.state.game_active = false;
            self.state.comms.push(make_comm(
                Country::USA,
                match &outcome {
                    crate::game::types::GameOutcome::Victory(msg) => msg.clone(),
                    crate::game::types::GameOutcome::Defeat(msg) => msg.clone(),
                },
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
        self.generate_scenario(cat).await;
        self.state.llm_loading = false;
    }

    fn apply_game_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::DefconChange(level) => {
                self.state.defcon = self.state.defcon.try_change(level);
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
                self.state.game_context.add_timeline(format!(
                    "{} — {:?}",
                    country, action
                ));
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
                _ => {}
            },
            AppEvent::GameEvent(_ge) => {}
            AppEvent::Quit => {}
        }
    }
}
