// ============================================================================
// Memory Ontology — Kùzu Example Data & Query Patterns
// ============================================================================


// ============================================================================
// EXAMPLE DATA: "Moved to Berlin" scenario
// ============================================================================

// === Entity ===
CREATE (:Entity {
    id: 'e-berlin',
    label: 'Berlin', label_resolved: 'Berlin, Germany',
    label_embedding: null,
    learned_at: timestamp('2024-01-01'), expire_at: null,
    created_at: timestamp('2024-01-01'), updated_at: timestamp('2024-01-01'),
    layer: 1, context: 'European capital city'
});

// === Time ===
CREATE (:Time {
    id: 't-2024-03',
    granularity: 'month',
    starts_at: timestamp('2024-03-01'), ends_at: timestamp('2024-03-31'),
    label: 'March 2024', label_resolved: '2024-03',
    learned_at: timestamp('2024-01-01'), expire_at: null,
    created_at: timestamp('2024-01-01'), updated_at: timestamp('2024-01-01'),
    layer: 1, context: 'calendar month'
});

// === AbstractTime ===
CREATE (:AbstractTime {
    id: 'at-future',
    semantics: 'unbounded forward',
    label: 'future', label_resolved: 'future (unbounded)',
    learned_at: timestamp('2024-01-01'), expire_at: null,
    created_at: timestamp('2024-01-01'), updated_at: timestamp('2024-01-01'),
    layer: 1, context: 'abstract temporal marker'
});

// === Event ===
CREATE (:Event {
    id: 'ev-moved',
    predicate: 'relocation',
    certainty: 1.0, source: 'user', status: 'occurred', is_ongoing: false,
    label: 'Moved to Berlin', label_resolved: 'Relocation to Berlin',
    label_embedding: null,
    learned_at: timestamp('2024-03-15'), expire_at: null,
    created_at: timestamp('2024-03-15'), updated_at: timestamp('2024-03-15'),
    layer: 3, context: 'Career relocation'
});

// === Memory ===
CREATE (:Memory {
    id: 'm-berlin',
    predicate: 'life_chapter',
    certainty: 1.0, source: 'user', status: 'occurred', is_ongoing: true,
    significance: 'Major life transition', emotions: ['excitement', 'hope'],
    reflection: 'The turning point I needed',
    label: 'Berlin Chapter', label_resolved: 'Life chapter: Berlin',
    label_embedding: null,
    learned_at: timestamp('2024-03-15'), expire_at: null,
    created_at: timestamp('2024-03-15'), updated_at: timestamp('2024-03-15'),
    layer: 4, context: 'Ongoing Berlin narrative'
});


// ============================================================================
// BIPARTITE WIRING
// ============================================================================

// Memory —[Contains]→ Event
CREATE (:Contains {
    id: 'c-01',
    containment_type: 'composition', weight: 1.0,
    label: 'contains', label_resolved: 'memory contains event',
    learned_at: timestamp('2024-03-15'), expire_at: null,
    created_at: timestamp('2024-03-15'), updated_at: timestamp('2024-03-15'),
    layer: 0, context: 'narrative composition'
});

MATCH (m:Memory {id: 'm-berlin'}), (c:Contains {id: 'c-01'})
CREATE (m)-[:FROM_Contains {role: 'container'}]->(c);

MATCH (c:Contains {id: 'c-01'}), (ev:Event {id: 'ev-moved'})
CREATE (c)-[:TO_Contains {role: 'contained'}]->(ev);


// Event —[ValidFrom]→ Time
CREATE (:ValidFrom {
    id: 'vf-01',
    precision: 'exact', confidence: 1.0,
    label: 'valid_from', label_resolved: 'starts March 2024',
    learned_at: timestamp('2024-03-15'), expire_at: null,
    created_at: timestamp('2024-03-15'), updated_at: timestamp('2024-03-15'),
    layer: 0, context: 'relocation start'
});

MATCH (ev:Event {id: 'ev-moved'}), (vf:ValidFrom {id: 'vf-01'})
CREATE (ev)-[:FROM_ValidFrom {role: 'source'}]->(vf);

MATCH (vf:ValidFrom {id: 'vf-01'}), (t:Time {id: 't-2024-03'})
CREATE (vf)-[:TO_ValidFrom {role: 'anchor'}]->(t);


// Memory —[LeadsTo]→ AbstractTime (ongoing)
CREATE (:LeadsTo {
    id: 'lt-01',
    probability: 0.9, strength: 0.8, mechanism: 'ongoing life chapter',
    label: 'leads_to', label_resolved: 'leads to future',
    learned_at: timestamp('2024-03-15'), expire_at: null,
    created_at: timestamp('2024-03-15'), updated_at: timestamp('2024-03-15'),
    layer: 0, context: 'open-ended chapter'
});

MATCH (m:Memory {id: 'm-berlin'}), (lt:LeadsTo {id: 'lt-01'})
CREATE (m)-[:FROM_LeadsTo {role: 'source'}]->(lt);

MATCH (lt:LeadsTo {id: 'lt-01'}), (at:AbstractTime {id: 'at-future'})
CREATE (lt)-[:TO_LeadsTo {role: 'target'}]->(at);


// Entity —[HasProperty]→ (self-contained)
CREATE (:HasProperty {
    id: 'hp-01',
    property_name: 'population', property_value: '3.7M',
    prop_context: '2024 estimate', certainty: 0.95,
    label: 'has_property', label_resolved: 'population of Berlin',
    learned_at: timestamp('2024-01-01'), expire_at: timestamp('2025-12-31'),
    created_at: timestamp('2024-01-01'), updated_at: timestamp('2024-01-01'),
    layer: 0, context: 'demographic data'
});

MATCH (b:Entity {id: 'e-berlin'}), (hp:HasProperty {id: 'hp-01'})
CREATE (b)-[:FROM_HasProperty {role: 'owner'}]->(hp);


// ============================================================================
// QUERY PATTERNS
// ============================================================================

// Memory Layer (Layer 4)
MATCH (m:Memory)-[:FROM_Contains]->(c:Contains)-[:TO_Contains]->(ev:Event)
RETURN m.label_resolved, m.emotions, ev.label_resolved, c.containment_type;

// Event Layer with Validity (Layer 3)
MATCH (ev:Event)-[:FROM_ValidFrom]->(vf:ValidFrom)-[:TO_ValidFrom]->(t:Time)
RETURN ev.label_resolved, ev.status, t.label_resolved, vf.precision;

// Full Validity Window
MATCH (f:Fact)-[:FROM_ValidFrom]->(vf:ValidFrom)-[:TO_ValidFrom]->(t1:Time),
      (f)-[:FROM_ValidTo]->(vt:ValidTo)-[:TO_ValidTo]->(t2:Time)
RETURN f.label_resolved, t1.label_resolved AS from, t2.label_resolved AS to;

// Causal Forward Chain
MATCH (a:Event)-[:FROM_LeadsTo]->(lt:LeadsTo)-[:TO_LeadsTo]->(b:Event)
RETURN a.label_resolved, lt.probability, lt.mechanism, b.label_resolved;

// Reverse Causality
MATCH (what:Event)-[:FROM_BecauseOf]->(bo:BecauseOf)-[:TO_BecauseOf]->(why:Event)
RETURN what.label_resolved, bo.explanation, why.label_resolved;

// What Prevented What?
MATCH (blocker:Event)-[:FROM_Prevents]->(p:Prevents)-[:TO_Prevents]->(blocked:Event)
RETURN blocker.label_resolved, p.mechanism, blocked.label_resolved, blocked.status;

// Similarity Search
MATCH (a)-[:FROM_Similar]->(s:Similar)-[:TO_Similar]->(b)
WHERE s.similarity > 0.7
RETURN a.label_resolved, b.label_resolved, s.similarity, s.sim_context;

// Temporal Ordering
MATCH (first:Event)-[:FROM_Before]->(b:Before)-[:TO_Before]->(second:Event)
RETURN first.label_resolved, b.gap_duration, second.label_resolved;

// During a Time Period
MATCH (ev:Event)-[:FROM_During]->(d:During)-[:TO_During]->(period:Time)
WHERE period.label_resolved = '2024'
RETURN ev.label_resolved, d.overlap_type;

// Vector Search: Memories
CALL vector_search(Memory, 'idx_memory_embedding', $query_embedding, 10)
RETURN node.label_resolved, node.significance, node.emotions, node.context;

// Vector Search: Events → Temporal Anchors
CALL vector_search(Event, 'idx_event_embedding', $query_embedding, 5)
WITH node AS ev
MATCH (ev)-[:FROM_ValidFrom]->(vf:ValidFrom)-[:TO_ValidFrom]->(t:Time)
RETURN ev.label_resolved, ev.status, t.label_resolved, vf.precision;

// Vector Search: Facts → Related Entities
CALL vector_search(Fact, 'idx_fact_embedding', $query_embedding, 5)
WITH node AS f
MATCH (f)-[:FROM_Contains]->(c:Contains)-[:TO_Contains]->(e:Entity)
RETURN f.label_resolved, f.predicate, e.label_resolved;

// Vector Search: Entities → Containing Facts
CALL vector_search(Entity, 'idx_entity_embedding', $query_embedding, 5)
WITH node AS e
MATCH (e)<-[:TO_Contains]-(c:Contains)<-[:FROM_Contains]-(f:Fact)
RETURN e.label_resolved, f.label_resolved, f.predicate;
