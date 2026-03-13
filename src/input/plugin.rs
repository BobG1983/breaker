//! Input plugin registration.

use bevy::{input::InputSystems, prelude::*};

use crate::input::{
    resources::{DoubleTapState, InputActions, InputConfig},
    systems::{clear_input_actions, read_input_actions},
};

/// Plugin for the input domain.
///
/// Translates raw keyboard input into [`InputActions`] each frame.
/// Runs in `PreUpdate` after `InputSystems` so all gameplay systems
/// see consistent, FixedUpdate-safe actions.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputActions>()
            .init_resource::<InputConfig>()
            .init_resource::<DoubleTapState>()
            .add_systems(PreUpdate, read_input_actions.after(InputSystems))
            .add_systems(FixedPostUpdate, clear_input_actions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<bevy::input::keyboard::KeyboardInput>()
            .add_plugins(InputPlugin)
            .update();
    }
}
