//! UI plugin registration.

use bevy::prelude::*;

use crate::ui::messages::UpgradeSelected;

/// Plugin for the UI domain.
///
/// Owns HUD rendering, menu screens, and upgrade selection.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<UpgradeSelected>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(UiPlugin)
            .update();
    }
}
