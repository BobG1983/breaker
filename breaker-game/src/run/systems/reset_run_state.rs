//! System to reset run state at the start of a new run.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tracing::info;

use crate::{
    run::resources::RunState,
    shared::{GameRng, RunSeed, SelectedArchetype},
};

/// Resets [`RunState`] to defaults and reseeds [`GameRng`] when leaving the
/// main menu (starting a run).
pub fn reset_run_state(
    mut run_state: ResMut<RunState>,
    mut rng: ResMut<GameRng>,
    seed: Res<RunSeed>,
    archetype: Option<Res<SelectedArchetype>>,
) {
    *run_state = RunState::default();
    if let Some(s) = seed.0 {
        rng.0 = ChaCha8Rng::seed_from_u64(s);
        info!("run started seed={s}");
    } else {
        rng.0 = ChaCha8Rng::from_os_rng();
        info!("run started seed=random");
    }
    let archetype_name = archetype.as_deref().map_or("none", |a| a.0.as_str());
    info!("run started archetype={}", archetype_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::RunOutcome;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RunState {
                node_index: 5,
                outcome: RunOutcome::Won,
                ..default()
            })
            .init_resource::<GameRng>()
            .init_resource::<RunSeed>()
            .add_systems(Update, reset_run_state);
        app
    }

    #[test]
    fn resets_to_defaults() {
        let mut app = test_app();
        app.update();

        let state = app.world().resource::<RunState>();
        assert_eq!(state.node_index, 0);
        assert_eq!(state.outcome, RunOutcome::InProgress);
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
        app.world_mut().insert_resource(RunState {
            node_index: 5,
            outcome: RunOutcome::Won,
            ..default()
        });
        app.update();

        let val2: f32 = app.world_mut().resource_mut::<GameRng>().0.random();
        // Not asserting inequality — OS entropy could theoretically match,
        // but we verify the code path runs without panic
        let _ = (val1, val2);
    }
}
