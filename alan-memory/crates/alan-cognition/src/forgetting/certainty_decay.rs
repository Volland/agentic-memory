use chrono::Utc;
use tracing::debug;

use alan_core::entity::AnyContentNode;

use crate::context::CognitiveContext;
use crate::error::Result;

/// Configuration for certainty decay.
pub struct CertaintyDecayConfig {
    /// Multiplicative decay factor applied each cycle (e.g. 0.95 = 5% decay).
    pub decay_rate: f64,
    /// Floor below which certainty will never drop.
    pub min_certainty: f64,
    /// Minimum hours since last update before decay is applied.
    pub min_hours_since_update: i64,
}

impl Default for CertaintyDecayConfig {
    fn default() -> Self {
        Self {
            decay_rate: 0.95,
            min_certainty: 0.1,
            min_hours_since_update: 24,
        }
    }
}

/// Step that reduces certainty on Fact (and Event/Memory) nodes that have not
/// been recently reinforced.
pub struct CertaintyDecayStep {
    pub config: CertaintyDecayConfig,
}

impl CertaintyDecayStep {
    pub fn new() -> Self {
        Self {
            config: CertaintyDecayConfig::default(),
        }
    }

    pub fn with_config(config: CertaintyDecayConfig) -> Self {
        Self { config }
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<usize> {
        let now = Utc::now();
        let mut decayed_count = 0;

        for node in &mut ctx.resolved_nodes {
            let should_decay = match node {
                AnyContentNode::Fact(f) => {
                    let hours = (now - f.universal.updated_at).num_hours();
                    hours >= self.config.min_hours_since_update && f.certainty > self.config.min_certainty
                }
                AnyContentNode::Event(e) => {
                    let hours = (now - e.universal.updated_at).num_hours();
                    hours >= self.config.min_hours_since_update && e.certainty > self.config.min_certainty
                }
                AnyContentNode::Memory(m) => {
                    let hours = (now - m.universal.updated_at).num_hours();
                    hours >= self.config.min_hours_since_update && m.certainty > self.config.min_certainty
                }
                _ => false,
            };

            if !should_decay {
                continue;
            }

            match node {
                AnyContentNode::Fact(f) => {
                    f.certainty = (f.certainty * self.config.decay_rate)
                        .max(self.config.min_certainty);
                    f.universal.updated_at = now;
                    decayed_count += 1;
                }
                AnyContentNode::Event(e) => {
                    e.certainty = (e.certainty * self.config.decay_rate)
                        .max(self.config.min_certainty);
                    e.universal.updated_at = now;
                    decayed_count += 1;
                }
                AnyContentNode::Memory(m) => {
                    m.certainty = (m.certainty * self.config.decay_rate)
                        .max(self.config.min_certainty);
                    m.universal.updated_at = now;
                    decayed_count += 1;
                }
                _ => {}
            }
        }

        debug!(
            decayed = decayed_count,
            decay_rate = self.config.decay_rate,
            "Certainty decay step complete"
        );

        Ok(decayed_count)
    }
}
