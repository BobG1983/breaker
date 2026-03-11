//! Game plugin group — wires together all domain plugins.

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;

use crate::audio::AudioPlugin;
use crate::bolt::BoltPlugin;
use crate::breaker::BreakerPlugin;
use crate::cells::CellsPlugin;
use crate::debug::DebugPlugin;
use crate::physics::PhysicsPlugin;
use crate::run::RunPlugin;
use crate::screen::ScreenPlugin;
use crate::ui::UiPlugin;
use crate::upgrades::UpgradesPlugin;

/// Plugin group that assembles all game domain plugins.
///
/// This is the single place that knows about all plugins.
/// Added to the Bevy [`App`] in [`crate::app::build_app`].
pub struct Game;

impl PluginGroup for Game {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(ScreenPlugin)
            .add(PhysicsPlugin)
            .add(BreakerPlugin)
            .add(BoltPlugin)
            .add(CellsPlugin)
            .add(UpgradesPlugin)
            .add(RunPlugin)
            .add(AudioPlugin)
            .add(UiPlugin)
            .add(DebugPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_plugin_group_builds() {
        App::new()
            .add_plugins(MinimalPlugins)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(Game.build().disable::<DebugPlugin>())
            .update();
    }
}
