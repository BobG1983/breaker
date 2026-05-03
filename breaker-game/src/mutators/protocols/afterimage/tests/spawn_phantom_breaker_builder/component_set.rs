//! Behaviors 1-3 — full extra-phantom component set, live BaseWidth/BaseHeight
//! inheritance (including runtime mutation), and phantom material color
//! distinct from the real breaker.

use bevy::{ecs::system::SystemState, prelude::*};

use super::{
    super::helpers::{build_afterimage_app, seed_active_protocols_with_afterimage},
    BuilderSpawnState, drive_rising_edge, override_config_to_2_5, spawn_real_breaker_via_builder,
    test_breaker_definition,
};
use crate::{
    breaker::components::{BumpState, DashState, ExtraBreaker, PhantomBreaker, PrimaryBreaker},
    prelude::*,
    shared::{BaseHeight, BaseWidth, GameDrawLayer, Lifespan, PhantomFlicker},
};

// ── Behavior 1 — Full component set (migration sanity) ──────────────────────

/// Pins that the afterimage CALLER uses the builder — the raw spawn does not
/// insert `Breaker`, `Lifespan`, `PhantomFlicker`, `Mesh2d`, mesh-material,
/// draw layer, bump stats, collision layers, or `ExtraBreaker`.
///
/// RED failure: the `expect("phantom spawned via builder must carry Breaker
/// AND PhantomBreaker")` panics because the raw-spawn phantom lacks `Breaker`.
#[test]
fn phantom_spawn_via_builder_inserts_full_extra_phantom_component_set() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);

    // First tick: records Idle in Local; no phantom.
    tick(&mut app);
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<PhantomBreaker>>()
            .iter(app.world())
            .count(),
        0,
        "no phantom before rising edge"
    );

    // Rising edge.
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(&mut app);

    // Identify phantom — requires BOTH Breaker AND PhantomBreaker (builder
    // contract). Pre-migration this query finds nothing because the bare
    // spawn lacks `Breaker`, so the `expect` is the RED-failure point.
    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<Breaker>, With<PhantomBreaker>)>()
        .iter(app.world())
        .next()
        .expect("phantom spawned via builder must carry Breaker AND PhantomBreaker");

    let world = app.world();

    // ── Positive assertions ─────────────────────────────────────────────
    assert!(world.get::<Breaker>(phantom).is_some(), "must have Breaker");
    assert!(
        world.get::<PhantomBreaker>(phantom).is_some(),
        "must have PhantomBreaker"
    );
    assert!(
        world.get::<ExtraBreaker>(phantom).is_some(),
        "must have ExtraBreaker (.extra() role)"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(phantom).is_some(),
        "must have CleanupOnExit::<NodeState> (Extra cleanup)"
    );
    let lifespan = world.get::<Lifespan>(phantom).expect("must have Lifespan");
    assert!(
        (lifespan.remaining - 2.5).abs() < f32::EPSILON,
        "Lifespan.remaining must be 2.5 (phantom_duration), got {}",
        lifespan.remaining
    );
    let flicker = world
        .get::<PhantomFlicker>(phantom)
        .expect("must have PhantomFlicker");
    assert!(
        (flicker.frequency - 4.0).abs() < f32::EPSILON,
        "PhantomFlicker.frequency must be 4.0, got {}",
        flicker.frequency
    );
    assert!(
        (flicker.min_alpha - 0.3).abs() < f32::EPSILON,
        "PhantomFlicker.min_alpha must be 0.3, got {}",
        flicker.min_alpha
    );
    assert!(world.get::<Mesh2d>(phantom).is_some(), "must have Mesh2d");
    assert!(
        world
            .get::<MeshMaterial2d<ColorMaterial>>(phantom)
            .is_some(),
        "must have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        matches!(
            world.get::<GameDrawLayer>(phantom),
            Some(GameDrawLayer::Breaker)
        ),
        "must have GameDrawLayer::Breaker"
    );
    assert!(
        world.get::<BaseWidth>(phantom).is_some(),
        "must have BaseWidth"
    );
    assert!(
        world.get::<BaseHeight>(phantom).is_some(),
        "must have BaseHeight"
    );
    assert!(
        world.get::<BumpState>(phantom).is_some(),
        "must have BumpState"
    );
    let layers = world
        .get::<CollisionLayers>(phantom)
        .expect("must have CollisionLayers");
    assert_eq!(
        layers.membership, BREAKER_LAYER,
        "membership must be BREAKER_LAYER"
    );
    assert_eq!(layers.mask, BOLT_LAYER, "mask must be BOLT_LAYER");

    // ── Negative assertions (edge case) ─────────────────────────────────
    // Role is `.extra()`, NOT `.primary()` — no PrimaryBreaker or RunState cleanup.
    assert!(
        world.get::<PrimaryBreaker>(phantom).is_none(),
        "phantom must NOT have PrimaryBreaker (it uses .extra())"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(phantom).is_none(),
        "phantom must NOT have CleanupOnExit::<RunState> (only .primary() gets this)"
    );
}

// ── Behavior 2 — Phantom carries the real breaker's LIVE BaseWidth/BaseHeight ─

/// Pins that the migration reads the real breaker's LIVE `BaseWidth` /
/// `BaseHeight` components, not the `DEFAULT_PHANTOM_BASE_WIDTH` (100.0) /
/// `DEFAULT_PHANTOM_BASE_HEIGHT` (20.0) fallback constants.
///
/// Uses `150.0 / 30.0` which differ from both the definition default
/// (`120.0 / 20.0`) AND the constants (`100.0 / 20.0`).
///
/// RED failure: the `BumpState` sub-assertion fails because the raw spawn
/// never inserts bump stats (only `build_core` does).
#[test]
fn phantom_inherits_real_breaker_live_width_and_height() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    // Spawn real breaker with overridden dimensions 150.0 / 30.0.
    let real = {
        let world = app.world_mut();
        let mut state: BuilderSpawnState = SystemState::new(world);
        let (mut commands, mut meshes, mut materials) = state.get_mut(world);
        let entity = Breaker::builder()
            .definition(&test_breaker_definition())
            .with_width(150.0)
            .with_height(30.0)
            .rendered(&mut meshes, &mut materials)
            .primary()
            .spawn(&mut commands);
        state.apply(world);
        entity
    };

    drive_rising_edge(&mut app, real);

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<Breaker>, With<PhantomBreaker>)>()
        .iter(app.world())
        .next()
        .expect("phantom must exist after rising edge");

    let world = app.world();
    let width = world
        .get::<BaseWidth>(phantom)
        .expect("must have BaseWidth")
        .0;
    let height = world
        .get::<BaseHeight>(phantom)
        .expect("must have BaseHeight")
        .0;

    assert!(
        (width - 150.0).abs() < f32::EPSILON,
        "phantom BaseWidth must be 150.0 (live value), got {width}"
    );
    assert!(
        (height - 30.0).abs() < f32::EPSILON,
        "phantom BaseHeight must be 30.0 (live value), got {height}"
    );
    // Negative: must NOT use the constant fallback values.
    assert!(
        (width - 100.0).abs() > 1.0,
        "phantom BaseWidth must NOT be the fallback constant 100.0"
    );
    assert!(
        (height - 20.0).abs() > 1.0,
        "phantom BaseHeight must NOT be the fallback constant 20.0"
    );
    // Builder-only marker sub-assertion: forces RED failure against the raw spawn.
    // `build_core` inserts BumpState; the raw spawn never does.
    assert!(
        world.get::<BumpState>(phantom).is_some(),
        "migrated phantom must carry BumpState — only build_core inserts this"
    );
    assert!(
        world.get::<CollisionLayers>(phantom).is_some(),
        "migrated phantom must carry CollisionLayers — only build_core inserts this"
    );
}

/// Edge case: pins that the migration reads `BaseWidth` AT SPAWN TIME, not
/// from the definition at system startup. Mutating `BaseWidth` between the
/// first tick and the rising-edge tick produces a phantom with the mutated value.
///
/// RED failure: same `BumpState` sub-assertion as the main test.
#[test]
fn phantom_inherits_real_breaker_live_width_after_runtime_mutation() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let real = {
        let world = app.world_mut();
        let mut state: BuilderSpawnState = SystemState::new(world);
        let (mut commands, mut meshes, mut materials) = state.get_mut(world);
        let entity = Breaker::builder()
            .definition(&test_breaker_definition())
            .with_width(150.0)
            .with_height(30.0)
            .rendered(&mut meshes, &mut materials)
            .primary()
            .spawn(&mut commands);
        state.apply(world);
        entity
    };

    // First tick: records Idle.
    tick(&mut app);

    // Mutate BaseWidth to 200.0 BEFORE the rising-edge tick.
    *app.world_mut().get_mut::<BaseWidth>(real).unwrap() = BaseWidth(200.0);

    // Rising edge — phantom spawned with the mutated 200.0 value.
    *app.world_mut().get_mut::<DashState>(real).unwrap() = DashState::Dashing;
    tick(&mut app);

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<Breaker>, With<PhantomBreaker>)>()
        .iter(app.world())
        .next()
        .expect("phantom must exist after rising edge");

    let world = app.world();
    let width = world
        .get::<BaseWidth>(phantom)
        .expect("must have BaseWidth")
        .0;

    assert!(
        (width - 200.0).abs() < f32::EPSILON,
        "phantom BaseWidth must be the runtime-mutated 200.0, got {width}"
    );
    // Builder-only marker: forces RED failure against the raw spawn.
    assert!(
        world.get::<BumpState>(phantom).is_some(),
        "migrated phantom must carry BumpState — only build_core inserts this"
    );
}

// ── Behavior 3 — Phantom material color differs from real breaker material ──

/// Pins that the builder's `.phantom()` terminal mixes `phantom_color_rgb`
/// into the spawned material, producing a color distinct from the real
/// breaker's material.
///
/// RED failure: `world.get::<MeshMaterial2d<ColorMaterial>>(phantom).unwrap()`
/// panics — the raw spawn never inserts mesh-material components.
#[test]
fn phantom_material_color_differs_from_real_breaker_material_color() {
    let mut app = build_afterimage_app();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    override_config_to_2_5(&mut app);
    seed_active_protocols_with_afterimage(&mut app);

    let real = spawn_real_breaker_via_builder(&mut app);
    drive_rising_edge(&mut app, real);

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<Breaker>, With<PhantomBreaker>)>()
        .iter(app.world())
        .next()
        .expect("phantom must exist after rising edge");

    // Read both materials — the raw-spawn phantom lacks MeshMaterial2d, so
    // this unwrap is the RED-failure point.
    let phantom_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(phantom)
        .expect("phantom must have MeshMaterial2d<ColorMaterial>")
        .0
        .clone();
    let real_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(real)
        .expect("real breaker must have MeshMaterial2d<ColorMaterial>")
        .0
        .clone();

    let phantom_srgba = app
        .world()
        .resource::<Assets<ColorMaterial>>()
        .get(&phantom_handle)
        .expect("phantom material must be in asset registry")
        .color
        .to_srgba();
    let real_srgba = app
        .world()
        .resource::<Assets<ColorMaterial>>()
        .get(&real_handle)
        .expect("real material must be in asset registry")
        .color
        .to_srgba();

    // The phantom's color must differ from the real breaker's color.
    let differs_from_real = (real_srgba.red - phantom_srgba.red).abs() > 1e-3
        || (real_srgba.green - phantom_srgba.green).abs() > 1e-3
        || (real_srgba.blue - phantom_srgba.blue).abs() > 1e-3;
    assert!(
        differs_from_real,
        "phantom color {:?} must differ from real breaker color {:?}",
        (phantom_srgba.red, phantom_srgba.green, phantom_srgba.blue),
        (real_srgba.red, real_srgba.green, real_srgba.blue),
    );

    // Edge case: the tint is MIXED, not a replacement.
    // A replacement would make the phantom's color equal to [0.4, 0.8, 1.0].
    let is_raw_tint = (phantom_srgba.red - 0.4).abs() <= 1e-3
        && (phantom_srgba.green - 0.8).abs() <= 1e-3
        && (phantom_srgba.blue - 1.0).abs() <= 1e-3;
    assert!(
        !is_raw_tint,
        "phantom color {:?} must NOT be the raw phantom_color_rgb [0.4, 0.8, 1.0] — \
         it must be a blend (mix), not a replacement",
        (phantom_srgba.red, phantom_srgba.green, phantom_srgba.blue),
    );
}
