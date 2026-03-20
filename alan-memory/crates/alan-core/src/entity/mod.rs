pub mod abstract_time;
pub mod entity;
pub mod event;
pub mod fact;
pub mod memory;
pub mod time;

use serde::{Deserialize, Serialize};

use crate::layer::Layer;
use crate::universal::UniversalColumns;

pub use self::abstract_time::AbstractTime;
pub use self::entity::Entity;
pub use self::event::{Event, EventStatus};
pub use self::fact::Fact;
pub use self::memory::Memory;
pub use self::time::{Time, TimeGranularity};

/// Enumeration of all content node types in the ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    Entity,
    Time,
    AbstractTime,
    Fact,
    Event,
    Memory,
    Conversation,
    Message,
}

impl NodeType {
    /// Get the default layer for this node type.
    pub fn default_layer(self) -> Layer {
        match self {
            Self::Entity | Self::Time | Self::AbstractTime => Layer::Entity,
            Self::Fact => Layer::Fact,
            Self::Event => Layer::Event,
            Self::Memory => Layer::Memory,
            Self::Conversation | Self::Message => Layer::Conversation,
        }
    }
}

/// Trait implemented by all ontology nodes (entities and relation nodes).
pub trait OntologyNode {
    /// Access the universal columns.
    fn universal(&self) -> &UniversalColumns;
    /// Mutable access to universal columns.
    fn universal_mut(&mut self) -> &mut UniversalColumns;
    /// The type of this node.
    fn node_type(&self) -> NodeType;
    /// The ontological layer.
    fn layer(&self) -> Layer {
        self.universal().layer
    }
}

/// Polymorphic holder for any content node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyContentNode {
    Entity(Entity),
    Time(Time),
    AbstractTime(AbstractTime),
    Fact(Fact),
    Event(Event),
    Memory(Memory),
}

impl AnyContentNode {
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Entity(_) => NodeType::Entity,
            Self::Time(_) => NodeType::Time,
            Self::AbstractTime(_) => NodeType::AbstractTime,
            Self::Fact(_) => NodeType::Fact,
            Self::Event(_) => NodeType::Event,
            Self::Memory(_) => NodeType::Memory,
        }
    }

    pub fn universal(&self) -> &UniversalColumns {
        match self {
            Self::Entity(n) => n.universal(),
            Self::Time(n) => n.universal(),
            Self::AbstractTime(n) => n.universal(),
            Self::Fact(n) => n.universal(),
            Self::Event(n) => n.universal(),
            Self::Memory(n) => n.universal(),
        }
    }

    pub fn universal_mut(&mut self) -> &mut UniversalColumns {
        match self {
            Self::Entity(n) => n.universal_mut(),
            Self::Time(n) => n.universal_mut(),
            Self::AbstractTime(n) => n.universal_mut(),
            Self::Fact(n) => n.universal_mut(),
            Self::Event(n) => n.universal_mut(),
            Self::Memory(n) => n.universal_mut(),
        }
    }
}
