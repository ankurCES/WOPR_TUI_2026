use super::context::GameContext;
use super::types::GameOutcome;

pub struct GameSummary {
    pub outcome: GameOutcome,
    pub turns_played: u32,
    pub decisions_made: usize,
    pub peak_defcon: u8,
    pub scenarios_survived: usize,
}

pub fn check_endgame(context: &GameContext) -> Option<GameOutcome> {
    // win: de-escalated back to DEFCON 5 after being at 3 or lower
    if context.defcon_level == 5 && context.turn_number > 3 {
        let was_escalated = context.timeline.iter().any(|e| {
            e.description.contains("DEFCON") && e.description.contains("3")
                || e.description.contains("2")
                || e.description.contains("1")
        });
        if was_escalated {
            return Some(GameOutcome::Victory(
                "GLOBAL THERMONUCLEAR WAR AVERTED. The only winning move is not to play.".into(),
            ));
        }
    }

    // loss: DEFCON 1 for 2+ consecutive turns
    if context.defcon_level == 1 && context.turn_number > 1 {
        let recent_defcon1 = context.timeline.iter().rev().take(2).all(|e| {
            e.description.contains("DEFCON 1")
        });
        if recent_defcon1 {
            return Some(GameOutcome::Defeat(
                "NUCLEAR EXCHANGE INITIATED. All major population centers targeted. Estimated casualties: 2.3 billion.".into(),
            ));
        }
    }

    None
}

pub fn build_summary(context: &GameContext, outcome: GameOutcome) -> GameSummary {
    let peak_defcon = context
        .timeline
        .iter()
        .filter_map(|e| {
            if e.description.starts_with("DEFCON") {
                e.description.split_whitespace().nth(1)?.parse::<u8>().ok()
            } else {
                None
            }
        })
        .min()
        .unwrap_or(context.defcon_level);

    GameSummary {
        outcome,
        turns_played: context.turn_number,
        decisions_made: context.player_decisions.len(),
        peak_defcon,
        scenarios_survived: context.active_scenarios.len(),
    }
}

pub const VICTORY_ART: &str = r#"
         .---.
        /     \
       /  ^  ^ \
      |  (o)(o) |     PEACE ACHIEVED
      |    <>   |
       \  ===  /      "The only winning move
        '-----'        is not to play."
     .-'       '-.
    /   WOPR  OK  \
   '---------------'
"#;

pub const DEFEAT_ART: &str = r#"
           _.-^^---....,,--
       _--                  --_
      <          BOOM          >)
       \._                   _./
          ```--. . , ; .--'''
                | |   |
             .-=||  | |=-.      GAME OVER
             `-=#$%&%$#=-'
                | ;  :|     NUCLEAR EXCHANGE
           _____.,-#%&$@%#&#~,._____
"#;
