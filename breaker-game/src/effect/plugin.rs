use bevy::prelude::*;

/// Plugin that registers all effect and trigger systems.
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        super::effects::register(app);
        super::triggers::register(app);
    }
}
