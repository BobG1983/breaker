//! System to reset run state at the start of a new run.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::{
    run::resources::RunState,
    shared::{GameRng, SelectedArchetype},
};

/// Resets [`RunState`] to defaults and reseeds [`GameRng`] when leaving the
/// main menu (starting a run).
pub fn reset_run_state(
    mut run_state: ResMut<RunState>,
    mut rng: ResMut<GameRng>,
    archetype: Option<Res<SelectedArchetype>>,
) {
    *run_state = RunState::default();
    // Reseed with entropy — Phase 4 will add user-selectable seeds
    rng.0 = ChaCha8Rng::from_os_rng();
    let archetype_name = archetype.as_deref().map_or("none", |a| a.0.as_str());
    info!("run started archetype={}", archetype_name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::RunOutcome;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(RunState {
            node_index: 5,
            outcome: RunOutcome::Won,
            ..default()
        });
        app.init_resource::<GameRng>();
        app.add_systems(Update, reset_run_state);
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
}
