# Bridging Relational and Graph: DuckDB + Kùzu Integration for Agentic Memory

How a dual-storage architecture gives conversational AI both efficient message storage and rich knowledge graph traversal.

---

## The Problem: Two Worlds of Data

Agentic memory systems face a fundamental tension. Conversations — the raw material from which knowledge is extracted — are inherently sequential and append-heavy. They benefit from relational storage: fast inserts, ordered retrieval, pagination, and full-text search. But the knowledge extracted from those conversations — entities, facts, events, causal chains, temporal relations — is inherently a graph. Querying "what do I know about Berlin, and how did I learn it?" requires traversing relationships across multiple node types and edge categories.

Forcing everything into one paradigm creates friction. A pure graph database handles conversations awkwardly — appending messages becomes a node-creation-plus-edge-wiring ceremony for what is essentially a log. A pure relational database handles knowledge graphs awkwardly — recursive CTEs and self-joins are poor substitutes for native graph traversal.

The solution: use both, and bridge them cleanly.

## The Architecture: DuckDB as Substrate, Kùzu as Graph

The LadybugDB memory ontology uses a **dual-storage architecture**:

- **DuckDB** stores conversations and messages as relational tables — the source of truth for all conversational data
- **Kùzu** (the graph engine) projects those DuckDB tables as node tables and weaves them into the broader knowledge graph

This works because Kùzu natively supports attaching external databases. A single `ATTACH` statement makes DuckDB tables available as graph nodes:

```cypher
ATTACH 'memory.duckdb' AS duck (dbtype duckdb);
```

From that point, Conversation and Message rows participate in graph queries alongside Entity, Fact, Event, and Memory nodes — with full Cypher traversal, pattern matching, and polymorphic edges.

## What Lives Where

### DuckDB: The Conversational Layer

DuckDB holds two tables, optimized for the access patterns conversations demand:

```sql
CREATE TABLE conversations (
    id               VARCHAR PRIMARY KEY,
    title            VARCHAR,
    started_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at         TIMESTAMP,
    participant      VARCHAR,     -- 'user'|'assistant'|'pair'|'group'
    model            VARCHAR,     -- LLM model identifier
    summary          VARCHAR,
    tags             VARCHAR[],
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE messages (
    id               VARCHAR PRIMARY KEY,
    conversation_id  VARCHAR NOT NULL REFERENCES conversations(id),
    role             VARCHAR NOT NULL,  -- 'user'|'assistant'|'system'|'tool'
    content          VARCHAR NOT NULL,
    content_embedding FLOAT[518],
    token_count      INTEGER,
    message_index    INTEGER NOT NULL,
    parent_message_id VARCHAR REFERENCES messages(id),
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

These are plain relational tables with foreign keys, B-tree indexes, and all the guarantees you expect. DuckDB handles:

- **Append-heavy writes** — every conversation turn adds a message row
- **Ordered retrieval** — `message_index` gives deterministic ordering within a conversation
- **Pagination and filtering** — "show me the last 20 conversations" or "all user messages in this thread"
- **Branching** — `parent_message_id` supports conversation forks without graph overhead
- **Bulk analytics** — "average tokens per conversation," "messages per day," "model usage distribution"

### Kùzu: The Knowledge Graph

Kùzu holds the full ontology — entities, facts, events, memories, and 13 types of typed edge nodes (Contains, Source, Similar, LeadsTo, Causes, Before, During, ValidFrom, and so on). This is a bipartite graph: entities never connect directly; every relationship passes through a dedicated edge node that carries its own properties.

The key insight is that Conversation and Message are **projected into this graph** as node tables at layer -1, below the entity layer:

| Layer | Meaning | Tables |
|-------|---------|--------|
| -1 | Raw conversational data (DuckDB-projected) | Conversation, Message |
| 0 | Edge/relation nodes | Contains, Source, Similar, ... |
| 1 | Entities and time | Entity, Time, AbstractTime |
| 2 | Facts | Fact |
| 3 | Events | Event |
| 4 | Memories | Memory |

Knowledge flows **upward**. Messages at layer -1 are processed to extract entities (1), facts (2), events (3), and memories (4). The graph tracks this provenance chain.

## The Bridge: Two Key Relations

Two relation types connect the conversational and knowledge layers:

### Contains — Structural Composition

The `Contains` edge node already models compositional relationships throughout the ontology (Memory contains Events, Events contain Facts, etc.). Extending it to the conversational layer is natural:

- **Conversation → Message**: a conversation contains its messages
- **Message → Entity/Fact/Event/Memory**: a message contains extracted knowledge

```cypher
// What knowledge was extracted from this message?
MATCH (msg:Message)-[:FROM_Contains]->(ct:Contains)-[:TO_Contains]->(knowledge)
WHERE msg.id = 'msg-42'
RETURN labels(knowledge)[0] AS type, knowledge.label_resolved, ct.weight;
```

### Source — Provenance Attribution

`Source` is a new edge node type in the Provenance category. It points from extracted knowledge **back** to the originating message, answering "where did this fact come from?"

```cypher
CREATE NODE TABLE Source (
    id               STRING PRIMARY KEY,
    extraction_method STRING,   -- 'llm'|'regex'|'ner'|'manual'|'tool'
    confidence       DOUBLE DEFAULT 1.0,
    fragment         STRING,    -- the relevant substring from the message
    -- universal columns ...
);
```

The `fragment` field is particularly useful — it stores the exact substring from the message that was used to extract the knowledge, enabling audit and debugging of the extraction pipeline.

```cypher
// Where did this fact come from?
MATCH (f:Fact)-[:FROM_Source]->(s:Source)-[:TO_Source]->(msg:Message)
      -[:CONVERSATION_MESSAGES]-(conv:Conversation)
WHERE f.id = 'fact-berlin-capital'
RETURN conv.title, msg.message_index, msg.role, s.fragment, s.extraction_method;
```

## Why This Split Works

### Each Engine Does What It's Best At

**DuckDB excels at:**
- Sequential append (message ingestion)
- Columnar analytics ("how many tokens did this conversation use?")
- Ordered scans (replaying a conversation)
- SQL-native full-text search
- Embedding storage for vector similarity

**Kùzu excels at:**
- Multi-hop traversal ("trace this fact back through its causal chain to the original conversation")
- Pattern matching ("find all entities mentioned in the same conversation as Berlin")
- Polymorphic edges (one `Contains` edge type handles Conversation→Message and Memory→Event)
- Layered visibility (query only memories, or only facts, or everything)

### No Data Duplication

The conversational data lives in DuckDB only. Kùzu's `ATTACH` mechanism creates a live projection — there's no ETL step, no sync job, no stale copy. When a new message is inserted into DuckDB, it's immediately visible in graph queries.

### Clean Separation of Concerns

The write path is simple: append messages to DuckDB. The extraction pipeline reads messages, produces knowledge nodes (in Kùzu), and wires them back to their source messages via `Source` edges. The read path can use either engine depending on the query:

```
┌─────────────┐     append      ┌──────────┐
│  Agent /    │ ──────────────> │  DuckDB  │
│  User       │                 │ messages │
└─────────────┘                 └────┬─────┘
                                     │
                              ATTACH + project
                                     │
                                     v
                        ┌────────────────────────┐
                        │        Kùzu Graph       │
                        │                        │
                        │  Message ──Source──> Fact│
                        │  Message ──Contains─> Entity│
                        │  Conversation ──Contains─> Message│
                        │  Fact ──LeadsTo──> Event│
                        │  ...                    │
                        └────────────────────────┘
```

## Query Patterns That Cross the Bridge

### "What did I learn from last week's conversations?"

Start in the relational world (filter by time), cross into the graph (follow Source edges):

```cypher
MATCH (conv:Conversation)-[:CONVERSATION_MESSAGES]->(msg:Message)
      <-[:TO_Source]-(s:Source)<-[:FROM_Source]-(knowledge)
WHERE conv.started_at > timestamp('2026-03-07')
RETURN conv.title,
       labels(knowledge)[0] AS type,
       knowledge.label_resolved,
       s.extraction_method
ORDER BY conv.started_at;
```

### "How did I learn about Berlin?"

Start from an entity, trace provenance back to conversations:

```cypher
MATCH (e:Entity {label_resolved: 'Berlin, Germany'})
      -[:FROM_Source]->(s:Source)-[:TO_Source]->(msg:Message)
      -[:CONVERSATION_MESSAGES]-(conv:Conversation)
RETURN conv.title, conv.started_at,
       msg.role, msg.message_index,
       s.fragment, s.confidence
ORDER BY conv.started_at;
```

### "Show me all facts extracted from assistant messages"

Filter by role (a relational concern), then traverse (a graph concern):

```cypher
MATCH (msg:Message)<-[:TO_Source]-(s:Source)<-[:FROM_Source]-(f:Fact)
WHERE msg.role = 'assistant'
RETURN f.label_resolved, f.predicate, f.certainty,
       s.extraction_method, msg.conversation_id;
```

### "Replay a conversation with its extracted knowledge inline"

Combine ordered retrieval with graph expansion:

```cypher
MATCH (conv:Conversation)-[:CONVERSATION_MESSAGES]->(msg:Message)
WHERE conv.id = $conversation_id
OPTIONAL MATCH (msg)-[:FROM_Contains]->(ct:Contains)-[:TO_Contains]->(knowledge)
RETURN msg.message_index, msg.role, msg.content,
       collect({type: labels(knowledge)[0], label: knowledge.label_resolved}) AS extracted
ORDER BY msg.message_index;
```

## Vector Search Across Both Layers

Both DuckDB and Kùzu support HNSW vector indexes. The system maintains embeddings at two levels:

- **Message level** (`content_embedding` on Message) — semantic search over raw conversation content
- **Knowledge level** (`label_embedding` on Entity/Fact/Event/Memory) — semantic search over extracted, structured knowledge

This means you can search for "career change" and get hits from both the raw conversation ("I've been thinking about switching to product management") and the extracted knowledge (Fact: "considering career transition to product management").

```cypher
// Search messages semantically
CALL vector_search(Message, 'idx_message_embedding', $query_embedding, 10)
WITH node AS msg
MATCH (msg)-[:CONVERSATION_MESSAGES]-(conv:Conversation)
RETURN msg.content, msg.role, conv.title;

// Search extracted knowledge semantically
CALL vector_search(Fact, 'idx_fact_embedding', $query_embedding, 10)
WITH node AS f
MATCH (f)-[:FROM_Source]->(s:Source)-[:TO_Source]->(msg:Message)
RETURN f.label_resolved, s.fragment, msg.conversation_id;
```

## The Extraction Pipeline

The integration creates a natural processing pipeline:

1. **Ingest**: Append messages to DuckDB as they arrive
2. **Extract**: Process messages through LLM/NER/regex to identify entities, facts, events
3. **Store**: Create knowledge nodes in Kùzu
4. **Wire**: Create `Source` edges (knowledge → message) and `Contains` edges (message → knowledge)
5. **Enrich**: Add temporal, causal, and similarity edges between knowledge nodes
6. **Query**: Traverse the unified graph — from high-level memories down to the exact conversation turn where a fact was first mentioned

Each step is independent and can run asynchronously. The DuckDB → Kùzu bridge ensures that step 2 can always access the latest messages, and steps 3-5 can always trace back to the source.

## Summary

| Concern | Engine | Why |
|---------|--------|-----|
| Message storage | DuckDB | Append-optimized, columnar, SQL |
| Conversation replay | DuckDB | Ordered index on (conversation_id, message_index) |
| Bulk analytics | DuckDB | Columnar aggregations, window functions |
| Knowledge graph | Kùzu | Native graph traversal, typed edges, Cypher |
| Provenance tracking | Kùzu (Source edges) | Multi-hop path from fact → message → conversation |
| Compositional queries | Kùzu (Contains edges) | Recursive containment across layers |
| Semantic search (messages) | Kùzu (HNSW on Message) | Vector index over projected DuckDB data |
| Semantic search (knowledge) | Kùzu (HNSW on Entity/Fact/Event/Memory) | Vector index over graph nodes |

The dual-storage approach avoids the "one database to rule them all" trap. Conversations are relational data — store them relationally. Knowledge is graph data — store it in a graph. The `ATTACH` bridge makes them feel like one system, with no sync overhead and full query interoperability.
