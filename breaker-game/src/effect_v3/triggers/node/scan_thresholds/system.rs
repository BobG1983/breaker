//! Scan system — populates `NodeTimerThresholdRegistry` from `BoundEffects` trees.

use bevy::prelude::*;

use super::super::resources::NodeTimerThresholdRegistry;
use crate::effect_v3::storage::BoundEffects;

/// Scans all `BoundEffects` trees for `NodeTimerThresholdOccurred` triggers
/// and populates `NodeTimerThresholdRegistry.thresholds` with unique ratios.
///
/// Clears thresholds before repopulating — idempotent.
pub fn scan_threshold_triggers(
    query: Query<&BoundEffects>,
    mut registry: ResMut<NodeTimerThresholdRegistry>,
) {
    registry.thresholds.clear();

    let mut seen = std::collections::HashSet::new();

    for bound in &query {
        for (_name, tree) in &bound.0 {
            collect_thresholds_from_tree(tree, &mut seen);
        }
    }

    registry.thresholds.extend(seen);
}

fn collect_thresholds_from_tree(
    tree: &crate::effect_v3::types::Tree,
    seen: &mut std::collections::HashSet<ordered_float::OrderedFloat<f32>>,
) {
    use crate::effect_v3::types::Tree;

    match tree {
        Tree::Fire(_) => {}
        Tree::When(trigger, inner) | Tree::Once(trigger, inner) => {
            collect_trigger(trigger, seen);
            collect_thresholds_from_tree(inner, seen);
        }
        Tree::During(_condition, scoped) => {
            collect_thresholds_from_scoped_tree(scoped, seen);
        }
        Tree::Until(trigger, scoped) => {
            collect_trigger(trigger, seen);
            collect_thresholds_from_scoped_tree(scoped, seen);
        }
        Tree::Sequence(terminals) => {
            for terminal in terminals {
                collect_thresholds_from_terminal(terminal, seen);
            }
        }
        Tree::On(_target, terminal) => {
            collect_thresholds_from_terminal(terminal, seen);
        }
    }
}

fn collect_thresholds_from_scoped_tree(
    scoped: &crate::effect_v3::types::ScopedTree,
    seen: &mut std::collections::HashSet<ordered_float::OrderedFloat<f32>>,
) {
    use crate::effect_v3::types::ScopedTree;

    match scoped {
        ScopedTree::Fire(_) | ScopedTree::Sequence(_) => {}
        ScopedTree::When(trigger, inner) => {
            collect_trigger(trigger, seen);
            collect_thresholds_from_tree(inner, seen);
        }
        ScopedTree::On(_target, scoped_terminal) => {
            collect_thresholds_from_scoped_terminal(scoped_terminal, seen);
        }
        ScopedTree::During(_condition, inner) => {
            collect_thresholds_from_scoped_tree(inner, seen);
        }
    }
}

fn collect_thresholds_from_terminal(
    terminal: &crate::effect_v3::types::Terminal,
    seen: &mut std::collections::HashSet<ordered_float::OrderedFloat<f32>>,
) {
    use crate::effect_v3::types::Terminal;

    match terminal {
        Terminal::Fire(_) => {}
        Terminal::Route(_route_type, tree) => {
            collect_thresholds_from_tree(tree, seen);
        }
    }
}

fn collect_thresholds_from_scoped_terminal(
    scoped_terminal: &crate::effect_v3::types::ScopedTerminal,
    seen: &mut std::collections::HashSet<ordered_float::OrderedFloat<f32>>,
) {
    use crate::effect_v3::types::ScopedTerminal;

    match scoped_terminal {
        ScopedTerminal::Fire(_) => {}
        ScopedTerminal::Route(_route_type, tree) => {
            collect_thresholds_from_tree(tree, seen);
        }
    }
}

fn collect_trigger(
    trigger: &crate::effect_v3::types::Trigger,
    seen: &mut std::collections::HashSet<ordered_float::OrderedFloat<f32>>,
) {
    use crate::effect_v3::types::Trigger;

    if let Trigger::NodeTimerThresholdOccurred(ratio) = trigger {
        seen.insert(*ratio);
    }
}
