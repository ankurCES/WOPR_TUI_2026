use super::types::{CommMessage, CommPriority, Country};

pub fn garble(text: &str, ratio: f32) -> (String, Vec<bool>) {
    let chars: Vec<char> = text.chars().collect();
    let mut mask = vec![false; chars.len()];
    // ponytail: deterministic garble from char index, no rng dep
    for (i, _) in chars.iter().enumerate() {
        let hash = ((i as f32 * 7.31) % 1.0) < ratio;
        mask[i] = hash;
    }
    let garbled: String = chars
        .iter()
        .zip(mask.iter())
        .map(|(c, &m)| if m && !c.is_whitespace() { '\u{2592}' } else { *c })
        .collect();
    (garbled, mask)
}

pub fn priority_label(p: &CommPriority) -> &'static str {
    match p {
        CommPriority::Flash => "FLASH",
        CommPriority::Immediate => "IMMEDIATE",
        CommPriority::Priority => "PRIORITY",
        CommPriority::Routine => "ROUTINE",
    }
}

pub fn make_comm(
    origin: Country,
    native: impl Into<String>,
    english: impl Into<String>,
    priority: CommPriority,
    tick: u64,
    garble_ratio: f32,
) -> CommMessage {
    let native_text = native.into();
    let (_, garbled_mask) = garble(&native_text, garble_ratio);
    CommMessage {
        origin,
        native_text,
        english_translation: english.into(),
        priority,
        timestamp: tick,
        garbled_mask,
    }
}
