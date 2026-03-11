//! Audio plugin registration.

use bevy::prelude::*;

/// Plugin for the audio domain.
///
/// Owns sound playback, music, and adaptive audio intensity.
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, _app: &mut App) {
        // Phase 0: stub — audio systems added in later phases.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(AudioPlugin)
            .update();
    }
}
