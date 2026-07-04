use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioCategory {
    MilitaryConfrontation,
    CyberWarfare,
    NuclearBrinksmanship,
    DiplomaticCrisis,
    EconomicWarfare,
    IntelligenceOps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommPriority {
    Routine,
    Priority,
    Immediate,
    Flash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Country {
    Russia,
    China,
    USA,
    NATO,
    DPRK,
    Iran,
    India,
    Pakistan,
    Unknown,
}

impl std::fmt::Display for Country {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Russia => write!(f, "RU"),
            Self::China => write!(f, "CN"),
            Self::USA => write!(f, "US"),
            Self::NATO => write!(f, "NATO"),
            Self::DPRK => write!(f, "DPRK"),
            Self::Iran => write!(f, "IR"),
            Self::India => write!(f, "IN"),
            Self::Pakistan => write!(f, "PK"),
            Self::Unknown => write!(f, "??"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationStatus {
    Allied,
    Neutral,
    Tense,
    Hostile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiplAction {
    Sanction,
    Treaty,
    Expel,
    Mobilize,
    DeEscalate,
    Threaten,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameOutcome {
    Victory(String),
    Defeat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommMessage {
    pub origin: Country,
    pub native_text: String,
    pub english_translation: String,
    pub priority: CommPriority,
    pub timestamp: u64,
    pub garbled_mask: Vec<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerOption {
    pub id: u32,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub category: ScenarioCategory,
    pub threat_level: ThreatLevel,
    pub affected_regions: Vec<String>,
    pub player_options: Vec<PlayerOption>,
    pub comms: Vec<CommMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub turn: u32,
    pub scenario_id: u32,
    pub option_index: usize,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub location: (f32, f32),
    pub severity: ThreatLevel,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEvent {
    pub turn: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub economic_stability: f32,
    pub military_tension: f32,
    pub political_unrest: f32,
}

impl Default for WorldState {
    fn default() -> Self {
        Self { economic_stability: 0.7, military_tension: 0.3, political_unrest: 0.2 }
    }
}
