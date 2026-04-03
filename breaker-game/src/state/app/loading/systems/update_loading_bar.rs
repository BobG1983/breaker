//! Updates the loading bar UI based on global progress.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::state::app::loading::components::{LoadingBarFill, LoadingProgressText};

/// Updates the loading bar width and text based on global progress.
pub(crate) fn update_loading_bar(
    progress: Res<ProgressTracker<crate::shared::GameState>>,
    mut bar_query: Query<&mut Node, With<LoadingBarFill>>,
    mut text_query: Query<&mut Text, With<LoadingProgressText>>,
) {
    let global = progress.get_global_progress();
    let ratio = if global.total > 0 {
        f32::from(u16::try_from(global.done).unwrap_or(u16::MAX))
            / f32::from(u16::try_from(global.total).unwrap_or(u16::MAX))
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
