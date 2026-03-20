/// Cypher DDL for the LadybugDB graph schema.
const SCHEMA_CYPHER: &str = r#"
-- Entity layer (L0)
CREATE NODE TABLE IF NOT EXISTS Entity (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    entity_type STRING,
    description STRING,
    aliases STRING[],
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- Time node (L0)
CREATE NODE TABLE IF NOT EXISTS Time (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    iso_start TIMESTAMP,
    iso_end TIMESTAMP,
    granularity STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- AbstractTime node (L0)
CREATE NODE TABLE IF NOT EXISTS AbstractTime (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    recurrence STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- Fact layer (L1)
CREATE NODE TABLE IF NOT EXISTS Fact (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    subject STRING,
    predicate STRING,
    object STRING,
    confidence FLOAT,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- Event layer (L2)
CREATE NODE TABLE IF NOT EXISTS Event (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    description STRING,
    status STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- Memory layer (L3)
CREATE NODE TABLE IF NOT EXISTS Memory (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    summary STRING,
    emotion STRING,
    importance FLOAT,
    access_count INT64,
    last_accessed TIMESTAMP,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING,
    embedding FLOAT[518]
);

-- Relation (edge) node tables
CREATE NODE TABLE IF NOT EXISTS Contains (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Source (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Similar (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    score FLOAT,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS HasProperty (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS LeadsTo (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Prevents (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Causes (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS BecauseOf (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Before (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS After (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS During (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS ValidFrom (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS ValidTo (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

CREATE NODE TABLE IF NOT EXISTS Supersedes (
    id STRING PRIMARY KEY,
    label STRING,
    label_resolved STRING,
    reason STRING,
    confidence FLOAT,
    learned_at TIMESTAMP,
    expire_at TIMESTAMP,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    layer INT16,
    context STRING
);

-- Bipartite REL tables (content_node -> edge_node -> content_node)
CREATE REL TABLE IF NOT EXISTS FROM_NODE (FROM Entity | Time | AbstractTime | Fact | Event | Memory TO Contains | Source | Similar | HasProperty | LeadsTo | Prevents | Causes | BecauseOf | Before | After | During | ValidFrom | ValidTo | Supersedes);
CREATE REL TABLE IF NOT EXISTS TO_NODE   (FROM Contains | Source | Similar | HasProperty | LeadsTo | Prevents | Causes | BecauseOf | Before | After | During | ValidFrom | ValidTo | Supersedes TO Entity | Time | AbstractTime | Fact | Event | Memory);
"#;

/// Return the full Cypher DDL for initializing the LadybugDB schema.
pub fn schema_cypher() -> &'static str {
    SCHEMA_CYPHER
}
