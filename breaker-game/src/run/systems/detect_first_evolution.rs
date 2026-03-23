//! System to detect `FirstEvolution` highlight when the player evolves a chip for the first time.

use bevy::prelude::*;

use crate::{
    chips::EvolutionRegistry,
    run::{definition::HighlightConfig, messages::HighlightTriggered, resources::*},
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
    evolution_registry: Option<Res<EvolutionRegistry>>,
    config: Res<HighlightConfig>,
    mut tracker: ResMut<HighlightTracker>,
    mut stats: ResMut<RunStats>,
    run_state: Res<RunState>,
    mut writer: MessageWriter<HighlightTriggered>,
) {
    let Some(registry) = evolution_registry else {
        // Drain reader to avoid stale messages
        for _msg in reader.read() {}
        return;
    };

    for msg in reader.read() {
        // Check if the selected chip name matches any recipe's result
        let is_evolution = registry
            .recipes()
            .iter()
            .any(|recipe| recipe.result_definition.name == msg.name);

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

            // Only record once in stats
            let already = stats
                .highlights
                .iter()
                .any(|h| h.kind == HighlightKind::FirstEvolution);
            if !already && stats.highlights.len() < config.highlight_cap as usize {
                stats.highlights.push(RunHighlight {
                    kind: HighlightKind::FirstEvolution,
                    node_index: run_state.node_index,
                    value: 1.0,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::{
        AmpEffect, ChipDefinition, ChipEffect, EvolutionIngredient, EvolutionRecipe, Rarity,
    };
    use crate::run::resources::{HighlightKind, RunHighlight};

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

    /// Creates a test `EvolutionRegistry` with one recipe producing
    /// `"Piercing Barrage"`.
    fn test_evolution_registry() -> EvolutionRegistry {
        let mut registry = EvolutionRegistry::default();
        registry.insert(EvolutionRecipe {
            ingredients: vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }],
            result_definition: ChipDefinition {
                name: "Piercing Barrage".to_owned(),
                description: "Test evolution chip".to_owned(),
                rarity: Rarity::Legendary,
                max_stacks: 1,
                effects: vec![ChipEffect::Amp(AmpEffect::Piercing(5))],
            },
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
            .insert_resource(test_evolution_registry())
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
        app.world_mut().resource_mut::<RunStats>().evolutions_performed = 1;

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

    // --- Behavior 24: Graceful without EvolutionRegistry ---

    #[test]
    fn graceful_without_evolution_registry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ChipSelected>()
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
            // NOTE: No EvolutionRegistry inserted — Option<Res<EvolutionRegistry>> is None
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
            "should not detect any highlight without EvolutionRegistry"
        );
        assert_eq!(
            stats.evolutions_performed, 0,
            "evolutions_performed should remain 0"
        );
    }

    // --- Behavior 25: Respects cap ---

    #[test]
    fn respects_highlight_cap() {
        let mut app = test_app();
        let config = HighlightConfig {
            highlight_cap: 1,
            ..Default::default()
        };
        app.insert_resource(config);

        // Pre-fill highlights to cap with a different kind
        app.world_mut()
            .resource_mut::<RunStats>()
            .highlights
            .push(RunHighlight {
                kind: HighlightKind::MassDestruction,
                node_index: 0,
                value: 10.0,
            });

        app.insert_resource(TestMessages(vec![ChipSelected {
            name: "Piercing Barrage".to_owned(),
        }]));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            1,
            "should not exceed highlight cap of 1"
        );
        let first_evo = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            first_evo.is_none(),
            "FirstEvolution should NOT be added when cap is reached"
        );

        // But the flag should still be set
        let tracker = app.world().resource::<HighlightTracker>();
        assert!(
            tracker.first_evolution_recorded,
            "first_evolution_recorded flag should still be set even when cap prevents highlight recording"
        );

        // evolutions_performed should still increment
        assert_eq!(
            stats.evolutions_performed, 1,
            "evolutions_performed should still increment even when cap is reached"
        );

        // HighlightTriggered should STILL be emitted
        let captured = app.world().resource::<CapturedHighlightTriggered>();
        let msg = captured
            .0
            .iter()
            .find(|h| h.kind == HighlightKind::FirstEvolution);
        assert!(
            msg.is_some(),
            "should still emit HighlightTriggered even when cap is reached"
        );
    }
}
