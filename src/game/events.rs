use super::types::*;

#[derive(Debug, Clone)]
pub enum GameEvent {
    ScenarioUpdate(Scenario),
    PlayerDecision { scenario_id: u32, option_index: usize },
    DefconChange(u8),
    CommReceived(CommMessage),
    ThreatDetected { location: (f32, f32), severity: ThreatLevel },
    MissileLaunch { origin: (f32, f32), target: (f32, f32) },
    DiplomaticAction { country: Country, action: DiplAction },
    GameOver(GameOutcome),
    BudgetWarning(u8),
}
