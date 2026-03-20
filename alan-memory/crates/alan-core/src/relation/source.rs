use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Method used to extract knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExtractionMethod {
    Llm,
    Regex,
    Ner,
    Manual,
    Tool,
}

/// Provenance edge linking a node back to its extraction source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub universal: UniversalColumns,
    pub extraction_method: ExtractionMethod,
    pub confidence: f64,
    pub fragment: Option<String>,
}

impl Source {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            extraction_method: ExtractionMethod::Llm,
            confidence: 1.0,
            fragment: None,
        }
    }

    pub fn with_extraction_method(mut self, extraction_method: ExtractionMethod) -> Self {
        self.extraction_method = extraction_method;
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_fragment(mut self, fragment: impl Into<String>) -> Self {
        self.fragment = Some(fragment.into());
        self
    }
}

impl RelationNode for Source {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::Source
    }
}
