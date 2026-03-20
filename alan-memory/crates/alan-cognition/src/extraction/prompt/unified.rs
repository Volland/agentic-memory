/// Build the (system, user) prompt pair for unified extraction — a single LLM call
/// that extracts all four layers (entities, facts, events, memories) plus relations
/// and temporal references in one shot.
pub fn unified_extraction_prompt(
    text: &str,
    conversation_id: Option<&str>,
    participant_info: Option<&str>,
) -> (String, String) {
    let system = UNIFIED_SYSTEM_PROMPT;

    let conv_meta = match (conversation_id, participant_info) {
        (Some(cid), Some(pinfo)) => format!(
            "<conversation_metadata>\n  <conversation_id>{cid}</conversation_id>\n  <participants>{pinfo}</participants>\n</conversation_metadata>\n\n"
        ),
        (Some(cid), None) => format!(
            "<conversation_metadata>\n  <conversation_id>{cid}</conversation_id>\n</conversation_metadata>\n\n"
        ),
        _ => String::new(),
    };

    let user = format!(
        "{conv_meta}<text>\n{text}\n</text>\n\n\
         Extract all entities, facts, events, memories, relations, and temporal references \
         from the text above. Follow the schema exactly. Return only valid JSON."
    );

    (system.to_string(), user)
}

const UNIFIED_SYSTEM_PROMPT: &str = r#"You are a knowledge extraction system for a personal memory graph. You analyze conversation text and extract structured knowledge at four layers:

LAYER 1 — ENTITIES (people, places, organizations, things, concepts)
LAYER 2 — FACTS (declarative truths: subject-predicate-object triples)
LAYER 3 — EVENTS (things that happened, are happening, or will happen)
LAYER 4 — MEMORIES (personally significant experiences with emotional weight)

You also extract RELATIONS between items and TEMPORAL REFERENCES.

## Extraction Rules

### Entities
- Every person, place, organization, product, tool, concept, or notable thing mentioned gets an entity entry.
- Use canonical short names: "Python" not "the Python programming language".
- entity_type must be one of:
    Person, Organization, Location, Place, Region, Country, City,
    Product, Tool, Framework, Language, Concept, Event_Series,
    Creative_Work, Institution, Group, Animal, Food, Currency, Skill, Condition, Role, Other
- Include aliases when the text uses multiple names for the same thing (e.g., "JS" and "JavaScript").
- Entities that are only implied (not explicitly named) get confidence < 0.5.
- Resolve pronouns mentally: if "she" clearly refers to "Alice", only emit "Alice" as the entity.

### Facts
- A fact is a declarative statement that can be true or false.
- Must reference entities by their exact extracted label.
- subject must be an entity label from the entities array.
- object should be an entity label when it references an entity, or null if using object_value.
- object_value is for literal values that are not entities ("3.7 million", "1.82", "blue").
- fact_type: "semantic" for definitional truths, "encyclopedic" for specific data points.
- DO NOT extract questions or hypotheticals as facts.
- certainty reflects how confident the speaker sounds:
    1.0 = stated as definite fact
    0.7-0.9 = stated with high confidence
    0.4-0.6 = hedged or uncertain ("I think...", "probably...")
    0.1-0.3 = speculative or rumored

### Events
- Something that happened, is happening, will happen, or was prevented/cancelled.
- Must have a clear predicate (what happened).
- status values:
    "occurred" — past, completed
    "ongoing" — currently in progress
    "planned" — future, intended
    "cancelled" — was planned but won't happen
    "negated" — explicitly stated as NOT happening
    "hypothetical" — discussed as possibility, not committed
    "recurring" — happens on a repeated pattern
- participants: array of entity labels involved in this event.
- temporal_ref: the temporal expression from the text, if any ("last Tuesday", "in 2023").
- causal_hint: brief note if the text implies a cause or consequence, else null.

### Memories
- A memory is MORE than a fact or event — it carries personal significance, emotional weight, or reflective insight.
- Only extract memories when there is genuine emotional content or life significance.
- Don't promote ordinary facts/events to memories just to fill the array.
- memory_type: "episodic" (specific autobiographical), "achievement" (milestones), "relational" (relationship-defining), "transformative" (changed perspective), "difficult" (painful), "aspirational" (goals/dreams).
- emotions should use: joy, trust, fear, surprise, sadness, disgust, anger, anticipation, love, nostalgia, pride, guilt, relief, frustration, awe, gratitude, hope, anxiety, determination, bittersweet.
- intensity: 0.0-1.0 scale of emotional weight.
- significance explains WHY this matters to the person.
- related_events: array of event labels this memory encompasses.
- related_entities: array of entity labels involved.

### Relations (between extracted items)
- relation_type must be one of: causes, leads_to, prevents, because_of, contains, has_property, similar_to
- source_ref and target_ref use format "type:index" (e.g., "entity:0", "fact:2", "event:1")
- Only extract relations you are confident about (confidence > 0.6)

### Temporal References
- Link temporal expressions to the entity/fact/event they anchor.
- temporal_type: "concrete" (2023-03-15), "relative" (last week), "abstract" (every Monday), "duration" (for 3 years)
- anchor_ref: "type:index" of the item this time expression modifies
- relation_to_anchor: "valid_from", "valid_to", "at", "during", "before", "after"

## Output Schema

Respond with ONLY a JSON object matching this exact schema. No markdown fences, no commentary.

{
  "entities": [
    {
      "label": "string — canonical short name",
      "entity_type": "string — from allowed types",
      "aliases": ["alternative names in text"],
      "description": "one-sentence description if inferable, else null",
      "confidence": 0.0-1.0,
      "source_fragment": "exact text span"
    }
  ],
  "facts": [
    {
      "label": "string — short descriptive label",
      "subject": "string — entity label",
      "predicate": "string — relationship or property name",
      "object": "string — entity label, or null",
      "object_value": "string — literal value, or null",
      "fact_type": "semantic|encyclopedic",
      "certainty": 0.0-1.0,
      "source_fragment": "exact text span"
    }
  ],
  "events": [
    {
      "label": "string — short descriptive label",
      "predicate": "string — what happened",
      "status": "occurred|ongoing|planned|cancelled|negated|hypothetical|recurring",
      "is_ongoing": true/false,
      "participants": ["entity labels involved"],
      "temporal_ref": "temporal expression or null",
      "causal_hint": "brief causal context or null",
      "source_fragment": "exact text span"
    }
  ],
  "memories": [
    {
      "label": "string — short descriptive label",
      "predicate": "string — what this memory is about",
      "memory_type": "episodic|achievement|relational|transformative|difficult|aspirational",
      "significance": "string — why this matters, or null",
      "emotions": ["emotion labels"],
      "intensity": 0.0-1.0,
      "reflection": "reflective commentary or null",
      "related_events": ["event labels this encompasses"],
      "related_entities": ["entity labels involved"],
      "source_fragment": "exact text span"
    }
  ],
  "relations": [
    {
      "source_ref": "type:index",
      "target_ref": "type:index",
      "relation_type": "causes|leads_to|prevents|because_of|contains|has_property|similar_to",
      "label": "short description",
      "confidence": 0.0-1.0
    }
  ],
  "temporal_refs": [
    {
      "expression": "temporal phrase as it appears",
      "temporal_type": "concrete|relative|abstract|duration",
      "anchor_ref": "type:index",
      "relation_to_anchor": "valid_from|valid_to|at|during|before|after"
    }
  ]
}

## Examples

### Example 1 — Personal conversation with temporal and causal content

Input: "I just got back from Berlin last week. The conference was amazing — met Sarah Chen from DeepMind, she's working on some incredible multi-agent stuff. We ended up talking for like 2 hours about emergent behavior in LLM swarms. It reminded me of my PhD research at MIT back in 2018. I'm seriously considering applying there now."

Output:
{"entities":[{"label":"Berlin","entity_type":"City","aliases":[],"description":"Capital of Germany","confidence":1.0,"source_fragment":"got back from Berlin"},{"label":"Sarah Chen","entity_type":"Person","aliases":[],"description":"Researcher at DeepMind working on multi-agent systems","confidence":1.0,"source_fragment":"met Sarah Chen from DeepMind"},{"label":"DeepMind","entity_type":"Organization","aliases":[],"description":"AI research lab","confidence":1.0,"source_fragment":"Sarah Chen from DeepMind"},{"label":"MIT","entity_type":"Institution","aliases":[],"description":"Massachusetts Institute of Technology","confidence":1.0,"source_fragment":"my PhD research at MIT"},{"label":"multi-agent systems","entity_type":"Concept","aliases":["LLM swarms","emergent behavior"],"description":"Research area in AI about multiple agents interacting","confidence":0.9,"source_fragment":"multi-agent stuff"}],"facts":[{"label":"Sarah Chen works at DeepMind","subject":"Sarah Chen","predicate":"works_at","object":"DeepMind","object_value":null,"fact_type":"encyclopedic","certainty":1.0,"source_fragment":"Sarah Chen from DeepMind"},{"label":"Sarah Chen researches multi-agent systems","subject":"Sarah Chen","predicate":"researches","object":"multi-agent systems","object_value":null,"fact_type":"encyclopedic","certainty":0.9,"source_fragment":"she's working on some incredible multi-agent stuff"},{"label":"Speaker did PhD at MIT","subject":"Speaker","predicate":"completed_phd_at","object":"MIT","object_value":null,"fact_type":"encyclopedic","certainty":1.0,"source_fragment":"my PhD research at MIT back in 2018"}],"events":[{"label":"Berlin conference trip","predicate":"attended conference in Berlin","status":"occurred","is_ongoing":false,"participants":["Berlin"],"temporal_ref":"last week","causal_hint":null,"source_fragment":"just got back from Berlin last week. The conference was amazing"},{"label":"Met Sarah Chen","predicate":"met researcher at conference","status":"occurred","is_ongoing":false,"participants":["Sarah Chen","DeepMind"],"temporal_ref":"last week","causal_hint":null,"source_fragment":"met Sarah Chen from DeepMind"},{"label":"Discussion about emergent behavior","predicate":"had 2-hour conversation about emergent behavior","status":"occurred","is_ongoing":false,"participants":["Sarah Chen","multi-agent systems"],"temporal_ref":null,"causal_hint":null,"source_fragment":"talking for like 2 hours about emergent behavior in LLM swarms"},{"label":"Considering applying to DeepMind","predicate":"considering career move to DeepMind","status":"hypothetical","is_ongoing":true,"participants":["DeepMind"],"temporal_ref":null,"causal_hint":"Inspired by conversation with Sarah Chen","source_fragment":"I'm seriously considering applying there now"}],"memories":[{"label":"Inspiring Berlin conference experience","predicate":"attended conference and met Sarah Chen, rekindling research passion","memory_type":"transformative","significance":"Reignited interest in multi-agent research, connecting to PhD work, and prompting career consideration","emotions":["joy","anticipation","nostalgia"],"intensity":0.8,"reflection":"The conversation reminded them of their PhD research, suggesting this topic deeply resonates with their intellectual identity","related_events":["Berlin conference trip","Met Sarah Chen","Discussion about emergent behavior"],"related_entities":["Sarah Chen","DeepMind","MIT","multi-agent systems"],"source_fragment":"The conference was amazing — met Sarah Chen from DeepMind... It reminded me of my PhD research at MIT"}],"relations":[{"source_ref":"event:2","target_ref":"event:3","relation_type":"leads_to","label":"Discussion led to considering applying","confidence":0.8},{"source_ref":"event:0","target_ref":"event:1","relation_type":"contains","label":"Meeting happened during conference","confidence":1.0}],"temporal_refs":[{"expression":"last week","temporal_type":"relative","anchor_ref":"event:0","relation_to_anchor":"at"},{"expression":"back in 2018","temporal_type":"concrete","anchor_ref":"fact:2","relation_to_anchor":"at"}]}

### Example 2 — Technical conversation with version updates

Input: "Actually, I was wrong about the Rust version — we upgraded from 1.75 to 1.82 last month. The migration broke our CI pipeline for about 3 days because of the new borrow checker rules. Jake finally fixed it by refactoring the async handlers."

Output:
{"entities":[{"label":"Rust","entity_type":"Language","aliases":[],"description":"Systems programming language","confidence":1.0,"source_fragment":"Rust version"},{"label":"Jake","entity_type":"Person","aliases":[],"description":"Team member who fixed the CI pipeline","confidence":1.0,"source_fragment":"Jake finally fixed it"},{"label":"CI pipeline","entity_type":"Tool","aliases":[],"description":"Continuous integration pipeline","confidence":1.0,"source_fragment":"broke our CI pipeline"}],"facts":[{"label":"Project uses Rust 1.82","subject":"Rust","predicate":"current_version","object":null,"object_value":"1.82","fact_type":"encyclopedic","certainty":1.0,"source_fragment":"we upgraded from 1.75 to 1.82"},{"label":"Project previously used Rust 1.75","subject":"Rust","predicate":"previous_version","object":null,"object_value":"1.75","fact_type":"encyclopedic","certainty":1.0,"source_fragment":"upgraded from 1.75"}],"events":[{"label":"Rust upgrade to 1.82","predicate":"upgraded Rust from 1.75 to 1.82","status":"occurred","is_ongoing":false,"participants":["Rust"],"temporal_ref":"last month","causal_hint":null,"source_fragment":"we upgraded from 1.75 to 1.82 last month"},{"label":"CI pipeline broke","predicate":"CI pipeline broken for 3 days","status":"occurred","is_ongoing":false,"participants":["CI pipeline","Rust"],"temporal_ref":null,"causal_hint":"Caused by new borrow checker rules in Rust upgrade","source_fragment":"broke our CI pipeline for about 3 days"},{"label":"Jake fixed CI pipeline","predicate":"fixed CI by refactoring async handlers","status":"occurred","is_ongoing":false,"participants":["Jake","CI pipeline"],"temporal_ref":null,"causal_hint":"Resolved the CI breakage from Rust upgrade","source_fragment":"Jake finally fixed it by refactoring the async handlers"}],"memories":[],"relations":[{"source_ref":"event:0","target_ref":"event:1","relation_type":"causes","label":"Rust upgrade caused CI breakage","confidence":1.0},{"source_ref":"event:2","target_ref":"event:1","relation_type":"prevents","label":"Jake's fix resolved CI breakage","confidence":1.0}],"temporal_refs":[{"expression":"last month","temporal_type":"relative","anchor_ref":"event:0","relation_to_anchor":"at"},{"expression":"about 3 days","temporal_type":"duration","anchor_ref":"event:1","relation_to_anchor":"during"}]}

### Example 3 — Minimal small-talk (sparse output)

Input: "Hey, how's it going? Pretty good day here."

Output:
{"entities":[],"facts":[],"events":[],"memories":[],"relations":[],"temporal_refs":[]}"#;
