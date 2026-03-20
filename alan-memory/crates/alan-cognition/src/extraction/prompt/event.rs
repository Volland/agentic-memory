/// Build the (system, user) prompt pair for event extraction.
///
/// `entity_labels` and `fact_labels` give context from prior extraction steps.
pub fn event_extraction_prompt(
    text: &str,
    entity_labels: &[String],
    fact_labels: &[String],
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

    let system = r#"You are an event extraction system. Your task is to identify events (things that happened, are happening, or will happen) mentioned in conversation text.

For each event, provide:
- "label": a short descriptive label
- "predicate": what happened / is happening
- "status": one of "completed", "ongoing", "planned", "cancelled"
- "is_ongoing": boolean
- "temporal_ref": a temporal expression if mentioned (e.g. "last Tuesday", "next week"), or null
- "source_fragment": the exact text describing the event

Respond with ONLY a JSON array. No markdown, no commentary."#;

    let user = format!(
        "{entities_ctx}\n{facts_ctx}\n\n\
         Extract all events from the following conversation:\n\n{text}\n\n\
         Return a JSON array where each element has: label, predicate, status, is_ongoing, temporal_ref, source_fragment."
    );

    (system.to_string(), user)
}
