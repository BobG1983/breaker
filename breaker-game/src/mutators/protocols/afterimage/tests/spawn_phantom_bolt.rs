//! Group F — `afterimage_spawn_phantom_bolt` (Behaviors F1–F10).
//!
//! Pins the message-consumer spawn system:
//! - On `BumpPerformed { grade: Perfect, bolt: Some(b), breaker: phantom }`
//!   where `phantom` carries `PhantomBreaker` → spawn a NEW entity with
//!   `(Bolt, ExtraBolt, PhantomBolt, PhantomLifetime(config.phantom_bolt_duration),
//!   PhantomOwner(b), Position2D(real.pos), Velocity2D(real.vel),
//!   BoltBaseDamage(real.base), CollisionLayers::new(BOLT_LAYER,
//!   BOLT_LAYER | WALL_LAYER | BREAKER_LAYER | CELL_LAYER),
//!   CleanupOnExit::<NodeState>)`.
//! - The real bolt is NEVER mutated.
//! - Non-Perfect grades do NOT spawn.
//! - `breaker` NOT carrying `PhantomBreaker` → no spawn.
//! - `bolt: None` → no-op.
//! - Despawned bolt → no-op (no panic).
//! - One-phantom-per-owner uniqueness (no re-spawn, no duration reset).
//! - Phantoms may outlive their real bolt (stale `PhantomOwner` tolerated).
//! - Harness-safe early return with reader drain when config absent.
//! - `run_if` gate on `ActiveProtocols`.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::{
    build_afterimage_app, build_afterimage_app_no_config, phantom_bolt_count,
    phantom_bolts_owned_by, seed_active_protocols_with_afterimage, spawn_phantom_bolt_entity,
    spawn_phantom_breaker_at, spawn_real_bolt, tick_n, write_bump_performed,
};
use crate::{
    bolt::components::{BoltBaseDamage, ExtraBolt},
    breaker::messages::BumpGrade,
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Spawns a `PhantomBreaker` at origin and a real bolt at (0.0, 0.0) with
/// velocity (0.0, 400.0) and `BoltBaseDamage(10.0)`. Returns `(phantom,
/// real_bolt)`.
fn canonical_setup(app: &mut App) -> (Entity, Entity) {
    let phantom = spawn_phantom_breaker_at(app, Vec2::ZERO, 1.5);
    let real = spawn_real_bolt(app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);
    (phantom, real)
}

// ── F1 — Perfect + phantom breaker → spawn with exact bundle ──────────────

#[test]
fn perfect_bump_on_phantom_breaker_spawns_new_phantom_bolt_entity() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "precondition: no phantom bolts before tick"
    );

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "expected exactly one phantom-bolt entity owned by real_bolt, got {}",
        owned.len()
    );
    let phantom_bolt = owned[0];
    assert_ne!(phantom_bolt, real_bolt, "phantom must be a new entity");

    // Verify every component field.
    assert_eq!(
        app.world().get::<PhantomOwner>(phantom_bolt).unwrap().0,
        real_bolt,
        "PhantomOwner must reference the real bolt"
    );
    let lifetime = app.world().get::<PhantomLifetime>(phantom_bolt).unwrap().0;
    // I14 pins spawn-before-tick-lifetime ordering — lifetime may have
    // already decremented once on the same tick.
    assert!(
        lifetime > 3.0 - 1.0 / 64.0 - 1e-4 && lifetime <= 3.0 + 1e-4,
        "PhantomLifetime.0 expected in (3.0 - 1/64, 3.0], got {lifetime}"
    );
    let pos = app.world().get::<Position2D>(phantom_bolt).unwrap().0;
    assert!(
        (pos - Vec2::new(0.0, 0.0)).length() < 1e-3,
        "Position copied from real bolt, expected (0.0, 0.0), got {pos:?}"
    );
    let vel = app.world().get::<Velocity2D>(phantom_bolt).unwrap().0;
    assert!(
        (vel - Vec2::new(0.0, 400.0)).length() < 1e-3,
        "Velocity copied from real bolt, expected (0.0, 400.0), got {vel:?}"
    );
    let damage = app.world().get::<BoltBaseDamage>(phantom_bolt).unwrap().0;
    assert!(
        (damage - 10.0).abs() < 1e-3,
        "BoltBaseDamage copied from real bolt, expected 10.0, got {damage}"
    );
    assert!(
        app.world().get::<Bolt>(phantom_bolt).is_some(),
        "phantom bolt must have the Bolt marker"
    );
    assert!(
        app.world().get::<ExtraBolt>(phantom_bolt).is_some(),
        "phantom bolt must have the ExtraBolt marker"
    );
    assert!(
        app.world()
            .get::<CleanupOnExit<NodeState>>(phantom_bolt)
            .is_some(),
        "phantom bolt must carry CleanupOnExit::<NodeState>"
    );

    // Real bolt is UNCHANGED.
    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "real bolt must NOT gain PhantomBolt marker"
    );
    assert!(
        app.world().get::<PhantomLifetime>(real_bolt).is_none(),
        "real bolt must NOT gain PhantomLifetime"
    );
    assert!(
        app.world().get::<PhantomOwner>(real_bolt).is_none(),
        "real bolt must NOT gain PhantomOwner"
    );
    let real_pos = app.world().get::<Position2D>(real_bolt).unwrap().0;
    assert!(
        real_pos.x.abs() < 20.0 && real_pos.y.abs() < 20.0,
        "real bolt position broadly unchanged (loose tolerance for in-tick \
         physics), got {real_pos:?}"
    );
}

// ── F1 (edge case) — CollisionLayers include CELL_LAYER ───────────────────

#[test]
fn spawned_phantom_bolt_collision_layers_include_cell_layer() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
    let layers = app
        .world()
        .get::<CollisionLayers>(owned[0])
        .expect("phantom bolt must have CollisionLayers");
    assert_ne!(
        layers.membership & BOLT_LAYER,
        0,
        "phantom membership must include BOLT_LAYER"
    );
    assert_ne!(
        layers.mask & BOLT_LAYER,
        0,
        "phantom mask must include BOLT_LAYER"
    );
    assert_ne!(
        layers.mask & CELL_LAYER,
        0,
        "phantom mask must include CELL_LAYER (differs from SpawnPhantomConfig::fire)"
    );
    assert_ne!(
        layers.mask & WALL_LAYER,
        0,
        "phantom mask must include WALL_LAYER"
    );
    assert_ne!(
        layers.mask & BREAKER_LAYER,
        0,
        "phantom mask must include BREAKER_LAYER"
    );
}

// ── F2 — Early grade does NOT spawn ───────────────────────────────────────

#[test]
fn early_grade_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Early);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Early grade must NOT spawn a phantom bolt"
    );
}

// ── F3 — Late grade does NOT spawn ────────────────────────────────────────

#[test]
fn late_grade_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, Some(real_bolt), BumpGrade::Late);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Late grade must NOT spawn a phantom bolt"
    );
}

// ── F4 — breaker is NOT a PhantomBreaker → no spawn ───────────────────────

#[test]
fn non_phantom_breaker_does_not_spawn_phantom_bolt() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (_phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // A real breaker (NOT a PhantomBreaker).
    let real_breaker = app.world_mut().spawn(Breaker).id();
    write_bump_performed(&mut app, real_breaker, Some(real_bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "Perfect bump attributed to a REAL breaker (no PhantomBreaker marker) \
         must NOT spawn a phantom bolt"
    );
}

// ── F4 (edge case) — placeholder / despawned breaker → no spawn, no panic ─

#[test]
fn placeholder_breaker_does_not_spawn_and_does_not_panic() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (_phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(
        &mut app,
        Entity::PLACEHOLDER,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "placeholder breaker must not spawn a phantom bolt"
    );
}

// ── F5 — bolt: None (spectator) is a no-op ────────────────────────────────

#[test]
fn spectator_bump_with_bolt_none_is_noop() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(&mut app, phantom_breaker, None, BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "bolt: None → no phantom-bolt entity spawned for the pre-existing real bolt"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "bolt: None → zero new phantom-bolt entities in world"
    );
}

// ── F6 — despawned bolt is tolerated ──────────────────────────────────────

#[test]
fn despawned_bolt_is_tolerated() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Despawn the real bolt before writing the bump message.
    app.world_mut().entity_mut(real_bolt).despawn();

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    // No panic. No phantom spawned.
    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "despawned bolt must not result in a phantom-bolt spawn"
    );
}

// ── F7 — one-phantom-per-owner uniqueness (no re-spawn, no duration reset) ─

#[test]
fn second_perfect_bump_does_not_spawn_second_phantom_or_reset_duration() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    // Pre-seed a phantom-bolt entity for real_bolt with lifetime 1.0.
    let existing =
        spawn_phantom_bolt_entity(&mut app, real_bolt, Vec2::ZERO, Vec2::new(0.0, 400.0), 1.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "uniqueness: only ONE phantom-bolt entity for real_bolt after second Perfect bump"
    );
    assert_eq!(owned[0], existing, "the pre-existing phantom must survive");
    let lifetime = app.world().get::<PhantomLifetime>(existing).unwrap().0;
    let expected = 1.0 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "existing phantom lifetime must NOT be reset to 3.0 — expected ~{expected}, got {lifetime}"
    );
}

// ── F7 (edge case) — 2.99 existing lifetime stays near 2.99, not 3.0 ──────

#[test]
fn second_perfect_bump_with_2_99_existing_lifetime_does_not_reset() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);
    let existing =
        spawn_phantom_bolt_entity(&mut app, real_bolt, Vec2::ZERO, Vec2::new(0.0, 400.0), 2.99);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let lifetime = app.world().get::<PhantomLifetime>(existing).unwrap().0;
    let expected = 2.99 - 1.0 / 64.0;
    assert!(
        (lifetime - expected).abs() < 1e-4,
        "existing lifetime 2.99 must decrement by 1/64, NOT reset to 3.0 — \
         expected ~{expected}, got {lifetime}"
    );
}

// ── F8 — phantom outlives real bolt (stale PhantomOwner) ──────────────────

#[test]
fn phantom_outlives_real_bolt_with_stale_phantom_owner() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(owned.len(), 1);
    let phantom = owned[0];

    // Despawn the real bolt.
    app.world_mut().entity_mut(real_bolt).despawn();
    tick_n(&mut app, 3);

    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom must still exist even though its owner is dead"
    );
    assert_eq!(
        app.world().get::<PhantomOwner>(phantom).unwrap().0,
        real_bolt,
        "PhantomOwner must retain the stale Entity reference"
    );
    assert!(
        app.world().get_entity(real_bolt).is_err(),
        "real bolt must be dead"
    );
    let lifetime = app.world().get::<PhantomLifetime>(phantom).unwrap().0;
    // At spawn lifetime is ~3.0; after 1 (same-tick) + 3 ticks = 4 total
    // ticks, lifetime ≈ 3.0 - 4/64.
    let expected_upper = 3.0 - 3.0 / 64.0;
    let expected_lower = 3.0 - 4.0 / 64.0;
    assert!(
        lifetime < expected_upper + 1e-4 && lifetime > expected_lower - 1e-4,
        "phantom lifetime must have decremented ~4/64 via tick_phantom_lifetime, got {lifetime}"
    );
}

// ── F9 — harness-safe: AfterimageConfig absent drains reader ──────────────

#[test]
fn absent_afterimage_config_drains_reader_does_not_retro_spawn() {
    let mut app = build_afterimage_app_no_config();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);
    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "tick 1 with absent AfterimageConfig → no spawn"
    );

    // Insert the config; tick 2 — the message has already been drained.
    app.world_mut()
        .insert_resource(super::super::system::AfterimageConfig {
            phantom_duration:      2.0,
            phantom_bolt_duration: 3.0,
        });
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "tick 2 must NOT retro-apply the drained BumpPerformed message"
    );
}

// ── F10 — ActiveProtocols missing Afterimage → no spawn (run_if gate) ─────

#[test]
fn active_protocols_missing_afterimage_does_not_spawn() {
    let mut app = build_afterimage_app();
    // Do NOT seed ActiveProtocols.
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 400.0), 10.0, 6.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert!(
        phantom_bolts_owned_by(&mut app, real_bolt).is_empty(),
        "run_if gate: no Afterimage in ActiveProtocols → no phantom-bolt spawn"
    );
}

// ── BaseSpeed inheritance — phantom copies real bolt's BaseSpeed ──────────

#[test]
fn phantom_bolt_inherits_base_speed_from_real_bolt_when_present() {
    // Given: a real bolt with BaseSpeed(500.0) and Velocity2D(0.0, 300.0).
    // Note the magnitudes diverge deliberately — if the BaseSpeed-present
    // branch runs, the phantom inherits 500.0; if the fallback ran instead,
    // the phantom would pick up `vel.length()` == 300.0.
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(0.0, 300.0), 10.0, 6.0);
    app.world_mut()
        .entity_mut(real_bolt)
        .insert(BaseSpeed(500.0));

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    let owned = phantom_bolts_owned_by(&mut app, real_bolt);
    assert_eq!(
        owned.len(),
        1,
        "expected exactly one phantom-bolt entity owned by real_bolt, got {}",
        owned.len()
    );
    let phantom = owned[0];
    let phantom_base_speed = app
        .world()
        .get::<BaseSpeed>(phantom)
        .expect("phantom bolt must have BaseSpeed")
        .0;
    assert!(
        (phantom_base_speed - 500.0).abs() < f32::EPSILON,
        "phantom BaseSpeed must inherit the real bolt's BaseSpeed (500.0) \
         — NOT the velocity magnitude fallback (300.0), got {phantom_base_speed}"
    );
}
