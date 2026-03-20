pub mod llm;
pub mod ner;
pub mod embedder;
pub mod classifier;
pub mod cognitive_process;

pub use llm::LlmBackend;
pub use ner::{NerBackend, NerSpan};
pub use embedder::EmbedderBackend;
pub use classifier::ClassifierBackend;
pub use cognitive_process::{CognitiveProcess, ProcessKind, ProcessResult};
