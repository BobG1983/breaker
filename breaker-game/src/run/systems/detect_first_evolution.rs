//! System to detect `FirstEvolution` highlight when the player evolves a chip for the first time.

use bevy::prelude::*;

use crate::{
    chips::ChipCatalog,
    run::{messages::HighlightTriggered, resources::*},
    ui::messages::ChipSelected,
};

/// Reads [`ChipSelected`] messages and detects `FirstEvolution` highlight
/// when the selected chip matches an evolution recipe result and it is the
/// first evolution in the run.
///
/// Updates [`HighlightTracker::first_evolution_recorded`] and
/// [`RunStats::evolutions_performed`].
pub(crate) fn detect_first_evolution(
    mut reader: MessageReader<ChipSelected>,
    registry: Option<Res<ChipCatalog>>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    let Some(registry) = registry else {
        // Drain reader to avoid stale messages
        for _msg in reader.read() {}
        return;
    };

    for msg in reader.read() {
        // Check if the selected chip name matches any recipe's result
        let is_evolution = registry
            .recipes()
            .iter()
            .any(|recipe| recipe.result_name == msg.name);

        if !is_evolution {
            continue;
        }

        stats.evolutions_performed += 1;

        if !tracker.first_evolution_recorded {
            tracker.first_evolution_recorded = true;

            // Always emit for juice/VFX feedback
            writer.write(HighlightTriggered {
                kind: HighlightKind::FirstEvolution,
            });

            // Record in stats — selection happens at run-end
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::FirstEvolution,
                node_index: run_state.node_index,
                value: 1.0,
                detail: None,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chips::{ChipCatalog, Recipe, definition::EvolutionIngredient},
        run::{
            definition::HighlightConfig,
            resources::{HighlightKind, RunHighlight},
        },
    };

    #[derive(Resource)]
    struct TestMessages(Vec<ChipSelected>);

    fn enqueue_messages(msg_res: Res<TestMessages>, mut writer: MessageWriter<ChipSelected>) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[derive(Resource, Default)]
    struct CapturedHighlightTriggered(Vec<HighlightTriggered>);

    fn collect_highlight_triggered(
        mut reader: MessageReader<HighlightTriggered>,
        mut captured: ResMut<CapturedHighlightTriggered>,
    ) {
        for msg in reader.read() {
            captured.0.push(msg.clone());
        }
    }

    /// Creates a test `ChipCatalog` with one recipe producing
    /// `"Piercing Barrage"`.
    fn test_chip_registry() -> ChipCatalog {
        let mut registry = ChipCatalog::default();
        registry.insert_recipe(Recipe {
            ingredients: vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }],
            result_name: "Piercing Barrage".to_owned(),
        });
        registry
    }

    /// Uses `Update` schedule (not `FixedUpdate`) since chip selection
    /// happens during the `ChipSelect` game state.
    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            .insert_resource(test_chip_registry())
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                Update,
                (
                    enqueue_messages,
                    detect_first_evolution,
                    collect_highlight_triggered,
                )
                    .chain(),
            );
        app
    }

    // --- Behavior 21: FirstEvolution detected on first evolution chip ---

    #[test]
    fn first_evolution_detected_when_chip_matches_recipe_result() {
        let mut app = test_app();
        // ChipSelected with name matching recipe result "Piercing Barrage"
        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Barrage".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        let first_evo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            first_evo.is_some(),
            "should detect FirstEvolution when selected chip matches recipe result 'Piercing Barrage'"
        );
        assert_eq!(
            stats.evolutions_performed, 1,
            "evolutions_performed should increment to 1"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            tracker.first_evolution_recorded,
            "first_evolution_recorded flag should be set"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered with FirstEvolution kind"
        );
    }

    // --- Behavior 22: NOT detected on second evolution ---

    #[test]
    fn not_detected_on_second_evolution() {
        let mut app = test_app();
        // Pre-set first_evolution_recorded=true, evolutions_performed=1
        {
            let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
            tracker.first_evolution_recorded = true;
        }
        app.world_mut()
            .resource_mut::<RunStats>()
            .evolutions_performed = 1;

        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Barrage".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        let first_evo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            first_evo.is_none(),
            "should NOT detect FirstEvolution when first_evolution_recorded=true"
        );
        assert_eq!(
            stats.evolutions_performed, 2,
            "evolutions_performed should still increment to 2"
        );

        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            msg.is_none(),
            "should NOT emit HighlightTriggered for second evolution"
        );
    }

    // --- Behavior 23: NOT triggered for non-evolution chip ---

    #[test]
    fn not_triggered_for_non_evolution_chip() {
        let mut app = test_app();
        // ChipSelected with name that does NOT match any recipe result
        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Shot".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        let first_evo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            first_evo.is_none(),
            "should NOT detect FirstEvolution when chip name doesn't match any recipe result"
        );
        assert_eq!(
            stats.evolutions_performed, 0,
            "evolutions_performed should remain 0 for non-evolution chip"
        );

        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            !tracker.first_evolution_recorded,
            "first_evolution_recorded should remain false"
        );
    }

    // --- Behavior 18: Graceful without ChipCatalog ---

    #[test]
    fn graceful_without_chip_registry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            // NOTE: No ChipCatalog inserted — Option<Res<ChipCatalog>> is None
            .init_resource::<CapturedHighlightTriggered>()
            .add_systems(
                Update,
                (
                    enqueue_messages,
                    detect_first_evolution,
                    collect_highlight_triggered,
                )
                    .chain(),
            );

        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Barrage".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.is_empty(),
            "should not detect any highlight without ChipCatalog"
        );
        assert_eq!(
            stats.evolutions_performed, 0,
            "evolutions_performed should remain 0"
        );
    }

    // --- Behavior 25: No cap during detection — stored beyond old cap ---

    #[test]
    fn stores_highlight_beyond_old_cap() {
        let mut app = test_app();

        // Pre-fill highlights to old cap of 5
        {
            let mut stats = app.world_mut().resource_mut::<RunStats>();
            for i in 0..5 {
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::MassDestruction,
                    node_index: i,
                    value: 10.0,
                    detail: None,
                });
            }
        }

        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Barrage".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        let first_evo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            first_evo.is_some(),
            "FirstEvolution should be stored even when 5 highlights already exist — selection happens at run-end"
        );
        assert!(
            stats.highlights.len() > 5,
            "highlight count should grow beyond old cap of 5. Got {}",
            stats.highlights.len()
        );

        // The flag should still be set
        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            tracker.first_evolution_recorded,
            "first_evolution_recorded flag should still be set"
        );

        // evolutions_performed should still increment
        assert_eq!(
            stats.evolutions_performed, 1,
            "evolutions_performed should increment"
        );

        // HighlightTriggered should be emitted
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            msg.is_some(),
            "should emit HighlightTriggered for FirstEvolution"
        );
    }
}
