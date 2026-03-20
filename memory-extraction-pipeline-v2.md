# Memory Extraction & Consolidation Pipeline v2

## Design Philosophy

The pipeline follows a **stratified extraction** model aligned with our ontology layers:
- **Layer 1**: Entities (people, places, things, concepts)
- **Layer 2**: Facts (semantic/encyclopedic knowledge — stable truths)
- **Layer 3**: Events (things that happen or don't happen — temporal)
- **Layer 4**: Memories (emotionally significant aggregations of events, tied to identity)

We never delete — we **expire** and **supersede**.

---

## Pipeline Architecture

### Phase 1: Extraction (Reactive — per ingestion)

```
Input: raw message chunk
    │
    ├─ Step 1: Coreference Resolution & Normalization
    │   └─ Resolve pronouns, anaphora, merge co-referent mentions
    │
    ├─ Step 2: Entity Extraction (parallel NER + LLM)
    │   └─ People, Organizations, Locations, Places, Things, Concepts, Time expressions
    │
    ├─ Step 3: Fact Extraction (grounded on entities)
    │   ├─ Semantic facts (definitions, properties, classifications)
    │   └─ Encyclopedic facts (specific data points, measurements, relationships)
    │
    ├─ Step 4: Event Extraction (grounded on entities + facts)
    │   ├─ Occurred / Ongoing / Planned / Cancelled / Hypothetical / Negated
    │   └─ Participant roles, temporal anchors, causal hints
    │
    └─ Step 5: Memory Extraction (grounded on entities + facts + events)
        └─ Emotional significance, identity relevance, reflective insight
```

### Phase 2: Consolidation (Reactive — per ingestion)

```
Extraction Output
    │
    ├─ Step 1: Entity Resolution (embedding similarity + LLM confirmation)
    │   └─ Merge duplicates, reconcile attributes, prefer most specific label
    │
    ├─ Step 2: Fact Reconciliation (supersede, not delete)
    │   ├─ Detect: confirms / contradicts / updates / unrelated
    │   ├─ If contradicts → create Supersedes edge, expire old fact
    │   └─ If confirms → reinforce certainty, link with Confirms edge
    │
    ├─ Step 3: Event Reconciliation
    │   ├─ Detect duplicate events (same event reported differently)
    │   ├─ Detect status updates (planned → occurred, ongoing → completed)
    │   └─ Create Supersedes edge for status transitions
    │
    ├─ Step 4: Memory Aggregation
    │   ├─ Cluster related events into memory candidates
    │   └─ Merge with existing memories if overlap detected
    │
    ├─ Step 5: Embedding Computation (batch)
    │
    ├─ Step 6: Relation Wiring (Source, Contains)
    │
    ├─ Step 7: Similarity Detection (cosine > threshold)
    │
    └─ Step 8: Temporal & Causal Wiring (existing temporal pipeline)
```

### Phase 3: Periodic Maintenance

- **Temporal Linking** (every 5 min) — ordering, causality, validity windows
- **Forgetting** (every 1 hour) — relevance scoring, expiration marking, certainty decay

---

## Supersede Model (No-Delete Pattern)

Instead of deleting outdated information, we:

1. **Set `expire_at`** on the old node (72-hour grace period)
2. **Create a `Supersedes` edge** from new node → old node with metadata:
   - `reason`: why the old info is outdated
   - `superseded_at`: timestamp
   - `confidence`: how certain we are this replaces the old info
3. **Queries filter by**: `expire_at IS NULL OR expire_at > now()`
4. **Traversal through Supersedes** allows historical reconstruction

### Example
```
Fact("Alice works at Google") ←── Supersedes ── Fact("Alice works at OpenAI")
     expire_at: 2024-01-15              reason: "Job change mentioned"
```

---

## Entity Type Taxonomy

### Concrete Entities
- **Person** — named individuals, speakers, mentioned people
- **Organization** — companies, institutions, teams, governments
- **Location** — geographical entities (countries, cities, addresses)
- **Place** — functional locations (home, office, the café on Main St)
- **Thing** — physical objects, products, devices
- **CreativeWork** — books, movies, songs, articles, projects

### Abstract Entities
- **Concept** — ideas, theories, fields of study, methodologies
- **Skill** — abilities, competencies, languages spoken
- **Condition** — health conditions, emotional states, situations
- **Role** — job titles, social roles, relationships

---

## Fact Taxonomy

### Semantic Facts (stable, definitional)
- Properties: "Python is a programming language"
- Classifications: "Berlin is a city in Germany"
- Definitions: "Machine learning is a subset of AI"

### Encyclopedic Facts (specific, verifiable)
- Attributes: "Alice is 32 years old"
- Relationships: "Alice works at Google"
- Quantities: "The project has 15 contributors"
- Preferences: "Bob prefers TypeScript over JavaScript"

### Fact Certainty Spectrum
- **Stated** (0.9-1.0): Directly stated as true
- **Implied** (0.6-0.8): Strongly implied but not explicitly stated
- **Inferred** (0.3-0.5): Logical inference from context
- **Speculative** (0.1-0.2): Mentioned as possibility

---

## Event Taxonomy

### Status Model
- **occurred** — definitively happened in the past
- **ongoing** — currently happening, not yet concluded
- **planned** — intended to happen in the future
- **cancelled** — was planned but explicitly cancelled
- **hypothetical** — discussed as possibility, not committed
- **negated** — explicitly stated as NOT having happened
- **recurring** — happens repeatedly on a pattern

### Participant Roles
- **agent** — who/what performed the action
- **patient** — who/what was affected
- **instrument** — what was used
- **beneficiary** — who benefited
- **location** — where it happened

---

## Memory Model

Memories are **higher-order aggregations** that:
1. Connect multiple events into a narrative arc
2. Carry emotional valence and personal significance
3. Link to identity (who the person is/was/wants to be)
4. Have a **significance score** based on emotional weight and life impact

### Memory Types
- **Episodic** — specific autobiographical events ("my first day at Google")
- **Achievement** — accomplishments and milestones
- **Relational** — relationship-defining moments
- **Transformative** — events that changed perspective or direction
- **Traumatic** — negative significant events (handle with care)
- **Aspirational** — goals and dreams being formed

---

## Prompt Design Principles

1. **Role + Task + Format** — every prompt has a clear persona, specific task, and exact output format
2. **Few-shot examples** — 2-3 diverse examples covering edge cases
3. **Negative examples** — show what NOT to extract to reduce false positives
4. **Grounding context** — pass prior extraction results to maintain coherence
5. **Chain-of-thought hints** — guide the LLM's reasoning without requiring CoT output
6. **Schema alignment** — output fields exactly match our Rust serde structs
