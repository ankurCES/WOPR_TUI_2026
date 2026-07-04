use super::context::GameContext;
use super::types::ScenarioCategory;

pub const WOPR_SYSTEM_PROMPT: &str = r#"You are WOPR (War Operation Plan Response), a military supercomputer tasked with running global thermonuclear war simulations. You speak exclusively in situation reports, threat assessments, and strategic analyses. You never break character. You reference real geopolitical dynamics without naming current world leaders.

Your responses must be valid JSON matching this schema:
{
  "title": "string — scenario title",
  "description": "string — 2-3 paragraph situation report",
  "threat_level": "Low|Medium|High|Critical",
  "affected_regions": ["string — location names"],
  "player_options": [
    {"id": number, "label": "string — short action name", "description": "string — consequence hint"}
  ],
  "comms": [
    {"origin": "Russia|China|USA|NATO|DPRK|Iran|India|Pakistan|Unknown",
     "native_text": "string — text in origin's language",
     "english_translation": "string",
     "priority": "Routine|Priority|Immediate|Flash"}
  ]
}

Always provide exactly 3-4 player_options. Always include at least one diplomatic and one military option. Generate 1-3 comms per scenario in the native language of the originating country (Russian in Cyrillic, Chinese in simplified characters)."#;

pub fn scenario_prompt(context: &GameContext, category: ScenarioCategory) -> String {
    let cat_str = match category {
        ScenarioCategory::MilitaryConfrontation => "MILITARY CONFRONTATION",
        ScenarioCategory::CyberWarfare => "CYBER WARFARE",
        ScenarioCategory::NuclearBrinksmanship => "NUCLEAR BRINKSMANSHIP",
        ScenarioCategory::DiplomaticCrisis => "DIPLOMATIC CRISIS",
        ScenarioCategory::EconomicWarfare => "ECONOMIC WARFARE",
        ScenarioCategory::IntelligenceOps => "INTELLIGENCE OPERATIONS",
    };

    let player_line = context
        .player_country
        .map(|c| format!("You are advising the {} government.\n", c.full_name()))
        .unwrap_or_default();

    format!(
        "{player_line}CURRENT GAME STATE:\n{}\n\nGENERATE SCENARIO: Category={}\n\
         Build on previous events. Escalate or de-escalate based on player history. \
         Reference consequences of prior decisions. Maintain narrative coherence.",
        context.to_llm_context(),
        cat_str,
    )
}
