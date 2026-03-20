/// SQL DDL for the DuckDB conversation/message tables.
const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS conversations (
    id              VARCHAR PRIMARY KEY,
    title           VARCHAR,
    started_at      TIMESTAMPTZ NOT NULL,
    ended_at        TIMESTAMPTZ,
    participant     VARCHAR,
    model           VARCHAR,
    summary         VARCHAR,
    tags            VARCHAR[],
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS messages (
    id                  VARCHAR PRIMARY KEY,
    conversation_id     VARCHAR NOT NULL REFERENCES conversations(id),
    role                VARCHAR NOT NULL,
    content             TEXT NOT NULL,
    content_embedding   FLOAT[518],
    token_count         INTEGER,
    message_index       INTEGER NOT NULL,
    parent_message_id   VARCHAR,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_messages_conversation ON messages(conversation_id, message_index);
CREATE INDEX IF NOT EXISTS idx_messages_role ON messages(conversation_id, role);
CREATE INDEX IF NOT EXISTS idx_conversations_time ON conversations(started_at);
"#;

/// Return the full SQL DDL for initializing the DuckDB schema.
pub fn schema_sql() -> &'static str {
    SCHEMA_SQL
}
