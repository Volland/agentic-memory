use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{EdgeNodeType, RelationNode};

/// Property assertion edge attaching a key-value pair to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HasProperty {
    pub universal: UniversalColumns,
    pub property_name: String,
    pub property_value: String,
    pub prop_context: Option<String>,
    pub certainty: f64,
}

impl HasProperty {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Relation),
            property_name: String::new(),
            property_value: String::new(),
            prop_context: None,
            certainty: 1.0,
        }
    }

    pub fn with_property_name(mut self, property_name: impl Into<String>) -> Self {
        self.property_name = property_name.into();
        self
    }

    pub fn with_property_value(mut self, property_value: impl Into<String>) -> Self {
        self.property_value = property_value.into();
        self
    }

    pub fn with_prop_context(mut self, prop_context: impl Into<String>) -> Self {
        self.prop_context = Some(prop_context.into());
        self
    }

    pub fn with_certainty(mut self, certainty: f64) -> Self {
        self.certainty = certainty;
        self
    }
}

impl RelationNode for HasProperty {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn edge_type(&self) -> EdgeNodeType {
        EdgeNodeType::HasProperty
    }
}
