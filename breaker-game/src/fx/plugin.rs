//! Fx plugin registration.

use bevy::prelude::*;

use crate::{
    fx::systems::{animate_fade_out, animate_punch_scale, tick_effect_flash},
    prelude::*,
};

/// Plugin for the fx domain.
///
/// Owns cross-cutting visual effects: fade-out animations, transition overlays,
/// and (future) screen shake and particles.
pub(crate) struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_fade_out, animate_punch_scale, tick_effect_flash)
                .run_if(in_state(NodeState::Playing)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        let mut app = TestAppBuilder::new().with_state_hierarchy().build();
        app.add_plugins(FxPlugin);
        app.update();
    }
}
