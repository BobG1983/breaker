//! Bolt plugin registration.

use bevy::prelude::*;

/// Plugin for the bolt domain.
///
/// Owns bolt components, velocity, and speed management.
pub struct BoltPlugin;

impl Plugin for BoltPlugin {
    fn build(&self, _app: &mut App) {
        // Phase 0: stub — systems registered in later phases.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(BoltPlugin)
            .update();
    }
}
