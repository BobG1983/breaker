//! System to reset run state at the start of a new run.

use bevy::{ecs::system::SystemParam, prelude::*};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    chips::inventory::ChipInventory,
    hazard::resources::ActiveHazards,
    prelude::*,
    protocol::resources::ActiveProtocols,
    shared::RunSeed,
    state::run::resources::{HighlightTracker, NodeOutcome},
};

/// Per-run inventories cleared on each new run. Grouped by **lifecycle**
/// (cleared on every new run), not by domain — callers outside
/// [`reset_run_state`] should read the individual resources directly.
/// Bundled here so [`reset_run_state`] stays within the system-parameter
/// limit.
#[derive(SystemParam)]
pub(crate) struct RunInventories<'w> {
    chips:     ResMut<'w, ChipInventory>,
    protocols: ResMut<'w, ActiveProtocols>,
    hazards:   ResMut<'w, ActiveHazards>,
}

impl RunInventories<'_> {
    fn clear_all(&mut self) {
        self.chips.clear();
        self.protocols.clear();
        self.hazards.clear();
    }
}

/// Resets [`NodeOutcome`] to defaults and reseeds [`GameRng`] when leaving the
/// main menu (starting a run).
pub(crate) fn reset_run_state(
    mut run_state: ResMut<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    seed: Res<RunSeed>,
    mut stats: ResMut<RunStats>,
    mut highlight_tracker: ResMut<HighlightTracker>,
    mut inventories: RunInventories,
) {
    *run_state = NodeOutcome::default();
    *stats = RunStats::default();
    *highlight_tracker = HighlightTracker::default();
    inventories.clear_all();
    if let Some(s) = seed.0 {
        *rng = GameRng::from_seed(s);
    } else {
        rng.0 = ChaCha8Rng::from_os_rng();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::resources::{NodeOutcome, NodeResult};

    fn test_app() -> App {
        TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index: 5,
                result: NodeResult::Won,
                ..default()
            })
            .with_resource::<GameRng>()
            .with_resource::<RunSeed>()
            .with_resource::<ChipInventory>()
            .with_resource::<RunStats>()
            .with_resource::<HighlightTracker>()
            .with_resource::<crate::protocol::resources::ActiveProtocols>()
            .with_resource::<crate::hazard::resources::ActiveHazards>()
            .with_system(Update, reset_run_state)
            .build()
    }

    #[test]
    fn resets_to_defaults() {
        let mut app = test_app();
        app.update();

        let state = app.world().resource::<NodeOutcome>();
        assert_eq!(state.node_index, 0);
        assert_eq!(state.result, NodeResult::InProgress);
    }

    #[test]
    fn reseeds_with_specific_seed_when_set() {
        use rand::Rng;
        let mut app = test_app();
        app.world_mut().insert_resource(RunSeed(Some(42)));
        app.update();

        let val1: f32 = app.world_mut().resource_mut::<GameRng>().0.random();

        // Same seed must produce same sequence
        let mut rng2 = GameRng::from_seed(42);
        let val2: f32 = rng2.0.random();
        assert!(
            (val1 - val2).abs() < f32::EPSILON,
            "expected deterministic output with seed 42"
        );
    }

    #[test]
    fn reseeds_with_entropy_when_none() {
        use rand::Rng;
        let mut app = test_app();
        // RunSeed default is None
        app.update();

        let val1: f32 = app.world_mut().resource_mut::<GameRng>().0.random();

        // Run again — should get a different RNG state (extremely unlikely to match)
        app.world_mut().insert_resource(NodeOutcome {
            node_index: 5,
            result: NodeResult::Won,
            ..default()
        });
        app.update();

        let val2: f32 = app.world_mut().resource_mut::<GameRng>().0.random();
        // Not asserting inequality — OS entropy could theoretically match,
        // but we verify the code path runs without panic
        let _ = (val1, val2);
    }

    // ── Behavior 28: reset_run_state clears ActiveProtocols and ActiveHazards ─

    /// Seed `ActiveProtocols` with a `Greed` definition and `ActiveHazards`
    /// with `add_stack(Decay)` ×2, then run the system once. Both resources
    /// should be emptied — replacing the per-run-reset coverage that a
    /// `plugin_builds` smoke test would not give us.
    #[test]
    fn clears_active_protocols_and_active_hazards() {
        use crate::{
            hazard::{definition::HazardKind, resources::ActiveHazards},
            protocol::{
                definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
                resources::ActiveProtocols,
            },
        };

        let mut app = test_app();

        // Seed ActiveProtocols with one Greed definition.
        {
            let mut protocols = app.world_mut().resource_mut::<ActiveProtocols>();
            protocols.insert(ProtocolDefinition {
                name:        "Greed".into(),
                description: String::new(),
                unlock_tier: 0,
                tuning:      ProtocolTuning::Greed {
                    rarity_boost_per_skip: 0.05,
                },
            });
        }
        // Seed ActiveHazards with two Decay stacks.
        {
            let mut hazards = app.world_mut().resource_mut::<ActiveHazards>();
            hazards.add_stack(HazardKind::Decay);
            hazards.add_stack(HazardKind::Decay);
        }

        app.update();

        assert!(
            app.world().resource::<ActiveProtocols>().is_empty(),
            "reset_run_state must clear ActiveProtocols"
        );
        assert!(
            app.world().resource::<ActiveHazards>().is_empty(),
            "reset_run_state must clear ActiveHazards"
        );
        assert!(
            !app.world()
                .resource::<ActiveProtocols>()
                .contains(ProtocolKind::Greed),
            "Greed must not be active after reset"
        );
        assert_eq!(
            app.world()
                .resource::<ActiveHazards>()
                .stacks(HazardKind::Decay),
            0,
            "Decay stacks must be 0 after reset"
        );
    }
}
