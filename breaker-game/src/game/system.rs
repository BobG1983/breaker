//! Game plugin group — wires together all domain plugins.

use bevy::{
    app::PluginGroupBuilder, camera::ScalingMode, core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom, prelude::*,
};
use rantzsoft_dmg::{DmgSystems, RantzDmgAppExt, RantzDmgPlugin};
use rantzsoft_spatial2d::plugin::RantzSpatial2dPlugin;

use crate::{
    audio::AudioPlugin,
    bolt::{BoltPlugin, components::Bolt},
    breaker::{BreakerPlugin, components::Breaker},
    cells::{CellsPlugin, behaviors::survival::salvo::components::Salvo, components::Cell},
    chips::ChipsPlugin,
    debug::DebugPlugin,
    effect_v3::{EffectV3Plugin, sets::EffectV3Systems},
    fx::FxPlugin,
    input::InputPlugin,
    mutators::MutatorsPlugin,
    shared::{GameDrawLayer, PlayfieldConfig},
    state::StatePlugin,
    walls::{WallPlugin, components::Wall},
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
            .add(EffectV3Plugin)
            .add(RantzDmgPlugin)
            .add(DmgRegistrationsPlugin)
            .add(DmgGameOrderingPlugin)
            .add(BoltPlugin)
            .add(CellsPlugin)
            .add(ChipsPlugin)
            .add(MutatorsPlugin)
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
                min_width:  1920.0,
                min_height: 1080.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Tonemapping::AcesFitted,
        Bloom::default(),
    ));
}

/// Wires per-`T` `rantzsoft_dmg` registrations for every game `Dmgable` type.
///
/// After W2 this includes `Cell` — the crate-owned `apply_damage::<Cell>`,
/// `apply_damage_boosts::<Cell>`, `apply_vulnerable::<Cell>`, and
/// `invulnerable_filter::<Cell>` systems replace the former in-tree
/// `apply_damage_to_cells` system entirely.
///
/// # Why centralized (vs each domain plugin registering itself)
///
/// Per-domain plugins could each call `register_dmgable::<SelfType>()` in
/// their own `Plugin::build`, matching the general "self-contained domain"
/// rule from `docs/architecture/plugins.md`. We deviate on purpose:
///
/// - **Ordering safety.** `register_dmgable::<T>` installs six systems into
///   `DmgSystems` sets configured by `RantzDmgPlugin::build`. A domain
///   plugin registering itself would race against plugin-build order —
///   `RantzDmgPlugin` MUST build before any `register_dmgable::<T>` runs.
///   Centralizing the calls after `add_plugins(RantzDmgPlugin)` in the
///   plugin group makes that invariant impossible to break.
/// - **Single audit point.** Every game `Dmgable` participant is listed in
///   one place; a reviewer can see the full set without walking five
///   domain plugins.
/// - **Test ergonomics.** `TestAppBuilder::with_effects_pipeline()` mirrors
///   this plugin exactly, so test fixtures match production wiring.
///
/// Domain plugins remain responsible for inserting per-domain systems,
/// components, messages, and RON-loaded assets — everything except the
/// crate-owned `Dmgable` registration.
struct DmgRegistrationsPlugin;

impl Plugin for DmgRegistrationsPlugin {
    fn build(&self, app: &mut App) {
        let _ = app
            .register_dmgable::<Bolt>()
            .register_dmgable::<Wall>()
            .register_dmgable::<Breaker>()
            .register_dmgable::<Salvo>()
            .register_dmgable::<Cell>();
    }
}

/// Game-side ordering edges that bridge `DmgSystems` and `EffectV3Systems`.
///
/// Effects that boost or modify damage must commit their changes before the
/// damage pipeline reads HP. This plugin ensures `EffectV3Systems::Tick` runs
/// before `DmgSystems::ApplyDamage` in `FixedUpdate`.
///
/// The `PostApplyDamage` ripple chain (`diffusion_emit_rings →
/// tether_emit_partner → echo_strike_emit_siblings`) is owned by
/// `MutatorsPlugin::wire_damage_chain` via a `.chain()` tuple — no sub-set
/// configuration is needed here.
struct DmgGameOrderingPlugin;

impl Plugin for DmgGameOrderingPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            FixedUpdate,
            EffectV3Systems::Tick.before(DmgSystems::ApplyDamage),
        );
    }
}
