//! Upgrades plugin registration.

use bevy::prelude::*;

/// Plugin for the upgrades domain.
///
/// Owns upgrade application, stacking, and registry resources.
pub struct UpgradesPlugin;

impl Plugin for UpgradesPlugin {
    fn build(&self, _app: &mut App) {
        // Phase 0: stub — registries and systems added in later phases.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(UpgradesPlugin)
            .update();
    }
}
