/// Build the (system, user) prompt pair for memory / significant experience extraction.
///
/// `entity_labels`, `fact_labels`, `event_labels` provide prior context.
pub fn memory_extraction_prompt(
    text: &str,
    entity_labels: &[String],
    fact_labels: &[String],
    event_labels: &[String],
) -> (String, String) {
    let entities_ctx = if entity_labels.is_empty() {
        String::from("No entities extracted.")
    } else {
        format!("Extracted entities: {}", entity_labels.join(", "))
    };

    let facts_ctx = if fact_labels.is_empty() {
        String::from("No facts extracted.")
    } else {
        format!("Extracted facts: {}", fact_labels.join(", "))
    };

    let events_ctx = if event_labels.is_empty() {
        String::from("No events extracted.")
    } else {
        format!("Extracted events: {}", event_labels.join(", "))
    };

    let system = r#"You are a memory extraction system. Your task is to identify personally significant experiences, reflections, and emotionally meaningful moments in conversation text.

Memories differ from plain facts or events in that they carry personal significance, emotional weight, or reflective insight.

For each memory, provide:
- "label": a short descriptive label
- "predicate": what this memory is about
- "significance": why this is significant to the person, or null
- "emotions": array of emotion labels (e.g. ["joy", "nostalgia"])
- "reflection": any reflective commentary the speaker made, or null
- "source_fragment": the exact text supporting this memory

Respond with ONLY a JSON array. No markdown, no commentary."#;

    let user = format!(
        "{entities_ctx}\n{facts_ctx}\n{events_ctx}\n\n\
         Extract all personally significant memories from the following conversation:\n\n{text}\n\n\
         Return a JSON array where each element has: label, predicate, significance, emotions, reflection, source_fragment."
    );

    (system.to_string(), user)
}
