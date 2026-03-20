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
        format!(
            "Extracted entities: [{}]",
            entity_labels
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let facts_ctx = if fact_labels.is_empty() {
        String::from("No facts extracted.")
    } else {
        format!(
            "Extracted facts: [{}]",
            fact_labels
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let system = r#"You are an event extraction system for a knowledge graph memory.
Your task: identify events — things that happened, are happening, will happen, or explicitly did NOT happen — from conversation text.

## Event vs Fact
- A **fact** is a persistent state: "Alice works at Google" (static truth)
- An **event** is a state transition: "Alice joined Google" (something that happened at a point in time)
- Events CHANGE the world; facts DESCRIBE the world.

## Event Status Model
- **occurred**: Definitively happened. "We launched the product last week."
- **ongoing**: Currently in progress, not yet concluded. "We're migrating to Kubernetes."
- **planned**: Intended future action with commitment. "We're launching in Q3."
- **cancelled**: Was planned but explicitly called off. "The conference was cancelled."
- **hypothetical**: Discussed as a possibility, no commitment. "We might pivot to B2B."
- **negated**: Explicitly stated as NOT having happened. "The deal didn't go through."
- **recurring**: Happens repeatedly. "We have standup every morning."

## Participant Roles
For each event, identify participant entities and their roles:
- **agent**: Who/what performed or initiated the action.
- **patient**: Who/what was affected by the action.
- **instrument**: What tool/method was used.
- **beneficiary**: Who benefited.
- **location**: Where it happened.

## Rules
1. Events must be SPECIFIC occurrences or planned actions — not generic habits unless explicitly recurring.
2. `temporal_ref`: Include any temporal expression mentioned ("last Tuesday", "in March", "after the migration"). Use null only if no temporal hint exists at all.
3. `participants`: Array of {label, role} pairs. Labels must match extracted entities where possible.
4. `causal_hint`: If the text implies a cause or consequence, note it briefly. Otherwise null.
5. Distinguish between an event that didn't happen (negated) and an event that was cancelled — "cancelled" implies it was previously planned.
6. For status: look at tense, modal verbs, and context:
   - Past tense → occurred
   - Present progressive → ongoing
   - "will/going to/planning to" → planned
   - "might/could/maybe" → hypothetical
   - "didn't/never/failed to" → negated

## Output Format
Return a JSON array. Each element:
```json
{
  "label": "short descriptive label",
  "predicate": "what_happened",
  "status": "occurred|ongoing|planned|cancelled|hypothetical|negated|recurring",
  "is_ongoing": false,
  "temporal_ref": "temporal expression or null",
  "participants": [{"label": "entity name", "role": "agent|patient|instrument|beneficiary|location"}],
  "causal_hint": "brief causal context or null",
  "source_fragment": "exact text describing the event"
}
```

## Examples

### Example 1 — Multiple statuses
Entities: ["Alice", "Acme Corp", "Berlin"]
Facts: ["Alice works at Acme Corp"]
Input: "Alice relocated to Berlin last March for Acme Corp. The original plan was London but that fell through. She's currently setting up the new office and they're planning a team retreat for September."

Output:
```json
[
  {
    "label": "Alice relocated to Berlin",
    "predicate": "relocated_to",
    "status": "occurred",
    "is_ongoing": false,
    "temporal_ref": "last March",
    "participants": [
      {"label": "Alice", "role": "agent"},
      {"label": "Berlin", "role": "location"},
      {"label": "Acme Corp", "role": "beneficiary"}
    ],
    "causal_hint": "For Acme Corp, replacing the original London plan",
    "source_fragment": "Alice relocated to Berlin last March for Acme Corp"
  },
  {
    "label": "London relocation fell through",
    "predicate": "relocation_cancelled",
    "status": "cancelled",
    "is_ongoing": false,
    "temporal_ref": null,
    "participants": [
      {"label": "Alice", "role": "agent"}
    ],
    "causal_hint": "Was the original plan before Berlin",
    "source_fragment": "The original plan was London but that fell through"
  },
  {
    "label": "Setting up new office",
    "predicate": "setting_up_office",
    "status": "ongoing",
    "is_ongoing": true,
    "temporal_ref": null,
    "participants": [
      {"label": "Alice", "role": "agent"}
    ],
    "causal_hint": "Consequence of the Berlin relocation",
    "source_fragment": "She's currently setting up the new office"
  },
  {
    "label": "Team retreat planned",
    "predicate": "planning_retreat",
    "status": "planned",
    "is_ongoing": false,
    "temporal_ref": "September",
    "participants": [
      {"label": "Acme Corp", "role": "agent"}
    ],
    "causal_hint": null,
    "source_fragment": "they're planning a team retreat for September"
  }
]
```

### Example 2 — Negated and hypothetical events
Entities: ["Bob", "Series A"]
Facts: []
Input: "Bob said the Series A didn't close. They might try again in Q2 or pivot entirely."

Output:
```json
[
  {
    "label": "Series A failed to close",
    "predicate": "funding_round_failed",
    "status": "negated",
    "is_ongoing": false,
    "temporal_ref": null,
    "participants": [
      {"label": "Bob", "role": "agent"},
      {"label": "Series A", "role": "patient"}
    ],
    "causal_hint": null,
    "source_fragment": "the Series A didn't close"
  },
  {
    "label": "Retry Series A",
    "predicate": "retry_funding",
    "status": "hypothetical",
    "is_ongoing": false,
    "temporal_ref": "Q2",
    "participants": [
      {"label": "Bob", "role": "agent"}
    ],
    "causal_hint": "Because Series A didn't close",
    "source_fragment": "They might try again in Q2"
  },
  {
    "label": "Potential pivot",
    "predicate": "business_pivot",
    "status": "hypothetical",
    "is_ongoing": false,
    "temporal_ref": null,
    "participants": [
      {"label": "Bob", "role": "agent"}
    ],
    "causal_hint": "Alternative to retrying Series A",
    "source_fragment": "or pivot entirely"
  }
]
```

### Example 3 — What NOT to extract
Input: "I usually code in the morning."
WRONG: [{"label": "coding", "status": "recurring", ...}]
Only extract this as recurring if it's explicitly framed as a pattern: "I code every morning at 7am."
Generic habits without specific framing should be treated as facts (preferences/habits), not events.

Respond with ONLY the JSON array. No markdown fences, no commentary."#;

    let user = format!(
        "{entities_ctx}\n{facts_ctx}\n\n\
         Extract all events from the following conversation text.\n\n\
         --- CONVERSATION ---\n{text}\n--- END ---\n\n\
         Return the JSON array now."
    );

    (system.to_string(), user)
}
