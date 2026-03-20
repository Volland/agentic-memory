use chrono::{Datelike, NaiveDate, TimeZone, Utc};

use crate::entity::time::{Time, TimeGranularity};
use crate::graph::wiring::BipartiteEdge;
use crate::entity::NodeType;
use crate::relation::contains::{Contains, ContainmentType};
use crate::relation::{AnyRelationNode, EdgeNodeType};

/// Builder for the time tree hierarchy: Year → Month → Week → Day.
/// Each level is a Time node connected by Contains edges.
pub struct TimeTreeBuilder;

/// Result of building a time tree for a year.
pub struct TimeTree {
    /// All time nodes (year, months, days).
    pub nodes: Vec<Time>,
    /// Contains edges wiring the hierarchy.
    pub edges: Vec<(BipartiteEdge, AnyRelationNode)>,
}

impl TimeTreeBuilder {
    /// Build the time tree for a given year.
    /// Creates: 1 year node, 12 month nodes, ~365 day nodes.
    /// Wires: year→month, month→day via Contains edges.
    pub fn build_year(year: i32) -> TimeTree {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        // Year node
        let year_start = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let year_end = Utc.with_ymd_and_hms(year, 12, 31, 23, 59, 59).unwrap();
        let year_node = Time::new(format!("{year}"), TimeGranularity::Year)
            .with_bounds(year_start, year_end);
        let year_id = year_node.universal.id.clone();
        nodes.push(year_node);

        // Month nodes
        for month in 1..=12u32 {
            let month_start = Utc.with_ymd_and_hms(year, month, 1, 0, 0, 0).unwrap();
            let last_day = last_day_of_month(year, month);
            let month_end = Utc
                .with_ymd_and_hms(year, month, last_day, 23, 59, 59)
                .unwrap();

            let month_label = format!("{year}-{month:02}");
            let month_node =
                Time::new(&month_label, TimeGranularity::Month).with_bounds(month_start, month_end);
            let month_id = month_node.universal.id.clone();
            nodes.push(month_node);

            // Wire year → month
            let contains = Contains::new(format!("{year} contains {month_label}"))
                .with_containment_type(ContainmentType::Composition);
            let contains_id = contains.universal.id.clone();
            let edge = BipartiteEdge::new_unchecked(
                year_id.clone(),
                NodeType::Time,
                contains_id,
                EdgeNodeType::Contains,
                month_id.clone(),
                NodeType::Time,
            );
            edges.push((edge, AnyRelationNode::Contains(contains)));

            // Day nodes for this month
            for day in 1..=last_day {
                let day_start = Utc.with_ymd_and_hms(year, month, day, 0, 0, 0).unwrap();
                let day_end = Utc
                    .with_ymd_and_hms(year, month, day, 23, 59, 59)
                    .unwrap();

                let day_label = format!("{year}-{month:02}-{day:02}");
                let day_node =
                    Time::new(&day_label, TimeGranularity::Day).with_bounds(day_start, day_end);
                let day_id = day_node.universal.id.clone();
                nodes.push(day_node);

                // Wire month → day
                let contains = Contains::new(format!("{month_label} contains {day_label}"))
                    .with_containment_type(ContainmentType::Composition);
                let contains_id = contains.universal.id.clone();
                let edge = BipartiteEdge::new_unchecked(
                    month_id.clone(),
                    NodeType::Time,
                    contains_id,
                    EdgeNodeType::Contains,
                    day_id,
                    NodeType::Time,
                );
                edges.push((edge, AnyRelationNode::Contains(contains)));
            }
        }

        TimeTree { nodes, edges }
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        31
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap()
            .pred_opt()
            .unwrap()
            .day()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_year_2024() {
        let tree = TimeTreeBuilder::build_year(2024);

        // 1 year + 12 months + 366 days (2024 is a leap year)
        assert_eq!(tree.nodes.len(), 1 + 12 + 366);

        // 12 year→month + 366 month→day
        assert_eq!(tree.edges.len(), 12 + 366);

        // Check year node
        assert_eq!(tree.nodes[0].universal.label, "2024");
        assert_eq!(tree.nodes[0].granularity, TimeGranularity::Year);

        // Check first month
        assert_eq!(tree.nodes[1].universal.label, "2024-01");
        assert_eq!(tree.nodes[1].granularity, TimeGranularity::Month);
    }

    #[test]
    fn test_build_year_2023() {
        let tree = TimeTreeBuilder::build_year(2023);
        // 1 year + 12 months + 365 days (2023 is not a leap year)
        assert_eq!(tree.nodes.len(), 1 + 12 + 365);
    }
}
