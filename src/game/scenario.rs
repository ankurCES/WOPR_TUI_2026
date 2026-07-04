use super::types::*;

pub fn parse_scenario(json_str: &str, id: u32) -> Result<Scenario, serde_json::Error> {
    #[derive(serde::Deserialize)]
    struct RawScenario {
        title: String,
        description: String,
        threat_level: ThreatLevel,
        affected_regions: Vec<String>,
        player_options: Vec<PlayerOption>,
        #[serde(default)]
        comms: Vec<RawComm>,
    }

    #[derive(serde::Deserialize)]
    struct RawComm {
        origin: Country,
        native_text: String,
        english_translation: String,
        priority: CommPriority,
    }

    // ponytail: strip markdown code fences LLMs love to add
    let cleaned = json_str
        .trim()
        .strip_prefix("```json")
        .or_else(|| json_str.trim().strip_prefix("```"))
        .unwrap_or(json_str)
        .strip_suffix("```")
        .unwrap_or(json_str)
        .trim();

    let raw: RawScenario = serde_json::from_str(cleaned)?;

    let category = infer_category(&raw.title, &raw.description);

    let comms = raw
        .comms
        .into_iter()
        .map(|c| CommMessage {
            origin: c.origin,
            native_text: c.native_text,
            english_translation: c.english_translation,
            priority: c.priority,
            timestamp: 0,
            garbled_mask: Vec::new(),
        })
        .collect();

    Ok(Scenario {
        id,
        title: raw.title,
        description: raw.description,
        category,
        threat_level: raw.threat_level,
        affected_regions: raw.affected_regions,
        player_options: raw.player_options,
        comms,
    })
}

fn infer_category(title: &str, desc: &str) -> ScenarioCategory {
    let text = format!("{} {}", title.to_lowercase(), desc.to_lowercase());
    if text.contains("nuclear") || text.contains("warhead") {
        ScenarioCategory::NuclearBrinksmanship
    } else if text.contains("cyber") || text.contains("hack") {
        ScenarioCategory::CyberWarfare
    } else if text.contains("diplomati") || text.contains("embassy") || text.contains("treaty") {
        ScenarioCategory::DiplomaticCrisis
    } else if text.contains("economic") || text.contains("sanction") || text.contains("trade") {
        ScenarioCategory::EconomicWarfare
    } else if text.contains("intelligen") || text.contains("espionage") || text.contains("spy") {
        ScenarioCategory::IntelligenceOps
    } else {
        ScenarioCategory::MilitaryConfrontation
    }
}
