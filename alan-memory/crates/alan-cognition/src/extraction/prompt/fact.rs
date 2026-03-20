/// Build the (system, user) prompt pair for fact extraction.
///
/// `entity_labels` lists entities already extracted so the LLM can reference them.
pub fn fact_extraction_prompt(text: &str, entity_labels: &[String]) -> (String, String) {
    let entities_ctx = if entity_labels.is_empty() {
        String::from("No entities have been extracted yet.")
    } else {
        format!("Already-extracted entities: {}", entity_labels.join(", "))
    };

    let system = r#"You are a fact extraction system. Your task is to identify declarative facts stated in conversation text.

A fact is a subject-predicate-object triple that can be verified as true or false.

For each fact, provide:
- "label": a short descriptive label
- "predicate": the relationship or property being stated
- "subject_label": the entity this fact is about (should match an extracted entity label)
- "object_label": the target entity if applicable, or null
- "certainty": 0.0–1.0 how certain the speaker seems
- "source_fragment": the exact text supporting this fact

Respond with ONLY a JSON array. No markdown, no commentary."#;

    let user = format!(
        "{entities_ctx}\n\n\
         Extract all facts from the following conversation:\n\n{text}\n\n\
         Return a JSON array where each element has: label, predicate, subject_label, object_label, certainty, source_fragment."
    );

    (system.to_string(), user)
}
