//! `F1` — spawn bundle verification and `BaseSpeed` inheritance.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::BaseSpeed;

use super::helpers::*;
use crate::{
    bolt::components::{BoltBaseDamage, ExtraBolt},
    breaker::messages::BumpGrade,
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
};

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
