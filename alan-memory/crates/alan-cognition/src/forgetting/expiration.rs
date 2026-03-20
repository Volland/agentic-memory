use chrono::{Duration, Utc};
use tracing::debug;

use alan_core::NodeId;

use crate::context::CognitiveContext;
use crate::error::Result;

/// Configuration for expiration.
pub struct ExpirationConfig {
    /// Relevance score below which a node is considered for expiration.
    pub relevance_threshold: f64,
    /// Grace period before actual expiration (hours).
    pub grace_period_hours: i64,
}

impl Default for ExpirationConfig {
    fn default() -> Self {
        Self {
            relevance_threshold: 0.2,
            grace_period_hours: 72, // 3 days
        }
    }
}

/// Step that sets `expire_at` on low-relevance nodes. NEVER deletes nodes --
/// only marks them with an expiration timestamp.
pub struct ExpirationStep {
    pub config: ExpirationConfig,
}

impl ExpirationStep {
    pub fn new() -> Self {
        Self {
            config: ExpirationConfig::default(),
        }
    }

    pub fn with_config(config: ExpirationConfig) -> Self {
        Self { config }
    }

    pub async fn execute(
        &self,
        ctx: &mut CognitiveContext,
        scores: &[(NodeId, f64)],
    ) -> Result<usize> {
        let now = Utc::now();
        let grace = Duration::hours(self.config.grace_period_hours);
        let expire_at = now + grace;
        let mut expired_count = 0;

        for (node_id, score) in scores {
            if *score >= self.config.relevance_threshold {
                continue;
            }

            // Find the node and set expire_at if not already set.
            if let Some(node) = ctx
                .resolved_nodes
                .iter_mut()
                .find(|n| n.universal().id == *node_id)
            {
                let universal = node.universal_mut();
                if universal.expire_at.is_none() {
                    universal.set_expiration(expire_at);
                    expired_count += 1;
                }
            }
        }

        debug!(
            expired = expired_count,
            threshold = self.config.relevance_threshold,
            "Expiration step complete — nodes marked (never deleted)"
        );

        Ok(expired_count)
    }
}
