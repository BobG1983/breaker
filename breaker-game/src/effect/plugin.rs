use bevy::prelude::*;

use super::sets::EffectSystems;
use crate::shared::PlayingState;

/// Plugin that registers all effect and trigger systems.
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            EffectSystems::Recalculate.run_if(in_state(PlayingState::Active)),
        );
        super::effects::register(app);
        super::triggers::register(app);
    }
}
