//! Updates the loading bar UI based on global progress.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::screen::loading::components::{LoadingBarFill, LoadingProgressText};

/// Updates the loading bar width and text based on global progress.
pub fn update_loading_bar(
    progress: Res<ProgressTracker<crate::shared::GameState>>,
    mut bar_query: Query<&mut Node, With<LoadingBarFill>>,
    mut text_query: Query<&mut Text, With<LoadingProgressText>>,
) {
    let global = progress.get_global_progress();
    #[allow(clippy::cast_precision_loss)]
    let ratio = if global.total > 0 {
        global.done as f32 / global.total as f32
    } else {
        0.0
    };

    for mut node in &mut bar_query {
        node.width = Val::Percent(ratio * 100.0);
    }

    for mut text in &mut text_query {
        **text = format!("Loading... {}/{}", global.done, global.total);
    }
}
