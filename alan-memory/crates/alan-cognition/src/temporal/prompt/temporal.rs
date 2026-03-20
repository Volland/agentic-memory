/// Build a prompt to extract temporal expressions from a text snippet and
/// the labels of known entities / events.
pub fn temporal_extraction_prompt(text: &str, entity_labels: &[&str]) -> String {
    let entities_list = if entity_labels.is_empty() {
        "None provided.".to_string()
    } else {
        entity_labels
            .iter()
            .map(|l| format!("- {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"You are a temporal information extraction expert. Given the following text and a list of known entities/events, extract all temporal expressions and link each to the relevant entity.

Text:
"{text}"

Known entities/events:
{entities_list}

For each temporal expression found, return a JSON array of objects:
[
  {{
    "expression": "the temporal phrase as it appears in text",
    "temporal_type": "concrete" | "abstract" | "relative",
    "anchor_entity_label": "label of the entity/event this time refers to",
    "relation_type": "valid_from" | "valid_to" | "at" | "during" | "before" | "after"
  }}
]

If no temporal expressions are found, return an empty array: []
Respond ONLY with the JSON array, no additional text."#
    )
}

/// System prompt for temporal extraction.
pub fn temporal_extraction_system() -> &'static str {
    "You are a precise temporal information extraction system. You identify time expressions in text and link them to entities. Always respond with valid JSON."
}
