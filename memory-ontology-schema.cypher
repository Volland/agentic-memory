-- ============================================================================
-- Memory Ontology — Kùzu Cypher Schema
-- Bipartite Graph with Dedicated Typed Edge Nodes
-- Semantic Spacetime + FIRE-inspired Memory Ontology
-- ============================================================================


-- ============================================================================
-- ENTITY NODE TABLES
-- ============================================================================

CREATE NODE TABLE Entity (
    id               STRING PRIMARY KEY,
    -- universal
    label            STRING,
    label_resolved   STRING,
    label_embedding  FLOAT[518],
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 1,
    context          STRING
);

CREATE NODE TABLE Time (
    id               STRING PRIMARY KEY,
    granularity      STRING,
    starts_at        TIMESTAMP,
    ends_at          TIMESTAMP,
    -- universal
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 1,
    context          STRING
);

CREATE NODE TABLE AbstractTime (
    id               STRING PRIMARY KEY,
    semantics        STRING,
    -- universal
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 1,
    context          STRING
);

CREATE NODE TABLE Fact (
    id               STRING PRIMARY KEY,
    predicate        STRING,
    certainty        DOUBLE DEFAULT 1.0,
    source           STRING,
    -- universal
    label            STRING,
    label_resolved   STRING,
    label_embedding  FLOAT[518],
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 2,
    context          STRING
);

CREATE NODE TABLE Event (
    id               STRING PRIMARY KEY,
    predicate        STRING,
    certainty        DOUBLE DEFAULT 1.0,
    source           STRING,
    status           STRING DEFAULT 'occurred',
    is_ongoing       BOOLEAN DEFAULT FALSE,
    -- universal
    label            STRING,
    label_resolved   STRING,
    label_embedding  FLOAT[518],
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 3,
    context          STRING
);

CREATE NODE TABLE Memory (
    id               STRING PRIMARY KEY,
    predicate        STRING,
    certainty        DOUBLE DEFAULT 1.0,
    source           STRING,
    status           STRING DEFAULT 'occurred',
    is_ongoing       BOOLEAN DEFAULT FALSE,
    significance     STRING,
    emotions         STRING[],
    reflection       STRING,
    -- universal
    label            STRING,
    label_resolved   STRING,
    label_embedding  FLOAT[518],
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 4,
    context          STRING
);


-- ============================================================================
-- EDGE NODE TABLES (dedicated typed relation nodes)
-- ============================================================================

-- Spacetime
CREATE NODE TABLE Contains (
    id               STRING PRIMARY KEY,
    containment_type STRING,
    weight           DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE Similar (
    id               STRING PRIMARY KEY,
    similarity       DOUBLE DEFAULT 0.0,
    sim_context      STRING,
    sim_method       STRING,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE HasProperty (
    id               STRING PRIMARY KEY,
    property_name    STRING,
    property_value   STRING,
    prop_context     STRING,
    certainty        DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

-- Causal
CREATE NODE TABLE LeadsTo (
    id               STRING PRIMARY KEY,
    probability      DOUBLE DEFAULT 1.0,
    strength         DOUBLE DEFAULT 1.0,
    mechanism        STRING,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE Prevents (
    id               STRING PRIMARY KEY,
    probability      DOUBLE DEFAULT 1.0,
    strength         DOUBLE DEFAULT 1.0,
    mechanism        STRING,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE Causes (
    id               STRING PRIMARY KEY,
    probability      DOUBLE DEFAULT 1.0,
    strength         DOUBLE DEFAULT 1.0,
    mechanism        STRING,
    directness       STRING DEFAULT 'direct',
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE BecauseOf (
    id               STRING PRIMARY KEY,
    probability      DOUBLE DEFAULT 1.0,
    strength         DOUBLE DEFAULT 1.0,
    explanation      STRING,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

-- Temporal
CREATE NODE TABLE Before (
    id               STRING PRIMARY KEY,
    gap_duration     STRING,
    confidence       DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE After (
    id               STRING PRIMARY KEY,
    gap_duration     STRING,
    confidence       DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE During (
    id               STRING PRIMARY KEY,
    overlap_type     STRING DEFAULT 'full',
    confidence       DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

-- Validity
CREATE NODE TABLE ValidFrom (
    id               STRING PRIMARY KEY,
    precision        STRING DEFAULT 'exact',
    confidence       DOUBLE DEFAULT 1.0,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);

CREATE NODE TABLE ValidTo (
    id               STRING PRIMARY KEY,
    precision        STRING DEFAULT 'exact',
    confidence       DOUBLE DEFAULT 1.0,
    termination      STRING,
    label            STRING,
    label_resolved   STRING,
    learned_at       TIMESTAMP,
    expire_at        TIMESTAMP,
    created_at       TIMESTAMP,
    updated_at       TIMESTAMP,
    layer            INT16 DEFAULT 0,
    context          STRING
);


-- ============================================================================
-- POLYMORPHIC REL TABLES (bipartite wiring)
-- ============================================================================

-- Contains
CREATE REL TABLE FROM_Contains (FROM Memory | Event | Fact | Time TO Contains, role STRING DEFAULT 'source');
CREATE REL TABLE TO_Contains (FROM Contains TO Memory | Event | Fact | Entity | Time, role STRING DEFAULT 'target');

-- LeadsTo
CREATE REL TABLE FROM_LeadsTo (FROM Event | Memory | Fact TO LeadsTo, role STRING DEFAULT 'source');
CREATE REL TABLE TO_LeadsTo (FROM LeadsTo TO Event | Memory | Fact | AbstractTime, role STRING DEFAULT 'target');

-- Prevents
CREATE REL TABLE FROM_Prevents (FROM Event | Fact TO Prevents, role STRING DEFAULT 'source');
CREATE REL TABLE TO_Prevents (FROM Prevents TO Event | Fact | Memory, role STRING DEFAULT 'target');

-- Causes
CREATE REL TABLE FROM_Causes (FROM Event | Fact TO Causes, role STRING DEFAULT 'source');
CREATE REL TABLE TO_Causes (FROM Causes TO Event | Fact | Memory, role STRING DEFAULT 'target');

-- BecauseOf
CREATE REL TABLE FROM_BecauseOf (FROM Event | Fact | Memory TO BecauseOf, role STRING DEFAULT 'source');
CREATE REL TABLE TO_BecauseOf (FROM BecauseOf TO Event | Fact, role STRING DEFAULT 'target');

-- Similar
CREATE REL TABLE FROM_Similar (FROM Entity | Fact | Event | Memory TO Similar, role STRING DEFAULT 'source');
CREATE REL TABLE TO_Similar (FROM Similar TO Entity | Fact | Event | Memory, role STRING DEFAULT 'target');

-- HasProperty
CREATE REL TABLE FROM_HasProperty (FROM Entity | Fact | Event | Memory TO HasProperty, role STRING DEFAULT 'source');
CREATE REL TABLE TO_HasProperty (FROM HasProperty TO Entity, role STRING DEFAULT 'target');

-- Before
CREATE REL TABLE FROM_Before (FROM Event | Memory | Time | AbstractTime TO Before, role STRING DEFAULT 'source');
CREATE REL TABLE TO_Before (FROM Before TO Event | Memory | Time | AbstractTime, role STRING DEFAULT 'target');

-- After
CREATE REL TABLE FROM_After (FROM Event | Memory | Time | AbstractTime TO After, role STRING DEFAULT 'source');
CREATE REL TABLE TO_After (FROM After TO Event | Memory | Time | AbstractTime, role STRING DEFAULT 'target');

-- During
CREATE REL TABLE FROM_During (FROM Event | Memory TO During, role STRING DEFAULT 'source');
CREATE REL TABLE TO_During (FROM During TO Event | Memory | Time, role STRING DEFAULT 'target');

-- ValidFrom
CREATE REL TABLE FROM_ValidFrom (FROM Fact | Event | Memory TO ValidFrom, role STRING DEFAULT 'source');
CREATE REL TABLE TO_ValidFrom (FROM ValidFrom TO Time | AbstractTime, role STRING DEFAULT 'target');

-- ValidTo
CREATE REL TABLE FROM_ValidTo (FROM Fact | Event | Memory TO ValidTo, role STRING DEFAULT 'source');
CREATE REL TABLE TO_ValidTo (FROM ValidTo TO Time | AbstractTime, role STRING DEFAULT 'target');

-- Time Tree
CREATE REL TABLE TIME_HIERARCHY (FROM Time TO Time, ONE_TO_MANY);


-- ============================================================================
-- VECTOR INDEXES
-- ============================================================================

CREATE VECTOR INDEX idx_entity_embedding ON Entity(label_embedding)
USING HNSW WITH (metric = 'cosine', m = 16, ef_construction = 200, ef_search = 100);

CREATE VECTOR INDEX idx_fact_embedding ON Fact(label_embedding)
USING HNSW WITH (metric = 'cosine', m = 16, ef_construction = 200, ef_search = 100);

CREATE VECTOR INDEX idx_event_embedding ON Event(label_embedding)
USING HNSW WITH (metric = 'cosine', m = 16, ef_construction = 200, ef_search = 100);

CREATE VECTOR INDEX idx_memory_embedding ON Memory(label_embedding)
USING HNSW WITH (metric = 'cosine', m = 16, ef_construction = 200, ef_search = 100);
