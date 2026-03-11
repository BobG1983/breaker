//! Run plugin registration.

use bevy::prelude::*;

use crate::run::messages::{NodeCleared, TimerExpired};
use crate::run::resources::RunState;

/// Plugin for the run domain.
///
/// Owns run state, node timer, and node sequencing.
pub struct RunPlugin;

impl Plugin for RunPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunState>();
        app.add_message::<NodeCleared>();
        app.add_message::<TimerExpired>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(RunPlugin)
            .update();
    }
}
