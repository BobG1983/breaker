use bevy::prelude::*;

use super::super::spawn_run_end_screen;
use crate::state::run::resources::{NodeOutcome, NodeResult, RunStats};

pub(super) fn test_app(result: NodeResult) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(NodeOutcome {
            node_index: 0,
            result,
            ..default()
        })
        .add_systems(Update, spawn_run_end_screen);
    app
}

pub(super) fn test_app_with_stats(result: NodeResult, stats: RunStats) -> App {
    let mut app = test_app(result);
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
