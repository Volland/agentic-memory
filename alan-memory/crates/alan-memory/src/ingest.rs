use serde::{Deserialize, Serialize};

use alan_core::conversation::{Conversation, Message};

/// Result of an ingestion operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IngestResult {
    /// Number of content nodes created.
    pub nodes_created: usize,
    /// Number of edges (relations) created.
    pub edges_created: usize,
    /// Number of entities extracted.
    pub entities_extracted: usize,
    /// Number of facts extracted.
    pub facts_extracted: usize,
    /// Number of events extracted.
    pub events_extracted: usize,
    /// Number of memories extracted.
    pub memories_extracted: usize,
}

/// Metadata for a document being ingested.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Title of the document.
    pub title: Option<String>,
    /// Source or origin of the document.
    pub source: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
}

/// Convert a document text into a (Conversation, Vec<Message>) pair.
///
/// Splits the text on double newlines into paragraphs, treating each
/// paragraph as a separate "user" message within a synthetic conversation.
pub fn document_to_messages(
    text: &str,
    metadata: &DocumentMetadata,
) -> (Conversation, Vec<Message>) {
    let title = metadata
        .title
        .clone()
        .unwrap_or_else(|| "Document import".to_string());

    let conv = Conversation::new()
        .with_title(&title)
        .with_tags(metadata.tags.clone());

    let conv_id = conv.id.clone();

    // Split on double newlines; filter out empty paragraphs.
    let paragraphs: Vec<&str> = text
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    // If no paragraphs found after splitting, treat the whole text as one message.
    let paragraphs = if paragraphs.is_empty() {
        vec![text.trim()]
    } else {
        paragraphs
    };

    let messages: Vec<Message> = paragraphs
        .into_iter()
        .enumerate()
        .filter(|(_, p)| !p.is_empty())
        .map(|(i, paragraph)| {
            Message::new(conv_id.clone(), "user", paragraph, i as i32)
        })
        .collect();

    (conv, messages)
}
