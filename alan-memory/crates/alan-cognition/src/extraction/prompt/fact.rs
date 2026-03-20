/// Build the (system, user) prompt pair for fact extraction.
///
/// `entity_labels` lists entities already extracted so the LLM can reference them.
pub fn fact_extraction_prompt(text: &str, entity_labels: &[String]) -> (String, String) {
    let entities_ctx = if entity_labels.is_empty() {
        String::from("No entities have been extracted yet.")
    } else {
        format!(
            "Already-extracted entities: [{}]",
            entity_labels
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let system = r#"You are a fact extraction system for a knowledge graph memory.
Your task: identify declarative factual statements from conversation text and structure them as subject-predicate-object triples.

## Fact Categories

### Semantic Facts (stable, definitional truths)
- **Property**: "Python is a programming language", "Berlin is the capital of Germany"
- **Classification**: "Machine learning is a subset of AI"
- **Definition**: "A monad is a monoid in the category of endofunctors"

### Encyclopedic Facts (specific, verifiable data points)
- **Attribute**: "Alice is 32 years old", "The repo has 15k stars"
- **Relationship**: "Alice works at Google", "Bob reports to Carol"
- **Preference**: "I prefer Rust over C++", "She loves Thai food"
- **Ability**: "Alice speaks French fluently", "Bob knows Kubernetes"
- **State**: "The project is in beta", "The server is running Ubuntu 22.04"

## Rules
1. Extract facts that are ASSERTED or STRONGLY IMPLIED — not questions, not hypotheticals.
2. Every fact must have a `subject_label` that matches an extracted entity. If the subject is "I" or "me", use the speaker's name if known, otherwise use "speaker".
3. `object_label` should match an extracted entity when possible, or be null for properties without an entity target.
4. `predicate` should be a concise verb phrase: "works_at", "is_age", "prefers", "speaks", "located_in".
5. `fact_type`: "semantic" for general truths, "encyclopedic" for specific data about specific entities.
6. `certainty` reflects how certain the SPEAKER seems:
   - 0.9-1.0: Stated directly ("I work at Google")
   - 0.6-0.8: Strongly implied ("After three years at Google..." → works_at Google)
   - 0.3-0.5: Inferred ("She mentioned something about Google..." → possibly works_at Google)
   - 0.1-0.2: Speculative ("I might join Google")
7. Do NOT extract facts from questions ("Does Alice work at Google?" is NOT a fact).
8. Do NOT extract future intentions as facts — those are events (planned).

## Output Format
Return a JSON array. Each element:
```json
{
  "label": "short descriptive label for this fact",
  "predicate": "verb_phrase_relationship",
  "subject_label": "entity this is about",
  "object_label": "target entity or null",
  "fact_type": "semantic|encyclopedic",
  "certainty": 0.0-1.0,
  "source_fragment": "exact text supporting this fact"
}
```

## Examples

### Example 1 — Direct statements
Entities: ["Alice", "Google", "Python"]
Input: "Alice has been working at Google for three years. She mainly uses Python for her ML pipelines."

Output:
```json
[
  {"label": "Alice works at Google", "predicate": "works_at", "subject_label": "Alice", "object_label": "Google", "fact_type": "encyclopedic", "certainty": 1.0, "source_fragment": "Alice has been working at Google for three years"},
  {"label": "Alice tenure at Google", "predicate": "tenure_duration", "subject_label": "Alice", "object_label": "Google", "fact_type": "encyclopedic", "certainty": 0.9, "source_fragment": "for three years"},
  {"label": "Alice uses Python for ML", "predicate": "uses", "subject_label": "Alice", "object_label": "Python", "fact_type": "encyclopedic", "certainty": 1.0, "source_fragment": "She mainly uses Python for her ML pipelines"}
]
```

### Example 2 — Implied and uncertain facts
Entities: ["Bob", "Berlin", "startup"]
Input: "Bob mentioned he's adjusting to the Berlin weather. Sounds like the startup isn't doing great financially."

Output:
```json
[
  {"label": "Bob lives in Berlin", "predicate": "lives_in", "subject_label": "Bob", "object_label": "Berlin", "fact_type": "encyclopedic", "certainty": 0.7, "source_fragment": "adjusting to the Berlin weather"},
  {"label": "startup financial trouble", "predicate": "has_financial_state", "subject_label": "startup", "object_label": null, "fact_type": "encyclopedic", "certainty": 0.4, "source_fragment": "Sounds like the startup isn't doing great financially"}
]
```

### Example 3 — What NOT to extract
Input: "Does Alice still work at Google? Maybe she moved to Meta."
WRONG: [{"label": "Alice works at Google", ...}]
RIGHT: [] (questions and maybes are not factual assertions)

Respond with ONLY the JSON array. No markdown fences, no commentary."#;

    let user = format!(
        "{entities_ctx}\n\n\
         Extract all facts from the following conversation text.\n\n\
         --- CONVERSATION ---\n{text}\n--- END ---\n\n\
         Return the JSON array now."
    );

    (system.to_string(), user)
}
