pub mod after;
pub mod because_of;
pub mod before;
pub mod causes;
pub mod contains;
pub mod during;
pub mod has_property;
pub mod leads_to;
pub mod prevents;
pub mod similar;
pub mod source;
pub mod valid_from;
pub mod valid_to;

pub use after::After;
pub use because_of::BecauseOf;
pub use before::Before;
pub use causes::Causes;
pub use contains::Contains;
pub use during::During;
pub use has_property::HasProperty;
pub use leads_to::LeadsTo;
pub use prevents::Prevents;
pub use similar::Similar;
pub use source::Source;
pub use valid_from::ValidFrom;
pub use valid_to::ValidTo;

use serde::{Deserialize, Serialize};

use crate::universal::UniversalColumns;

/// Discriminant for every relation (edge) type in the ontology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeNodeType {
    Contains,
    Source,
    Similar,
    HasProperty,
    LeadsTo,
    Prevents,
    Causes,
    BecauseOf,
    Before,
    After,
    During,
    ValidFrom,
    ValidTo,
}

/// High-level category that groups related edge types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeCategory {
    /// Spatial / structural containment edges.
    Spacetime,
    /// Data-lineage and extraction provenance.
    Provenance,
    /// Cause-and-effect family.
    Causal,
    /// Ordering / sequencing in time.
    Temporal,
    /// Temporal validity windows.
    Validity,
}

impl EdgeNodeType {
    /// Return the high-level category this edge type belongs to.
    pub fn category(&self) -> EdgeCategory {
        match self {
            EdgeNodeType::Contains => EdgeCategory::Spacetime,
            EdgeNodeType::Source => EdgeCategory::Provenance,
            EdgeNodeType::Similar => EdgeCategory::Spacetime,
            EdgeNodeType::HasProperty => EdgeCategory::Spacetime,
            EdgeNodeType::LeadsTo => EdgeCategory::Causal,
            EdgeNodeType::Prevents => EdgeCategory::Causal,
            EdgeNodeType::Causes => EdgeCategory::Causal,
            EdgeNodeType::BecauseOf => EdgeCategory::Causal,
            EdgeNodeType::Before => EdgeCategory::Temporal,
            EdgeNodeType::After => EdgeCategory::Temporal,
            EdgeNodeType::During => EdgeCategory::Temporal,
            EdgeNodeType::ValidFrom => EdgeCategory::Validity,
            EdgeNodeType::ValidTo => EdgeCategory::Validity,
        }
    }
}

/// Trait implemented by every relation (edge) node in the ontology.
pub trait RelationNode {
    /// Borrow the universal columns.
    fn universal(&self) -> &UniversalColumns;
    /// Mutably borrow the universal columns.
    fn universal_mut(&mut self) -> &mut UniversalColumns;
    /// The discriminant for this edge type.
    fn edge_type(&self) -> EdgeNodeType;
}

/// Type-erased wrapper that can hold any relation node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyRelationNode {
    Contains(Contains),
    Source(Source),
    Similar(Similar),
    HasProperty(HasProperty),
    LeadsTo(LeadsTo),
    Prevents(Prevents),
    Causes(Causes),
    BecauseOf(BecauseOf),
    Before(Before),
    After(After),
    During(During),
    ValidFrom(ValidFrom),
    ValidTo(ValidTo),
}

impl AnyRelationNode {
    /// Borrow the universal columns regardless of variant.
    pub fn universal(&self) -> &UniversalColumns {
        match self {
            AnyRelationNode::Contains(n) => n.universal(),
            AnyRelationNode::Source(n) => n.universal(),
            AnyRelationNode::Similar(n) => n.universal(),
            AnyRelationNode::HasProperty(n) => n.universal(),
            AnyRelationNode::LeadsTo(n) => n.universal(),
            AnyRelationNode::Prevents(n) => n.universal(),
            AnyRelationNode::Causes(n) => n.universal(),
            AnyRelationNode::BecauseOf(n) => n.universal(),
            AnyRelationNode::Before(n) => n.universal(),
            AnyRelationNode::After(n) => n.universal(),
            AnyRelationNode::During(n) => n.universal(),
            AnyRelationNode::ValidFrom(n) => n.universal(),
            AnyRelationNode::ValidTo(n) => n.universal(),
        }
    }

    /// Return the edge type discriminant regardless of variant.
    pub fn edge_type(&self) -> EdgeNodeType {
        match self {
            AnyRelationNode::Contains(n) => n.edge_type(),
            AnyRelationNode::Source(n) => n.edge_type(),
            AnyRelationNode::Similar(n) => n.edge_type(),
            AnyRelationNode::HasProperty(n) => n.edge_type(),
            AnyRelationNode::LeadsTo(n) => n.edge_type(),
            AnyRelationNode::Prevents(n) => n.edge_type(),
            AnyRelationNode::Causes(n) => n.edge_type(),
            AnyRelationNode::BecauseOf(n) => n.edge_type(),
            AnyRelationNode::Before(n) => n.edge_type(),
            AnyRelationNode::After(n) => n.edge_type(),
            AnyRelationNode::During(n) => n.edge_type(),
            AnyRelationNode::ValidFrom(n) => n.edge_type(),
            AnyRelationNode::ValidTo(n) => n.edge_type(),
        }
    }
}
