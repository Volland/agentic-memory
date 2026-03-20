/// Build a prompt that asks the LLM whether two entities refer to the same
/// real-world concept.
pub fn entity_resolution_prompt(entity_a: &str, entity_b: &str) -> String {
    format!(
        r#"You are an entity resolution expert for a knowledge graph. Determine whether these two entity labels refer to the same real-world entity.

Entity A: "{entity_a}"
Entity B: "{entity_b}"

## Resolution Criteria
Consider ALL of the following:
- **Aliases & nicknames**: "Bob" and "Robert Smith" could be the same person
- **Abbreviations**: "MIT" and "Massachusetts Institute of Technology"
- **Transliterations**: "München" and "Munich"
- **Partial vs full names**: "Dr. Chen" and "Alice Chen"
- **Role references**: "my boss" and "Carol" (if context links them)
- **Contextual equivalence**: "the startup" and "Acme Corp" (if context established)
- **Different specificity levels**: "New York" and "NYC" (same), "New York" and "New York City" (same), but "New York" (state) and "New York City" are DIFFERENT

## When to say YES (same_entity: true)
- The labels unambiguously refer to the same real-world thing
- One is clearly a variant/alias of the other
- Context makes the coreference certain

## When to say NO (same_entity: false)
- Labels refer to genuinely different entities even if similar names
- Ambiguity that cannot be resolved (prefer false over speculative merges)
- Different specificity levels that actually mean different things (state vs city)

## If YES — provide merge guidance:
- `canonical_label`: The best label to keep (prefer: most specific, most commonly used, proper noun over description)
- `merge_strategy`: "keep_a", "keep_b", or "merge" (combine attributes from both)

Respond with a JSON object:
{{
  "same_entity": true or false,
  "confidence": 0.0 to 1.0,
  "reasoning": "brief explanation",
  "canonical_label": "best label to use (only if same_entity is true)",
  "merge_strategy": "keep_a|keep_b|merge (only if same_entity is true)"
}}

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Build a system prompt for the entity resolution task.
pub fn entity_resolution_system() -> &'static str {
    "You are a precise entity resolution system for a knowledge graph. You determine whether two text labels refer to the same real-world entity. You favor precision over recall — if in doubt, keep entities separate. Always respond with valid JSON."
}
