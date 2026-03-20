/// Build a prompt that asks the LLM to determine the relationship between
/// a new fact and an existing fact — with support for the supersede model.
pub fn contradiction_detection_prompt(fact_a: &str, fact_b: &str) -> String {
    format!(
        r#"You are a fact reconciliation expert for a knowledge graph that uses a NO-DELETE model. We never delete facts — instead, newer facts SUPERSEDE older ones.

Compare these two facts and determine their relationship:

New Fact: "{fact_a}"
Existing Fact: "{fact_b}"

## Relationship Types

### "confirms"
The new fact reinforces or provides additional evidence for the existing fact.
Example: Existing: "Alice works at Google" → New: "Alice is a Google engineer" → confirms

### "contradicts"
The new fact directly negates or is logically incompatible with the existing fact.
Example: Existing: "Alice works at Google" → New: "Alice works at Meta" → contradicts
IMPORTANT: A contradicting new fact should SUPERSEDE the old one. The old fact gets an expiration timestamp, and a Supersedes edge is created.

### "updates"
The new fact provides more specific, more recent, or more detailed information about the same topic. The old fact isn't wrong — it's just outdated or less precise.
Example: Existing: "Alice works at Google" → New: "Alice was promoted to Staff Engineer at Google" → updates
Example: Existing: "Bob lives in California" → New: "Bob lives in San Francisco" → updates (more specific)
The old fact should be SUPERSEDED by the more current/specific one.

### "unrelated"
The facts are about different topics or different aspects of the same entity.
Example: Existing: "Alice works at Google" → New: "Alice speaks French" → unrelated

## Response Fields

- `relationship`: One of "confirms", "contradicts", "updates", "unrelated"
- `confidence`: 0.0 to 1.0 — how certain you are about this relationship classification
- `reasoning`: Brief explanation of your reasoning
- `should_supersede`: true if the new fact should supersede (replace) the existing fact. True for "contradicts" and "updates". False for "confirms" and "unrelated".
- `supersede_reason`: If should_supersede is true, explain what changed (e.g., "Job change: Google → Meta", "More specific location info")
- `suggested_certainty_adjustment`: -1.0 to 1.0
  - Positive: reinforces existing fact (confirmation)
  - Negative: weakens existing fact (contradiction)
  - Zero: no change (unrelated)

Respond with a JSON object:
{{
  "relationship": "confirms|contradicts|updates|unrelated",
  "confidence": 0.0 to 1.0,
  "reasoning": "brief explanation",
  "should_supersede": true or false,
  "supersede_reason": "explanation or null",
  "suggested_certainty_adjustment": -1.0 to 1.0
}}

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Build a system prompt for the fact reconciliation task.
pub fn contradiction_detection_system() -> &'static str {
    "You are a precise fact reconciliation system for a knowledge graph that uses a NO-DELETE supersede model. You determine logical relationships between factual assertions and advise whether newer facts should supersede older ones. Always respond with valid JSON."
}

/// Build a prompt that asks the LLM to determine the relationship between
/// a new event and an existing event — detecting duplicates and status updates.
pub fn event_reconciliation_prompt(event_a: &str, event_b: &str) -> String {
    format!(
        r#"You are an event reconciliation expert for a knowledge graph with a NO-DELETE model.

Compare these two events and determine their relationship:

New Event: "{event_a}"
Existing Event: "{event_b}"

## Relationship Types

### "same_event"
Both descriptions refer to the exact same real-world occurrence, possibly described differently.
Example: Existing: "Alice moved to Berlin" → New: "Alice relocated to Berlin in March" → same_event (merge them, keep the more detailed version)

### "status_update"
The new event updates the status of the existing event (e.g., planned → occurred, ongoing → completed).
Example: Existing: "Team planning Q3 launch" (planned) → New: "Q3 launch happened successfully" (occurred) → status_update
The old event should be SUPERSEDED by the new status.

### "continuation"
The new event is a follow-up or continuation of the existing event, but they are distinct occurrences.
Example: Existing: "Started Kubernetes migration" → New: "Completed phase 2 of K8s migration" → continuation

### "unrelated"
Different events, no meaningful relationship.

Respond with a JSON object:
{{
  "relationship": "same_event|status_update|continuation|unrelated",
  "confidence": 0.0 to 1.0,
  "reasoning": "brief explanation",
  "should_supersede": true or false,
  "canonical_label": "best label to keep (only for same_event)",
  "supersede_reason": "what changed (only if should_supersede)"
}}

Respond ONLY with the JSON object, no additional text."#
    )
}

/// Build a system prompt for the event reconciliation task.
pub fn event_reconciliation_system() -> &'static str {
    "You are a precise event reconciliation system for a knowledge graph. You detect duplicate events, status transitions, and continuations. You use a NO-DELETE supersede model. Always respond with valid JSON."
}
