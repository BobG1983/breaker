use bevy::prelude::*;

use super::super::spawn_run_end_screen;
use crate::state::run::resources::{RunOutcome, RunState, RunStats};

pub(super) fn test_app(outcome: RunOutcome) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(RunState {
            node_index: 0,
            outcome,
            ..default()
        })
        .add_systems(Update, spawn_run_end_screen);
    app
}

pub(super) fn test_app_with_stats(outcome: RunOutcome, stats: RunStats) -> App {
    let mut app = test_app(outcome);
    app.insert_resource(stats);
    app
}

/// Collects all `Text` component values from the world.
pub(super) fn collect_texts(app: &mut App) -> Vec<String> {
    app.world_mut()
        .query::<&Text>()
        .iter(app.world())
        .map(|t| t.0.clone())
        .collect()
}
