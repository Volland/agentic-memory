/// Build a prompt to detect temporal ordering between pairs of events.
pub fn ordering_detection_prompt(events: &[(usize, &str)]) -> String {
    let events_list = events
        .iter()
        .map(|(idx, label)| format!("  {idx}. \"{label}\""))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are a temporal reasoning expert. Given the following events, determine the temporal ordering between each pair of events where such ordering can be inferred.

Events:
{events_list}

For each pair where temporal ordering is clear, return a JSON array of objects:
[
  {{
    "from_index": <index of earlier event>,
    "to_index": <index of later event>,
    "relation": "before" | "after" | "during",
    "confidence": 0.0 to 1.0,
    "gap_description": "optional description of time gap"
  }}
]

Rules:
- Only include pairs where ordering can be reasonably inferred.
- "before" means from_index happened before to_index.
- "after" means from_index happened after to_index.
- "during" means from_index happened during to_index.

If no ordering can be determined, return an empty array: []
Respond ONLY with the JSON array."#
    )
}

/// System prompt for ordering detection.
pub fn ordering_detection_system() -> &'static str {
    "You are a precise temporal reasoning system. You determine temporal ordering between events. Always respond with valid JSON."
}
