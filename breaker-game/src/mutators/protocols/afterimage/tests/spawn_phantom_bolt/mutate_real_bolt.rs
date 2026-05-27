//! T18 — Perfect bump on phantom breaker mutates the real bolt into a phantom
//! (Wave 4A behavioral contract).
//!
//! F1 (`perfect_bump_on_phantom_breaker_spawns_new_phantom_bolt_entity`) and its
//! two edge-case siblings are DELETED here; T18 is their inversion under the
//! mutate-real-bolt design.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    bolt::components::{
        BoltBaseDamage, LifetimeEndBehavior, PhantomBolt, PhantomDamagedCells, PhantomDedupKey,
    },
    breaker::messages::BumpGrade,
    prelude::*,
    shared::{Lifespan, PhantomFlicker},
};

// ── Assertion helpers ─────────────────────────────────────────────────────────

fn assert_phantom_components(world: &World, real_bolt: Entity) {
    assert!(
        world.get::<PhantomBolt>(real_bolt).is_some(),
        "real bolt must gain the PhantomBolt marker"
    );
    let key = world
        .get::<PhantomDedupKey>(real_bolt)
        .expect("real bolt must have PhantomDedupKey");
    assert_eq!(
        *key,
        PhantomDedupKey::Bolt(real_bolt),
        "PhantomDedupKey must be Bolt(real_bolt), got {key:?}"
    );
    let deduped = world
        .get::<PhantomDamagedCells>(real_bolt)
        .expect("real bolt must have PhantomDamagedCells");
    assert!(
        deduped.0.is_empty(),
        "PhantomDamagedCells must be empty on first mutation"
    );
}

fn assert_lifespan_and_behavior(world: &World, real_bolt: Entity) {
    let dt = 1.0_f32 / 64.0;
    let phantom_bolt_duration = 3.0_f32;
    let lifespan = world
        .get::<Lifespan>(real_bolt)
        .expect("real bolt must have Lifespan");
    assert!(
        lifespan.remaining > phantom_bolt_duration - dt - 1e-4
            && lifespan.remaining <= phantom_bolt_duration + 1e-4,
        "Lifespan.remaining expected in ({}, {}], got {}",
        phantom_bolt_duration - dt - 1e-4,
        phantom_bolt_duration + 1e-4,
        lifespan.remaining
    );
    let behavior = world
        .get::<LifetimeEndBehavior>(real_bolt)
        .expect("real bolt must have LifetimeEndBehavior");
    assert_eq!(
        *behavior,
        LifetimeEndBehavior::RevertToNormalBolt,
        "LifetimeEndBehavior must be RevertToNormalBolt"
    );
    let flicker = world
        .get::<PhantomFlicker>(real_bolt)
        .expect("real bolt must have PhantomFlicker");
    assert!(
        (flicker.frequency - 4.0).abs() < 1e-6,
        "PhantomFlicker.frequency must be 4.0, got {}",
        flicker.frequency
    );
    assert!(
        (flicker.min_alpha - 0.3).abs() < 1e-6,
        "PhantomFlicker.min_alpha must be 0.3, got {}",
        flicker.min_alpha
    );
}

fn assert_no_new_entity_spawned(app: &mut App, real_bolt: Entity) {
    assert_eq!(
        phantom_bolt_count(app),
        1,
        "exactly one phantom-bolt entity must exist (the real bolt itself)"
    );
    let phantom_entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, With<PhantomBolt>)>()
        .iter(app.world())
        .collect();
    assert_eq!(
        phantom_entities.len(),
        1,
        "exactly one Bolt+PhantomBolt entity"
    );
    assert_eq!(
        phantom_entities[0], real_bolt,
        "the single phantom bolt must BE the real bolt entity — no new entity spawned"
    );
    let non_phantom_count = app
        .world_mut()
        .query_filtered::<Entity, (With<Bolt>, Without<PhantomBolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        non_phantom_count, 0,
        "no Bolt-without-PhantomBolt entities must remain"
    );
}

fn assert_gameplay_state_preserved(
    world: &World,
    real_bolt: Entity,
    expected_vel: Vec2,
    expected_dmg: f32,
) {
    let pos = world.get::<Position2D>(real_bolt).unwrap().0;
    assert!(
        (pos - Vec2::ZERO).length() < 1e-3,
        "real bolt position broadly unchanged, got {pos:?}"
    );
    let vel = world.get::<Velocity2D>(real_bolt).unwrap().0;
    assert!(
        (vel - expected_vel).length() < 1e-3,
        "real bolt velocity must be approximately {expected_vel:?}, got {vel:?}"
    );
    assert!(
        (world.get::<BoltBaseDamage>(real_bolt).unwrap().0 - expected_dmg).abs() < 1e-6,
        "BoltBaseDamage must be {expected_dmg}"
    );
    assert!(
        world.get::<Bolt>(real_bolt).is_some(),
        "Bolt marker must still be present"
    );
}

// ── T18 — Perfect bump on phantom breaker mutates the real bolt ────────────

#[test]
fn perfect_bump_on_phantom_breaker_mutates_real_bolt_into_phantom() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let (phantom_breaker, real_bolt) = canonical_setup(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_none(),
        "pre-condition: real bolt must NOT have PhantomBolt before tick"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        0,
        "pre-condition: no phantom bolts before tick"
    );

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert_phantom_components(app.world(), real_bolt);
    assert_lifespan_and_behavior(app.world(), real_bolt);
    assert_no_new_entity_spawned(&mut app, real_bolt);
    assert_gameplay_state_preserved(app.world(), real_bolt, Vec2::new(0.0, 400.0), 10.0);
}

// ── T18 edge case — different velocity/damage does not alter gameplay state ─

#[test]
fn perfect_bump_does_not_alter_real_bolt_velocity_or_damage() {
    let mut app = build_afterimage_app();
    seed_active_protocols_with_afterimage(&mut app);
    let phantom_breaker = spawn_phantom_breaker_at(&mut app, Vec2::ZERO, 1.5);
    let real_bolt = spawn_real_bolt(&mut app, Vec2::ZERO, Vec2::new(50.0, -300.0), 5.0, 6.0);

    write_bump_performed(
        &mut app,
        phantom_breaker,
        Some(real_bolt),
        BumpGrade::Perfect,
    );
    tick(&mut app);

    assert!(
        app.world().get::<PhantomBolt>(real_bolt).is_some(),
        "real bolt must gain PhantomBolt marker"
    );

    let vel = app.world().get::<Velocity2D>(real_bolt).unwrap().0;
    assert!(
        (vel - Vec2::new(50.0, -300.0)).length() < 1e-3,
        "velocity must be approximately (50.0, -300.0), got {vel:?}"
    );
    assert!(
        (app.world().get::<BoltBaseDamage>(real_bolt).unwrap().0 - 5.0).abs() < 1e-6,
        "BoltBaseDamage must remain 5.0 after mutation"
    );
    assert_eq!(
        phantom_bolt_count(&mut app),
        1,
        "exactly one phantom bolt (the real bolt itself)"
    );
}
