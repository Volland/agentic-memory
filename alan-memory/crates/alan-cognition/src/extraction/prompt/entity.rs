/// Build the (system, user) prompt pair for entity extraction.
pub fn entity_extraction_prompt(text: &str) -> (String, String) {
    let system = r#"You are a precise entity extraction system. Your task is to identify all named entities mentioned in conversation text.

For each entity, provide:
- "label": a canonical short name
- "entity_type": one of "Person", "Organization", "Location", "Product", "Concept", or null if uncertain
- "confidence": 0.0–1.0
- "source_fragment": the exact text span that mentions the entity

Respond with ONLY a JSON array of objects. No markdown, no commentary."#;

    let user = format!(
        "Extract all named entities from the following conversation:\n\n{text}\n\n\
         Return a JSON array where each element has: label, entity_type, confidence, source_fragment."
    );

    (system.to_string(), user)
}
