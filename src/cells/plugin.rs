//! Cells plugin registration.

use bevy::prelude::*;

use crate::cells::messages::CellDestroyed;

/// Plugin for the cells domain.
///
/// Owns cell components, grid layout, and destruction logic.
pub struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellDestroyed>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(CellsPlugin)
            .update();
    }
}
