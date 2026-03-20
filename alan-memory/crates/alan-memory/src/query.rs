use std::collections::{HashSet, VecDeque};

use alan_core::entity::AnyContentNode;
use alan_core::graph::traits::{NodeStore, RelationStore};
use alan_core::graph::wiring::BipartiteEdge;
use alan_core::id::NodeId;
use alan_core::layer::Layer;
use alan_core::relation::AnyRelationNode;

use crate::error::{MemoryError, Result};

/// A subgraph extracted from the memory graph via traversal.
#[derive(Debug, Clone)]
pub struct Subgraph {
    /// Content nodes in the subgraph.
    pub nodes: Vec<AnyContentNode>,
    /// Edges (with their relation nodes) in the subgraph.
    pub edges: Vec<(BipartiteEdge, AnyRelationNode)>,
}

impl Subgraph {
    /// Create an empty subgraph.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl Default for Subgraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Breadth-first traversal from a starting node, collecting neighbors up to
/// the given depth, optionally filtering by ontological layer.
///
/// Returns a Subgraph containing all discovered nodes and the edges
/// connecting them.
pub async fn traverse(
    nodes: &dyn NodeStore,
    relations: &dyn RelationStore,
    start: &NodeId,
    depth: usize,
    layer_filter: Option<Layer>,
) -> Result<Subgraph> {
    let mut subgraph = Subgraph::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();

    // Seed with the start node.
    if let Some(start_node) = nodes
        .get_node(start)
        .await
        .map_err(|e| MemoryError::Storage(e.to_string()))?
    {
        let passes_filter = layer_filter
            .map(|l| start_node.universal().layer == l)
            .unwrap_or(true);

        if passes_filter {
            visited.insert(start.as_str().to_string());
            subgraph.nodes.push(start_node);
            queue.push_back((start.clone(), 0));
        }
    } else {
        return Ok(subgraph);
    }

    while let Some((current_id, current_depth)) = queue.pop_front() {
        if current_depth >= depth {
            continue;
        }

        // Get outgoing edges.
        let outgoing = relations
            .get_edges_from(&current_id, None)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

        for (edge, relation) in &outgoing {
            let neighbor_id = &edge.to_node;
            let neighbor_key = neighbor_id.as_str().to_string();

            if visited.contains(&neighbor_key) {
                continue;
            }

            if let Some(neighbor_node) = nodes
                .get_node(neighbor_id)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?
            {
                let passes_filter = layer_filter
                    .map(|l| neighbor_node.universal().layer == l)
                    .unwrap_or(true);

                if passes_filter {
                    visited.insert(neighbor_key);
                    subgraph.nodes.push(neighbor_node);
                    subgraph.edges.push((edge.clone(), relation.clone()));
                    queue.push_back((neighbor_id.clone(), current_depth + 1));
                }
            }
        }

        // Get incoming edges.
        let incoming = relations
            .get_edges_to(&current_id, None)
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;

        for (edge, relation) in &incoming {
            let neighbor_id = &edge.from_node;
            let neighbor_key = neighbor_id.as_str().to_string();

            if visited.contains(&neighbor_key) {
                continue;
            }

            if let Some(neighbor_node) = nodes
                .get_node(neighbor_id)
                .await
                .map_err(|e| MemoryError::Storage(e.to_string()))?
            {
                let passes_filter = layer_filter
                    .map(|l| neighbor_node.universal().layer == l)
                    .unwrap_or(true);

                if passes_filter {
                    visited.insert(neighbor_key);
                    subgraph.nodes.push(neighbor_node);
                    subgraph.edges.push((edge.clone(), relation.clone()));
                    queue.push_back((neighbor_id.clone(), current_depth + 1));
                }
            }
        }
    }

    Ok(subgraph)
}
