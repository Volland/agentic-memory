use tracing::debug;

use alan_core::entity::{AnyContentNode, NodeType};
use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::Result;

/// A group of related memories that could potentially be merged into a
/// higher-level summary. This is a suggestion only -- it does not perform
/// the actual merge.
#[derive(Debug, Clone)]
pub struct ConsolidationCandidate {
    /// IDs of the nodes that could be consolidated.
    pub node_ids: Vec<NodeId>,
    /// Labels of the nodes for display purposes.
    pub labels: Vec<String>,
    /// Reason this group was identified.
    pub reason: String,
    /// Estimated benefit of consolidation (0.0 - 1.0).
    pub benefit_score: f64,
}

/// Configuration for consolidation candidate detection.
pub struct ConsolidationCandidateConfig {
    /// Minimum number of related nodes to form a candidate group.
    pub min_group_size: usize,
    /// Maximum number of candidate groups to return.
    pub max_candidates: usize,
}

impl Default for ConsolidationCandidateConfig {
    fn default() -> Self {
        Self {
            min_group_size: 3,
            max_candidates: 10,
        }
    }
}

/// Step that identifies groups of related memories/facts/events that could be
/// merged into a higher-level summary. Returns suggestions without modifying
/// the graph.
pub struct ConsolidationCandidateStep {
    pub config: ConsolidationCandidateConfig,
}

impl ConsolidationCandidateStep {
    pub fn new() -> Self {
        Self {
            config: ConsolidationCandidateConfig::default(),
        }
    }

    pub fn with_config(config: ConsolidationCandidateConfig) -> Self {
        Self { config }
    }

    pub async fn execute(
        &self,
        ctx: &CognitiveContext,
    ) -> Result<Vec<ConsolidationCandidate>> {
        let mut candidates: Vec<ConsolidationCandidate> = Vec::new();

        // Strategy 1: Find clusters of nodes connected by Similar edges.
        let similar_groups = find_similar_clusters(ctx, self.config.min_group_size);
        for group in similar_groups {
            let labels: Vec<String> = group
                .iter()
                .filter_map(|id| {
                    ctx.resolved_nodes
                        .iter()
                        .find(|n| n.universal().id == *id)
                        .map(|n| n.universal().label.clone())
                })
                .collect();

            if labels.len() >= self.config.min_group_size {
                candidates.push(ConsolidationCandidate {
                    node_ids: group,
                    labels,
                    reason: "Connected by similarity edges".into(),
                    benefit_score: 0.7,
                });
            }
        }

        // Strategy 2: Find facts with low certainty that share subjects.
        let low_certainty_groups = find_low_certainty_clusters(ctx, self.config.min_group_size);
        for (subject, group) in low_certainty_groups {
            let labels: Vec<String> = group
                .iter()
                .filter_map(|id| {
                    ctx.resolved_nodes
                        .iter()
                        .find(|n| n.universal().id == *id)
                        .map(|n| n.universal().label.clone())
                })
                .collect();

            if labels.len() >= self.config.min_group_size {
                candidates.push(ConsolidationCandidate {
                    node_ids: group,
                    labels,
                    reason: format!("Low-certainty facts about '{subject}'"),
                    benefit_score: 0.5,
                });
            }
        }

        // Sort by benefit and truncate.
        candidates.sort_by(|a, b| {
            b.benefit_score
                .partial_cmp(&a.benefit_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(self.config.max_candidates);

        debug!(
            candidates = candidates.len(),
            "Consolidation candidates identified"
        );

        Ok(candidates)
    }
}

/// Find clusters of nodes connected by Similar edges.
fn find_similar_clusters(ctx: &CognitiveContext, min_size: usize) -> Vec<Vec<NodeId>> {
    use alan_core::relation::EdgeNodeType;
    use std::collections::{HashMap, HashSet};

    // Build adjacency list from Similar edges.
    let mut adjacency: HashMap<NodeId, HashSet<NodeId>> = HashMap::new();

    for (edge, _) in &ctx.resolved_edges {
        if edge.edge_type == EdgeNodeType::Similar {
            adjacency
                .entry(edge.from_node.clone())
                .or_default()
                .insert(edge.to_node.clone());
            adjacency
                .entry(edge.to_node.clone())
                .or_default()
                .insert(edge.from_node.clone());
        }
    }

    // Simple connected-components via BFS.
    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut clusters: Vec<Vec<NodeId>> = Vec::new();

    for start in adjacency.keys() {
        if visited.contains(start) {
            continue;
        }

        let mut component: Vec<NodeId> = Vec::new();
        let mut queue: Vec<NodeId> = vec![start.clone()];

        while let Some(current) = queue.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            component.push(current.clone());
            if let Some(neighbors) = adjacency.get(&current) {
                for neighbor in neighbors {
                    if !visited.contains(neighbor) {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        if component.len() >= min_size {
            clusters.push(component);
        }
    }

    clusters
}

/// Find groups of low-certainty facts that share similar labels (rough subject
/// clustering by label prefix).
fn find_low_certainty_clusters(
    ctx: &CognitiveContext,
    min_size: usize,
) -> Vec<(String, Vec<NodeId>)> {
    use std::collections::HashMap;

    let mut subject_groups: HashMap<String, Vec<NodeId>> = HashMap::new();

    for node in &ctx.resolved_nodes {
        if node.node_type() != NodeType::Fact {
            continue;
        }

        let certainty = if let AnyContentNode::Fact(f) = node {
            f.certainty
        } else {
            continue;
        };

        if certainty > 0.5 {
            continue;
        }

        // Use the first significant word of the label as a rough subject key.
        let subject = node
            .universal()
            .label
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_lowercase();

        subject_groups
            .entry(subject)
            .or_default()
            .push(node.universal().id.clone());
    }

    subject_groups
        .into_iter()
        .filter(|(_, ids)| ids.len() >= min_size)
        .collect()
}
