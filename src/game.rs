//! Game plugin group — wires together all domain plugins.

use bevy::{app::PluginGroupBuilder, prelude::*};

use crate::{
    audio::AudioPlugin, bolt::BoltPlugin, breaker::BreakerPlugin, cells::CellsPlugin,
    chips::ChipsPlugin, debug::DebugPlugin, input::InputPlugin, physics::PhysicsPlugin,
    run::RunPlugin, screen::ScreenPlugin, ui::UiPlugin, wall::WallPlugin,
};

/// Plugin group that assembles all game domain plugins.
///
/// This is the single place that knows about all plugins.
/// Added to the Bevy [`App`] in [`crate::app::build_app`].
pub struct Game;

impl PluginGroup for Game {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(InputPlugin)
            .add(ScreenPlugin)
            .add(PhysicsPlugin)
            .add(WallPlugin)
            .add(BreakerPlugin)
            .add(BoltPlugin)
            .add(CellsPlugin)
            .add(ChipsPlugin)
            .add(RunPlugin)
            .add(AudioPlugin)
            .add(UiPlugin)
            .add(DebugPlugin)
    }
}

#[cfg(all(test, not(target_os = "macos")))]
mod tests {
    use super::*;

    #[test]
    fn game_plugin_group_builds() {
        let mut app = App::new();
        app.add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: None,
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    file_path: "assets".into(),
                    ..default()
                }),
        );
        app.add_plugins(Game.build().disable::<DebugPlugin>());
        app.update();
    }
}
