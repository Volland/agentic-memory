use alan_core::conversation::message::Message;

/// Strategy for splitting a sequence of messages into processable chunks.
#[derive(Debug, Clone)]
pub enum ChunkStrategy {
    /// Each message becomes its own chunk.
    PerMessage,
    /// Sliding window of `window_size` messages with `overlap` messages shared
    /// between consecutive chunks.
    SlidingWindow { window_size: usize, overlap: usize },
    /// Group by conversation turn (user + assistant pair).
    ByTurn,
    /// Keep adding messages until the estimated token budget is reached.
    TokenBudget { max_tokens: usize },
}

/// A chunk of messages ready for cognitive processing.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub messages: Vec<Message>,
    /// Pre-concatenated text of all messages in the chunk.
    pub text: String,
}

impl Chunk {
    fn from_messages(msgs: Vec<Message>) -> Self {
        let text = msgs
            .iter()
            .map(|m| format!("{}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        Self {
            messages: msgs,
            text,
        }
    }
}

/// Split messages according to the chosen strategy.
pub fn chunk_messages(messages: &[Message], strategy: &ChunkStrategy) -> Vec<Chunk> {
    if messages.is_empty() {
        return Vec::new();
    }

    match strategy {
        ChunkStrategy::PerMessage => messages
            .iter()
            .map(|m| Chunk::from_messages(vec![m.clone()]))
            .collect(),

        ChunkStrategy::SlidingWindow {
            window_size,
            overlap,
        } => {
            let window = (*window_size).max(1);
            let step = (window.saturating_sub(*overlap)).max(1);
            let mut chunks = Vec::new();
            let mut start = 0;
            while start < messages.len() {
                let end = (start + window).min(messages.len());
                chunks.push(Chunk::from_messages(messages[start..end].to_vec()));
                start += step;
            }
            chunks
        }

        ChunkStrategy::ByTurn => {
            let mut chunks = Vec::new();
            let mut current_turn: Vec<Message> = Vec::new();

            for msg in messages {
                if msg.role == "user" && !current_turn.is_empty() {
                    chunks.push(Chunk::from_messages(std::mem::take(&mut current_turn)));
                }
                current_turn.push(msg.clone());
            }
            if !current_turn.is_empty() {
                chunks.push(Chunk::from_messages(current_turn));
            }
            chunks
        }

        ChunkStrategy::TokenBudget { max_tokens } => {
            let budget = (*max_tokens).max(1);
            let mut chunks = Vec::new();
            let mut current: Vec<Message> = Vec::new();
            let mut current_tokens: usize = 0;

            for msg in messages {
                // Rough estimate: 1 token ≈ 4 chars.
                let estimated = msg.content.len() / 4;
                if !current.is_empty() && current_tokens + estimated > budget {
                    chunks.push(Chunk::from_messages(std::mem::take(&mut current)));
                    current_tokens = 0;
                }
                current_tokens += estimated;
                current.push(msg.clone());
            }
            if !current.is_empty() {
                chunks.push(Chunk::from_messages(current));
            }
            chunks
        }
    }
}
