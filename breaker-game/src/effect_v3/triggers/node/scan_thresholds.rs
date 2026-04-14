//! Scan system — populates `NodeTimerThresholdRegistry` from `BoundEffects` trees.

use bevy::prelude::*;

use super::resources::NodeTimerThresholdRegistry;
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

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::{
            effects::SpeedBoostConfig,
            storage::BoundEffects,
            triggers::node::resources::NodeTimerThresholdRegistry,
            types::{EffectType, ScopedTree, Tree, Trigger},
        },
        shared::test_utils::{TestAppBuilder, tick},
    };

    fn scan_test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<NodeTimerThresholdRegistry>()
            .with_system(FixedUpdate, scan_threshold_triggers)
            .build()
    }

    fn make_speed_boost() -> EffectType {
        EffectType::SpeedBoost(SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        })
    }

    // ── B3-1: Scan collects unique threshold ratios from BoundEffects ──

    #[test]
    fn scan_collects_unique_threshold_ratios_from_bound_effects() {
        let mut app = scan_test_app();

        // Entity A: threshold at 0.5
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.5)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        // Entity B: threshold at 0.75
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_b".to_string(),
            Tree::When(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.75)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert_eq!(
            registry.thresholds.len(),
            2,
            "should contain exactly 2 unique thresholds, got {:?}",
            registry.thresholds,
        );
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.5)),
            "thresholds should contain 0.5, got {:?}",
            registry.thresholds,
        );
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.75)),
            "thresholds should contain 0.75, got {:?}",
            registry.thresholds,
        );
    }

    #[test]
    fn scan_deduplicates_same_threshold_from_multiple_entities() {
        let mut app = scan_test_app();

        // Entity A: threshold at 0.5
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.5)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        // Entity B: same threshold at 0.5
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_b".to_string(),
            Tree::When(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.5)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        let count_half = registry
            .thresholds
            .iter()
            .filter(|t| **t == OrderedFloat(0.5))
            .count();
        assert_eq!(
            count_half, 1,
            "duplicate threshold 0.5 should appear only once, got {count_half}",
        );
    }

    // ── B3-2: Scan finds thresholds nested inside When and Until gates ──

    #[test]
    fn scan_finds_threshold_nested_inside_when_chain() {
        let mut app = scan_test_app();

        // Nested: When(BumpOccurred, When(NodeTimerThreshold(0.25), Fire(...)))
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::BumpOccurred,
                Box::new(Tree::When(
                    Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.25)),
                    Box::new(Tree::Fire(make_speed_boost())),
                )),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.25)),
            "nested threshold 0.25 should be collected, got {:?}",
            registry.thresholds,
        );
    }

    #[test]
    fn scan_finds_threshold_inside_until_gate() {
        use crate::effect_v3::types::ReversibleEffectType;

        let mut app = scan_test_app();

        // Until(NodeTimerThreshold(0.9), Fire(SpeedBoost(...)))
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Until(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.9)),
                Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(
                    SpeedBoostConfig {
                        multiplier: OrderedFloat(1.5),
                    },
                ))),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.9)),
            "Until gate threshold 0.9 should be collected, got {:?}",
            registry.thresholds,
        );
    }

    // ── B3-3: Scan handles entities with no threshold triggers ──

    #[test]
    fn scan_handles_no_threshold_triggers() {
        let mut app = scan_test_app();

        // Entity with only BumpOccurred trigger — no threshold
        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::BumpOccurred,
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.is_empty(),
            "thresholds should be empty when no threshold triggers exist, got {:?}",
            registry.thresholds,
        );
    }

    #[test]
    fn scan_handles_no_entities_with_bound_effects() {
        let mut app = scan_test_app();

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.is_empty(),
            "thresholds should remain empty when no entities exist, got {:?}",
            registry.thresholds,
        );
    }

    // ── B3-4: Scan handles empty BoundEffects ──

    #[test]
    fn scan_handles_empty_bound_effects() {
        let mut app = scan_test_app();

        app.world_mut().spawn(BoundEffects(vec![]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.is_empty(),
            "thresholds should be empty for empty BoundEffects, got {:?}",
            registry.thresholds,
        );
    }

    // ── B3-6: Scan finds thresholds inside Once variant ──

    #[test]
    fn scan_finds_threshold_inside_once_variant() {
        let mut app = scan_test_app();

        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::Once(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.1)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.1)),
            "Once variant threshold 0.1 should be collected, got {:?}",
            registry.thresholds,
        );
    }

    // ── B3-7: Scan is idempotent ──

    #[test]
    fn scan_is_idempotent_second_run_produces_same_result() {
        let mut app = scan_test_app();

        app.world_mut().spawn(BoundEffects(vec![(
            "chip_a".to_string(),
            Tree::When(
                Trigger::NodeTimerThresholdOccurred(OrderedFloat(0.5)),
                Box::new(Tree::Fire(make_speed_boost())),
            ),
        )]));

        // First scan
        tick(&mut app);

        let count_after_first = app
            .world()
            .resource::<NodeTimerThresholdRegistry>()
            .thresholds
            .len();
        assert_eq!(
            count_after_first, 1,
            "first scan should collect 1 threshold"
        );

        // Second scan
        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert_eq!(
            registry.thresholds.len(),
            1,
            "second scan should still have exactly 1 threshold (idempotent), got {}",
            registry.thresholds.len(),
        );
        assert!(
            registry.thresholds.contains(&OrderedFloat(0.5)),
            "threshold 0.5 should still be present after second scan",
        );
    }
}
