use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;

use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::graph::traits::NodeStore;
use alan_core::id::NodeId;

/// In-memory implementation of NodeStore for testing and development.
pub struct InMemoryNodeStore {
    nodes: Mutex<HashMap<String, AnyContentNode>>,
}

impl InMemoryNodeStore {
    pub fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryNodeStore {
    fn default() -> Self {
        Self::new()
    }
}

fn lock_err(e: impl std::fmt::Display) -> alan_core::AlanError {
    alan_core::AlanError::Validation(format!("Lock poisoned: {e}"))
}

#[async_trait]
impl NodeStore for InMemoryNodeStore {
    async fn create_node(&self, node: &AnyContentNode) -> alan_core::Result<NodeId> {
        let mut store = self.nodes.lock().map_err(lock_err)?;
        let id = node.universal().id.clone();
        let key = id.as_str().to_string();
        if store.contains_key(&key) {
            return Err(alan_core::AlanError::Validation(format!(
                "Node already exists: {key}"
            )));
        }
        store.insert(key, node.clone());
        Ok(id)
    }

    async fn get_node(&self, id: &NodeId) -> alan_core::Result<Option<AnyContentNode>> {
        let store = self.nodes.lock().map_err(lock_err)?;
        Ok(store.get(id.as_str()).cloned())
    }

    async fn update_node(&self, node: &AnyContentNode) -> alan_core::Result<()> {
        let mut store = self.nodes.lock().map_err(lock_err)?;
        let key = node.universal().id.as_str().to_string();
        if !store.contains_key(&key) {
            return Err(alan_core::AlanError::NotFound(format!(
                "Node not found: {key}"
            )));
        }
        store.insert(key, node.clone());
        Ok(())
    }

    async fn delete_node(&self, id: &NodeId) -> alan_core::Result<()> {
        let mut store = self.nodes.lock().map_err(lock_err)?;
        let key = id.as_str();
        if store.remove(key).is_none() {
            return Err(alan_core::AlanError::NotFound(format!(
                "Node not found: {key}"
            )));
        }
        Ok(())
    }

    async fn find_by_label(
        &self,
        label: &str,
        node_type: Option<NodeType>,
    ) -> alan_core::Result<Vec<AnyContentNode>> {
        let store = self.nodes.lock().map_err(lock_err)?;
        let label_lower = label.to_lowercase();
        let results = store
            .values()
            .filter(|n| {
                let matches_label = n.universal().label.to_lowercase().contains(&label_lower)
                    || n.universal()
                        .label_resolved
                        .as_ref()
                        .map(|r| r.to_lowercase().contains(&label_lower))
                        .unwrap_or(false);
                let matches_type = node_type.map(|t| n.node_type() == t).unwrap_or(true);
                matches_label && matches_type && !n.universal().is_expired()
            })
            .cloned()
            .collect();
        Ok(results)
    }

    async fn list_by_type(&self, node_type: NodeType) -> alan_core::Result<Vec<AnyContentNode>> {
        let store = self.nodes.lock().map_err(lock_err)?;
        let results = store
            .values()
            .filter(|n| n.node_type() == node_type && !n.universal().is_expired())
            .cloned()
            .collect();
        Ok(results)
    }
}
