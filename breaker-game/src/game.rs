//! Game plugin group — wires together all domain plugins.

use bevy::{
    app::PluginGroupBuilder, camera::ScalingMode, core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom, prelude::*,
};
use rantzsoft_spatial2d::plugin::RantzSpatial2dPlugin;

use crate::{
    audio::AudioPlugin,
    bolt::BoltPlugin,
    breaker::BreakerPlugin,
    cells::CellsPlugin,
    chips::ChipsPlugin,
    debug::DebugPlugin,
    effect::EffectPlugin,
    fx::FxPlugin,
    input::InputPlugin,
    shared::{GameDrawLayer, PlayfieldConfig},
    state::StatePlugin,
    walls::WallPlugin,
};

/// Plugin group that assembles all game domain plugins.
///
/// This is the single place that knows about all plugins.
/// Added to the Bevy [`App`] in [`crate::app::build_app`].
///
/// Use [`Game::default()`] for normal rendering (includes [`RenderSetupPlugin`]
/// which spawns the camera and inserts [`ClearColor`]). Use [`Game::headless()`]
/// for headless mode (includes [`HeadlessAssetsPlugin`] which registers asset
/// types that render-pipeline plugins would normally provide).
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
            .add(StatePlugin)
            .add(RantzSpatial2dPlugin::<GameDrawLayer>::default())
            .add(rantzsoft_physics2d::plugin::RantzPhysics2dPlugin)
            .add(WallPlugin)
            .add(BreakerPlugin)
            .add(EffectPlugin)
            .add(BoltPlugin)
            .add(CellsPlugin)
            .add(ChipsPlugin)
            .add(FxPlugin)
            .add(AudioPlugin)
            .add(DebugPlugin);

        if self.headless {
            // DebugPlugin depends on GizmoConfigStore (from GizmoPlugin in
            // DefaultPlugins). In headless mode GizmoPlugin may be disabled,
            // and debug overlays serve no purpose without a window anyway.
            builder = builder.disable::<DebugPlugin>().add(HeadlessAssetsPlugin);
        } else {
            builder = builder.add(RenderSetupPlugin);
        }

        builder
    }
}

/// Registers plugins and asset types normally provided by render-pipeline
/// plugins (`MeshPlugin`, `ColorMaterialPlugin`, `TextPlugin`, etc.). In
/// headless mode those plugins are absent, but gameplay spawn systems still
/// need the asset storage.
///
/// Included by [`Game`] only in headless mode.
struct HeadlessAssetsPlugin;

impl Plugin for HeadlessAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::mesh::MeshPlugin)
            .init_asset::<ColorMaterial>()
            .add_plugins(bevy::text::TextPlugin);
    }
}

/// Spawns the 2D camera and inserts [`ClearColor`].
///
/// Included by [`Game`] when not running headless.
struct RenderSetupPlugin;

impl Plugin for RenderSetupPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(PlayfieldConfig::default().background_color()))
            .add_systems(Startup, spawn_camera);
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
        ))
        .add_plugins(game.build().disable::<DebugPlugin>());
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

    #[test]
    fn headless_game_registers_headless_assets() {
        let mut app = test_app(Game::headless());
        app.update();

        assert!(
            app.world().get_resource::<Assets<Mesh>>().is_some(),
            "headless game must register Assets<Mesh> via MeshPlugin"
        );
        assert!(
            app.world()
                .get_resource::<Assets<ColorMaterial>>()
                .is_some(),
            "headless game must register Assets<ColorMaterial>"
        );
        assert!(
            app.world().get_resource::<Assets<Font>>().is_some(),
            "headless game must register Assets<Font> via TextPlugin"
        );
    }
}
