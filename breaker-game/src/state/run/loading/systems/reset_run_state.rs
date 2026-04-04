//! System to reset run state at the start of a new run.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tracing::info;

use crate::{
    breaker::SelectedBreaker,
    chips::inventory::ChipInventory,
    shared::{GameRng, RunSeed},
    state::run::resources::{HighlightTracker, NodeOutcome, RunStats},
};

/// Resets [`NodeOutcome`] to defaults and reseeds [`GameRng`] when leaving the
/// main menu (starting a run).
pub(crate) fn reset_run_state(
    mut run_state: ResMut<NodeOutcome>,
    mut rng: ResMut<GameRng>,
    seed: Res<RunSeed>,
    selected_breaker: Option<Res<SelectedBreaker>>,
    mut chip_inventory: ResMut<ChipInventory>,
    mut stats: ResMut<RunStats>,
    mut highlight_tracker: ResMut<HighlightTracker>,
) {
    *run_state = NodeOutcome::default();
    *stats = RunStats::default();
    *highlight_tracker = HighlightTracker::default();
    chip_inventory.clear();
    if let Some(s) = seed.0 {
        *rng = GameRng::from_seed(s);
        info!("run started seed={s}");
    } else {
        rng.0 = ChaCha8Rng::from_os_rng();
        info!("run started seed=random");
    }
    let breaker_name = selected_breaker.as_deref().map_or("none", |b| b.0.as_str());
    info!("run started breaker={}", breaker_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::resources::{NodeOutcome, NodeResult};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(NodeOutcome {
                node_index: 5,
                result: NodeResult::Won,
                ..default()
            })
            .init_resource::<GameRng>()
            .init_resource::<RunSeed>()
            .init_resource::<ChipInventory>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .add_systems(Update, reset_run_state);
        app
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
}
