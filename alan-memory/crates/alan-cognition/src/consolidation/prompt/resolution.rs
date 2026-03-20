/// Build a prompt that asks the LLM whether two entities refer to the same
/// real-world concept.
pub fn entity_resolution_prompt(entity_a: &str, entity_b: &str) -> String {
    format!(
        r#"You are an entity resolution expert. Determine whether the following two entity labels refer to the same real-world concept, person, place, or thing.

Entity A: "{entity_a}"
Entity B: "{entity_b}"

Consider:
- Aliases, nicknames, abbreviations
- Different spellings or transliterations
- Partial vs full names
- Contextual equivalence

Respond with a JSON object:
{{
  "same_entity": true or false,
  "confidence": 0.0 to 1.0,
  "reasoning": "brief explanation"
}}

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Build a system prompt for the entity resolution task.
pub fn entity_resolution_system() -> &'static str {
    "You are a precise entity resolution system. You determine whether two text labels refer to the same real-world entity. Always respond with valid JSON."
}
