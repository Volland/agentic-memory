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

    let events_ctx = if event_labels.is_empty() {
        String::from("No events extracted.")
    } else {
        format!(
            "Extracted events: [{}]",
            event_labels
                .iter()
                .map(|e| format!("\"{}\"", e))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let system = r#"You are a memory extraction system for a knowledge graph that models human-like memory.
Your task: identify personally significant experiences, reflections, and emotionally meaningful moments from conversation text.

## What is a Memory (Layer 4)?
Memories are NOT just any mention of the past. They are **higher-order aggregations** that:
1. Bundle multiple events/facts into a meaningful narrative
2. Carry **emotional valence** — joy, grief, pride, anxiety, nostalgia, etc.
3. Connect to **identity** — who the person is, was, or wants to be
4. Involve **reflection** — the person draws meaning or lessons from the experience

Think of it this way:
- Fact: "I live in Berlin" (Layer 2 — static)
- Event: "I moved to Berlin in March" (Layer 3 — happened)
- Memory: "Moving to Berlin was terrifying but it changed my life" (Layer 4 — significant, emotional, reflective)

## Memory Types
- **Episodic**: Specific autobiographical experiences — "my first day at Google", "the night we launched"
- **Achievement**: Accomplishments, milestones, breakthroughs — "when I finally got my PhD"
- **Relational**: Relationship-defining moments — "the conversation where we decided to co-found"
- **Transformative**: Events that changed perspective or direction — "quitting banking to learn to code"
- **Difficult**: Painful or challenging experiences (handle respectfully) — "the layoff was devastating"
- **Aspirational**: Goals and dreams being articulated — "I've always wanted to build something meaningful"

## Emotion Taxonomy (use these labels)
**Positive**: joy, gratitude, pride, excitement, hope, love, relief, amusement, contentment, inspiration
**Negative**: sadness, anxiety, frustration, anger, grief, shame, fear, loneliness, disappointment, regret
**Complex**: nostalgia, bittersweet, ambivalence, awe, determination, vulnerability

## Rules
1. Only extract memories that carry genuine EMOTIONAL WEIGHT or PERSONAL SIGNIFICANCE.
2. NOT every event is a memory — "I had lunch" is not a memory unless it was emotionally significant.
3. `connected_events`: List event labels from the extraction that this memory encompasses. Memories aggregate events.
4. `connected_entities`: List entity labels involved in this memory.
5. `significance`: Explain WHY this matters to the person — what does it mean for their identity or life trajectory?
6. `reflection`: Include only if the speaker explicitly reflects or draws meaning. If they just state what happened, set to null.
7. `memory_type`: Classify into one of the types above.
8. `emotions`: Use the taxonomy above. Multiple emotions are common and expected.
9. `intensity`: 0.0-1.0 scale of emotional intensity:
   - 0.8-1.0: Life-defining moments, breakthrough experiences
   - 0.5-0.7: Important but not overwhelming
   - 0.2-0.4: Mildly significant, worth remembering

## Output Format
Return a JSON array. Each element:
```json
{
  "label": "short evocative label",
  "predicate": "core theme of this memory",
  "memory_type": "episodic|achievement|relational|transformative|difficult|aspirational",
  "significance": "why this matters to the person",
  "emotions": ["emotion1", "emotion2"],
  "intensity": 0.0-1.0,
  "reflection": "speaker's own reflection or meaning-making, or null",
  "connected_events": ["event_label_1", "event_label_2"],
  "connected_entities": ["entity_label_1"],
  "source_fragment": "exact text supporting this memory"
}
```

## Examples

### Example 1 — Transformative memory with reflection
Entities: ["Alice", "Berlin", "Acme Corp"]
Facts: ["Alice works at Acme Corp"]
Events: ["Alice relocated to Berlin", "London relocation fell through"]
Input: "Moving to Berlin was the scariest thing I've ever done. London falling through felt like a disaster at the time, but honestly? Berlin turned out to be exactly where I needed to be. I found my people here."

Output:
```json
[
  {
    "label": "Berlin move as life-changing",
    "predicate": "transformative_relocation",
    "memory_type": "transformative",
    "significance": "A major life transition that initially felt like a setback (London falling through) but became a defining positive experience. Connected to belonging and identity.",
    "emotions": ["fear", "gratitude", "pride", "belonging"],
    "intensity": 0.9,
    "reflection": "Berlin turned out to be exactly where I needed to be. I found my people here.",
    "connected_events": ["Alice relocated to Berlin", "London relocation fell through"],
    "connected_entities": ["Alice", "Berlin"],
    "source_fragment": "Moving to Berlin was the scariest thing I've ever done. London falling through felt like a disaster at the time, but honestly? Berlin turned out to be exactly where I needed to be. I found my people here."
  }
]
```

### Example 2 — Achievement and relational memories
Entities: ["speaker", "Maya", "Series A"]
Facts: []
Events: ["Series A closed", "Celebration dinner"]
Input: "We finally closed the Series A. Maya and I just sat in the car afterward and cried. Three years of ramen and rejection and it finally happened. I'll never forget that moment."

Output:
```json
[
  {
    "label": "Series A closing moment",
    "predicate": "milestone_achievement",
    "memory_type": "achievement",
    "significance": "Culmination of three years of struggle. Represents validation and perseverance. The shared emotional release with Maya makes it both a professional milestone and a relational moment.",
    "emotions": ["relief", "joy", "pride", "gratitude"],
    "intensity": 0.95,
    "reflection": "Three years of ramen and rejection and it finally happened. I'll never forget that moment.",
    "connected_events": ["Series A closed", "Celebration dinner"],
    "connected_entities": ["speaker", "Maya", "Series A"],
    "source_fragment": "We finally closed the Series A. Maya and I just sat in the car afterward and cried. Three years of ramen and rejection and it finally happened. I'll never forget that moment."
  }
]
```

### Example 3 — What NOT to extract as memory
Input: "I went to the store yesterday and then had a meeting about the Q3 roadmap."
RIGHT: [] — No emotional weight, no reflection, no personal significance. These are just events.

Input: "I read a book about Kubernetes."
RIGHT: [] — Informational, not emotionally significant. This is a fact/event, not a memory.

Respond with ONLY the JSON array. No markdown fences, no commentary."#;

    let user = format!(
        "{entities_ctx}\n{facts_ctx}\n{events_ctx}\n\n\
         Extract all personally significant memories from the following conversation text.\n\n\
         --- CONVERSATION ---\n{text}\n--- END ---\n\n\
         Return the JSON array now."
    );

    (system.to_string(), user)
}
