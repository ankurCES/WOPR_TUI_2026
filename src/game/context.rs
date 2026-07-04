use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub turn_number: u32,
    pub defcon_level: u8,
    pub active_scenarios: Vec<Scenario>,
    pub player_decisions: Vec<Decision>,
    pub diplomatic_status: HashMap<String, RelationStatus>,
    pub active_threats: Vec<Threat>,
    pub timeline: Vec<TimelineEvent>,
    pub world_state: WorldState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_country: Option<Country>,
}

impl GameContext {
    pub fn new() -> Self {
        let mut diplo = HashMap::new();
        diplo.insert("Russia".into(), RelationStatus::Tense);
        diplo.insert("China".into(), RelationStatus::Neutral);
        diplo.insert("NATO".into(), RelationStatus::Allied);
        diplo.insert("DPRK".into(), RelationStatus::Hostile);
        diplo.insert("Iran".into(), RelationStatus::Tense);

        Self {
            turn_number: 0,
            defcon_level: 5,
            active_scenarios: Vec::new(),
            player_decisions: Vec::new(),
            diplomatic_status: diplo,
            active_threats: Vec::new(),
            timeline: Vec::new(),
            world_state: WorldState::default(),
            player_country: None,
        }
    }

    pub fn to_llm_context(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    pub fn summary(&self) -> String {
        format!(
            "TURN {} | DEFCON {} | {} threats | {} decisions",
            self.turn_number,
            self.defcon_level,
            self.active_threats.len(),
            self.player_decisions.len(),
        )
    }

    pub fn record_decision(&mut self, scenario_id: u32, option_index: usize, label: String) {
        self.player_decisions.push(Decision {
            turn: self.turn_number,
            scenario_id,
            option_index,
            label,
        });
    }

    pub fn add_timeline(&mut self, desc: impl Into<String>) {
        self.timeline.push(TimelineEvent {
            turn: self.turn_number,
            description: desc.into(),
        });
    }

    pub fn advance_turn(&mut self) {
        self.turn_number += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_round_trips_json() {
        let ctx = GameContext::new();
        let json = serde_json::to_string(&ctx).unwrap();
        let back: GameContext = serde_json::from_str(&json).unwrap();
        assert_eq!(back.turn_number, ctx.turn_number);
        assert_eq!(back.defcon_level, ctx.defcon_level);
    }
}
