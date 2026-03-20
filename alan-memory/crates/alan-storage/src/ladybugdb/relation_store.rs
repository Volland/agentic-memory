use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use alan_core::graph::traits::RelationStore;
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::id::NodeId;
use alan_core::relation::{AnyRelationNode, EdgeNodeType};

/// In-memory implementation of RelationStore for testing and development.
pub struct InMemoryRelationStore {
    /// Edges keyed by edge_node id.
    edges: Mutex<HashMap<String, BipartiteEdge>>,
    /// Relation nodes keyed by id.
    relations: Mutex<HashMap<String, AnyRelationNode>>,
}

impl InMemoryRelationStore {
    pub fn new() -> Self {
        Self {
            edges: Mutex::new(HashMap::new()),
            relations: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryRelationStore {
    fn default() -> Self {
        Self::new()
    }
}

fn lock_err(e: impl std::fmt::Display) -> alan_core::AlanError {
    alan_core::AlanError::Validation(format!("Lock poisoned: {e}"))
}

#[async_trait]
impl RelationStore for InMemoryRelationStore {
    async fn create_edge(
        &self,
        edge: &BipartiteEdge,
        relation: &AnyRelationNode,
    ) -> alan_core::Result<()> {
        let edge_key = edge.edge_node.as_str().to_string();
        let rel_key = relation.universal().id.as_str().to_string();

        let mut edges = self.edges.lock().map_err(lock_err)?;
        let mut relations = self.relations.lock().map_err(lock_err)?;

        if edges.contains_key(&edge_key) {
            return Err(alan_core::AlanError::Validation(format!(
                "Edge already exists: {edge_key}"
            )));
        }

        edges.insert(edge_key, edge.clone());
        relations.insert(rel_key, relation.clone());
        Ok(())
    }

    async fn get_edges_from(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> alan_core::Result<Vec<(BipartiteEdge, AnyRelationNode)>> {
        let edges = self.edges.lock().map_err(lock_err)?;
        let relations = self.relations.lock().map_err(lock_err)?;

        let node_key = node_id.as_str();
        let results = edges
            .values()
            .filter(|e| {
                e.from_node.as_str() == node_key
                    && edge_type.map(|t| e.edge_type == t).unwrap_or(true)
            })
            .filter_map(|e| {
                let rel_key = e.edge_node.as_str();
                relations.get(rel_key).map(|r| (e.clone(), r.clone()))
            })
            .collect();
        Ok(results)
    }

    async fn get_edges_to(
        &self,
        node_id: &NodeId,
        edge_type: Option<EdgeNodeType>,
    ) -> alan_core::Result<Vec<(BipartiteEdge, AnyRelationNode)>> {
        let edges = self.edges.lock().map_err(lock_err)?;
        let relations = self.relations.lock().map_err(lock_err)?;

        let node_key = node_id.as_str();
        let results = edges
            .values()
            .filter(|e| {
                e.to_node.as_str() == node_key
                    && edge_type.map(|t| e.edge_type == t).unwrap_or(true)
            })
            .filter_map(|e| {
                let rel_key = e.edge_node.as_str();
                relations.get(rel_key).map(|r| (e.clone(), r.clone()))
            })
            .collect();
        Ok(results)
    }

    async fn get_relation(&self, id: &NodeId) -> alan_core::Result<Option<AnyRelationNode>> {
        let relations = self.relations.lock().map_err(lock_err)?;
        Ok(relations.get(id.as_str()).cloned())
    }

    async fn update_relation(&self, relation: &AnyRelationNode) -> alan_core::Result<()> {
        let mut relations = self.relations.lock().map_err(lock_err)?;
        let key = relation.universal().id.as_str().to_string();
        if !relations.contains_key(&key) {
            return Err(alan_core::AlanError::NotFound(format!(
                "Relation not found: {key}"
            )));
        }
        relations.insert(key, relation.clone());
        Ok(())
    }
}
