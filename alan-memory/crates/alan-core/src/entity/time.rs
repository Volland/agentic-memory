use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

use super::{NodeType, OntologyNode};

/// Granularity level for concrete time nodes in the time tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeGranularity {
    Year,
    Month,
    Week,
    Day,
}

/// Concrete time node (layer 1).
/// Represents a specific point or period in the time tree hierarchy.
/// Year→Month→Week→Day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Time {
    pub universal: UniversalColumns,
    /// Granularity of this time node.
    pub granularity: TimeGranularity,
    /// Start of this time period.
    pub starts_at: Option<DateTime<Utc>>,
    /// End of this time period.
    pub ends_at: Option<DateTime<Utc>>,
}

impl Time {
    pub fn new(label: impl Into<String>, granularity: TimeGranularity) -> Self {
        Self {
            universal: UniversalColumns::new(label, Layer::Entity),
            granularity,
            starts_at: None,
            ends_at: None,
        }
    }

    pub fn with_bounds(mut self, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Self {
        self.starts_at = Some(starts_at);
        self.ends_at = Some(ends_at);
        self
    }
}

impl OntologyNode for Time {
    fn universal(&self) -> &UniversalColumns {
        &self.universal
    }

    fn universal_mut(&mut self) -> &mut UniversalColumns {
        &mut self.universal
    }

    fn node_type(&self) -> NodeType {
        NodeType::Time
    }
}
