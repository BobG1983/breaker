//! Game plugin group — wires together all domain plugins.

use bevy::{
    app::PluginGroupBuilder, camera::ScalingMode, core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom, prelude::*,
};

use crate::{
    audio::AudioPlugin, behaviors::BehaviorsPlugin, bolt::BoltPlugin, breaker::BreakerPlugin,
    cells::CellsPlugin, chips::ChipsPlugin, debug::DebugPlugin, fx::FxPlugin, input::InputPlugin,
    interpolate::InterpolatePlugin, physics::PhysicsPlugin, run::RunPlugin, screen::ScreenPlugin,
    shared::PlayfieldConfig, ui::UiPlugin, wall::WallPlugin,
};

/// Plugin group that assembles all game domain plugins.
///
/// This is the single place that knows about all plugins.
/// Added to the Bevy [`App`] in [`crate::app::build_app`].
///
/// When `headless` is `false` (the default), includes [`RenderSetupPlugin`]
/// which spawns the camera and inserts [`ClearColor`].
#[derive(Default)]
pub struct Game {
    /// When `true`, skips [`RenderSetupPlugin`] (no camera or clear color).
    headless: bool,
}

impl Game {
    /// Creates a headless [`Game`] that skips rendering setup.
    #[must_use]
    pub const fn headless() -> Self {
        Self { headless: true }
    }
}

impl PluginGroup for Game {
    fn build(self) -> PluginGroupBuilder {
        let mut builder = PluginGroupBuilder::start::<Self>()
            .add(InputPlugin)
            .add(ScreenPlugin)
            .add(InterpolatePlugin)
            .add(PhysicsPlugin)
            .add(WallPlugin)
            .add(BreakerPlugin)
            .add(BehaviorsPlugin)
            .add(BoltPlugin)
            .add(CellsPlugin)
            .add(ChipsPlugin)
            .add(FxPlugin)
            .add(RunPlugin)
            .add(AudioPlugin)
            .add(UiPlugin)
            .add(DebugPlugin);

        if !self.headless {
            builder = builder.add(RenderSetupPlugin);
        } else {
            // DebugPlugin depends on GizmoConfigStore (from GizmoPlugin in
            // DefaultPlugins). In headless mode GizmoPlugin may be disabled,
            // and debug overlays serve no purpose without a window anyway.
            builder = builder.disable::<DebugPlugin>();
        }

        builder
    }
}

/// Spawns the 2D camera and inserts [`ClearColor`].
///
/// Included by [`Game`] when not running headless.
struct RenderSetupPlugin;

impl Plugin for RenderSetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(PlayfieldConfig::default().background_color()));
        app.add_systems(Startup, spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1920.0,
                min_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::AcesFitted,
        Bloom::default(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(game: Game) -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            bevy::asset::AssetPlugin::default(),
            bevy::input::InputPlugin,
        ));
        app.add_plugins(game.build().disable::<DebugPlugin>());
        app
    }

    #[test]
    fn game_plugin_group_builds() {
        let mut app = test_app(Game::default());
        app.update();
    }

    #[test]
    fn headless_game_spawns_no_camera() {
        let mut app = test_app(Game::headless());
        app.update();

        let count = app
            .world_mut()
            .query::<&Camera2d>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "headless game should not spawn a camera");
    }
}
