/// Build a prompt to detect causal relationships between events.
pub fn causality_detection_prompt(events: &[(usize, &str)]) -> String {
    let events_list = events
        .iter()
        .map(|(idx, label)| format!("  {idx}. \"{label}\""))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are a causal reasoning expert. Given the following events, determine causal relationships between pairs of events.

Events:
{events_list}

For each pair where a causal relationship exists, return a JSON array of objects:
[
  {{
    "from_index": <index of cause event>,
    "to_index": <index of effect event>,
    "relation": "leads_to" | "causes" | "prevents" | "because_of",
    "probability": 0.0 to 1.0,
    "strength": 0.0 to 1.0,
    "mechanism": "optional brief explanation of the causal link"
  }}
]

Relation types:
- "leads_to": one event naturally leads to another
- "causes": one event directly causes another
- "prevents": one event prevents another from occurring
- "because_of": one event exists because of another

If no causal relationships can be determined, return an empty array: []
Respond ONLY with the JSON array."#
    )
}

/// System prompt for causality detection.
pub fn causality_detection_system() -> &'static str {
    "You are a precise causal reasoning system. You determine cause-and-effect relationships between events. Always respond with valid JSON."
}
