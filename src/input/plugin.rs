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
        app.init_resource::<InputActions>();
        app.init_resource::<InputConfig>();
        app.init_resource::<DoubleTapState>();
        app.add_systems(PreUpdate, read_input_actions.after(InputSystems));
        app.add_systems(FixedPostUpdate, clear_input_actions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_message::<bevy::input::keyboard::KeyboardInput>();
        app.add_plugins(InputPlugin);
        app.update();
    }
}
