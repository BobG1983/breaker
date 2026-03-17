//! Chips plugin registration.

use bevy::prelude::*;

/// Plugin for the chips domain.
///
/// Owns chip application, stacking, and registry resources.
pub struct ChipsPlugin;

impl Plugin for ChipsPlugin {
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
            .add_plugins(ChipsPlugin)
            .update();
    }
}
