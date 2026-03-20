use tracing::debug;

use alan_core::entity::{AbstractTime, AnyContentNode, Time, TimeGranularity};

use crate::context::CognitiveContext;
use crate::error::Result;

/// Step that resolves extracted temporal references into concrete Time or
/// AbstractTime content nodes.
pub struct TimeResolutionStep;

impl TimeResolutionStep {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, ctx: &mut CognitiveContext) -> Result<()> {
        if ctx.extracted_temporal_refs.is_empty() {
            return Ok(());
        }

        debug!(
            count = ctx.extracted_temporal_refs.len(),
            "Resolving temporal references to Time/AbstractTime nodes"
        );

        for tref in &ctx.extracted_temporal_refs {
            match tref.temporal_type.as_str() {
                "concrete" => {
                    // Attempt to create a concrete Time node from the expression.
                    let time_node = resolve_concrete_time(&tref.expression);
                    ctx.resolved_nodes.push(AnyContentNode::Time(time_node));
                }
                "abstract" => {
                    let abstract_node = resolve_abstract_time(&tref.expression);
                    ctx.resolved_nodes
                        .push(AnyContentNode::AbstractTime(abstract_node));
                }
                "relative" => {
                    // Relative expressions (e.g. "next week", "yesterday") are
                    // treated as concrete when possible, abstract otherwise.
                    let node = resolve_relative_time(&tref.expression);
                    ctx.resolved_nodes.push(node);
                }
                _ => {
                    // Default to abstract for unknown types.
                    let abstract_node = AbstractTime::new(
                        &tref.expression,
                        format!("unresolved temporal: {}", tref.expression),
                    );
                    ctx.resolved_nodes
                        .push(AnyContentNode::AbstractTime(abstract_node));
                }
            }
        }

        debug!("Time resolution complete");
        Ok(())
    }
}

/// Create a concrete Time node from a temporal expression.
/// In a production system this would parse dates; here we create a Day-level
/// node with the expression as its label.
fn resolve_concrete_time(expression: &str) -> Time {
    Time::new(expression, TimeGranularity::Day)
}

/// Map well-known abstract temporal phrases to canonical AbstractTime nodes.
fn resolve_abstract_time(expression: &str) -> AbstractTime {
    let lower = expression.to_lowercase();
    if lower.contains("soon") || lower.contains("shortly") {
        AbstractTime::soon()
    } else if lower.contains("never") {
        AbstractTime::never()
    } else if lower.contains("now") || lower.contains("currently") || lower.contains("right now") {
        AbstractTime::now()
    } else if lower.contains("future") || lower.contains("eventually") || lower.contains("someday")
    {
        AbstractTime::future()
    } else if lower.contains("past") || lower.contains("formerly") || lower.contains("used to") {
        AbstractTime::past()
    } else {
        AbstractTime::new(expression, format!("abstract: {expression}"))
    }
}

/// Attempt to resolve a relative time expression. Falls back to AbstractTime
/// when concrete resolution is not possible.
fn resolve_relative_time(expression: &str) -> AnyContentNode {
    let lower = expression.to_lowercase();

    // Patterns that map to concrete granularities.
    if lower.contains("yesterday")
        || lower.contains("today")
        || lower.contains("tomorrow")
        || lower.contains("last week")
        || lower.contains("next week")
    {
        AnyContentNode::Time(Time::new(expression, TimeGranularity::Day))
    } else if lower.contains("last month") || lower.contains("next month") {
        AnyContentNode::Time(Time::new(expression, TimeGranularity::Month))
    } else if lower.contains("last year") || lower.contains("next year") {
        AnyContentNode::Time(Time::new(expression, TimeGranularity::Year))
    } else {
        // Cannot resolve concretely — treat as abstract.
        AnyContentNode::AbstractTime(AbstractTime::new(
            expression,
            format!("relative: {expression}"),
        ))
    }
}
