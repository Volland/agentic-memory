-- ============================================================================
-- Memory Ontology — DuckDB Schema
-- Conversational layer: source of truth for conversations and messages
-- Projected into Kùzu as node tables via ATTACH
-- ============================================================================

-- ============================================================================
-- CONVERSATIONS
-- ============================================================================

CREATE TABLE conversations (
    id               VARCHAR PRIMARY KEY,
    title            VARCHAR,
    started_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at         TIMESTAMP,
    participant      VARCHAR,              -- 'user'|'assistant'|'pair'|'group'
    model            VARCHAR,              -- LLM model identifier
    summary          VARCHAR,
    tags             VARCHAR[],
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- MESSAGES
-- ============================================================================

CREATE TABLE messages (
    id               VARCHAR PRIMARY KEY,
    conversation_id  VARCHAR NOT NULL REFERENCES conversations(id),
    role             VARCHAR NOT NULL,     -- 'user'|'assistant'|'system'|'tool'
    content          VARCHAR NOT NULL,
    content_embedding FLOAT[518],          -- embedding for semantic search over messages
    token_count      INTEGER,
    message_index    INTEGER NOT NULL,     -- ordering within conversation
    parent_message_id VARCHAR REFERENCES messages(id), -- for branching conversations
    created_at       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Fast conversation message retrieval in order
CREATE INDEX idx_messages_conversation ON messages(conversation_id, message_index);

-- Filter messages by role within a conversation
CREATE INDEX idx_messages_role ON messages(conversation_id, role);

-- Recent conversations first
CREATE INDEX idx_conversations_time ON conversations(started_at DESC);

-- Branch lookups
CREATE INDEX idx_messages_parent ON messages(parent_message_id);
