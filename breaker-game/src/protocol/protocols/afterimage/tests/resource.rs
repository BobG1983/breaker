//! Group B — Component shape (Behaviors B1–B4).
//!
//! Pins that:
//! - `PhantomBreaker` + `PhantomBreakerLifetime` round-trip through
//!   `Commands::insert` and queries.
//! - `PhantomBreakerLifetime` derives `Component + Debug + Clone + Copy +
//!   PartialEq`.
//! - Afterimage re-uses the existing `PhantomBolt` / `PhantomLifetime` /
//!   `PhantomOwner` from `crate::effect_v3::effects::phantom_bolt::components`
//!   — NOT an afterimage-local shadow.
//! - A `PhantomBreaker` entity round-trips through query-with-position.

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::{PhantomBreaker, PhantomBreakerLifetime};
use crate::{
    effect_v3::effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
    prelude::*,
};

// ── B1 — PhantomBreakerLifetime round-trips through Commands + queries ─────

#[test]
fn phantom_breaker_lifetime_round_trips_via_commands() {
    let mut app = TestAppBuilder::new().build();
    let entity = app.world_mut().spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands
            .entity(entity)
            .insert((PhantomBreaker, PhantomBreakerLifetime(1.5)));
    }
    queue.apply(app.world_mut());

    assert!(
        app.world().get::<PhantomBreaker>(entity).is_some(),
        "PhantomBreaker must be present after Commands::insert"
    );
    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(entity)
        .expect("PhantomBreakerLifetime must be attached after insert");
    assert!(
        (lifetime.0 - 1.5).abs() < f32::EPSILON,
        "PhantomBreakerLifetime.0 expected 1.5, got {}",
        lifetime.0
    );
}

// ── B1 (edge case) — 0.0 survives the round-trip ───────────────────────────

#[test]
fn phantom_breaker_lifetime_round_trips_with_zero_value() {
    let mut app = TestAppBuilder::new().build();
    let entity = app.world_mut().spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands
            .entity(entity)
            .insert((PhantomBreaker, PhantomBreakerLifetime(0.0)));
    }
    queue.apply(app.world_mut());

    let lifetime = app
        .world()
        .get::<PhantomBreakerLifetime>(entity)
        .expect("PhantomBreakerLifetime must survive the insert with value 0.0");
    assert!(
        lifetime.0.abs() < f32::EPSILON,
        "PhantomBreakerLifetime.0 expected 0.0, got {}",
        lifetime.0
    );
}

// ── B2 — PhantomBreakerLifetime derives Debug + Clone + Copy + PartialEq ───

#[test]
fn phantom_breaker_lifetime_derives_debug_clone_copy_partial_eq() {
    let lifetime = PhantomBreakerLifetime(2.0);

    // Debug: renders with the type name and the value.
    let debug = format!("{lifetime:?}");
    assert!(
        debug.contains("PhantomBreakerLifetime"),
        "debug string must contain the type name, got {debug}"
    );
    assert!(
        debug.contains("2.0") || debug.contains('2'),
        "debug string must contain the value 2.0, got {debug}"
    );

    // Copy: let copy = lifetime; original still usable.
    let copy = lifetime;
    let clone = lifetime;
    assert!(
        (copy.0 - 2.0).abs() < f32::EPSILON,
        "copy.0 must equal 2.0, got {}",
        copy.0
    );
    assert!(
        (clone.0 - 2.0).abs() < f32::EPSILON,
        "clone.0 must equal 2.0, got {}",
        clone.0
    );
    assert!(
        (lifetime.0 - 2.0).abs() < f32::EPSILON,
        "original lifetime.0 must still be usable after Copy — expected 2.0, got {}",
        lifetime.0
    );

    // PartialEq equality.
    assert_eq!(
        PhantomBreakerLifetime(2.0),
        PhantomBreakerLifetime(2.0),
        "identical values must compare =="
    );
    // PartialEq inequality.
    assert_ne!(
        PhantomBreakerLifetime(2.0),
        PhantomBreakerLifetime(1.0),
        "different values must compare !="
    );
}

// ── B3 — afterimage re-uses effect_v3 PhantomBolt / PhantomLifetime / ──────
//       PhantomOwner (not a local shadow).

#[test]
fn afterimage_reuses_effect_v3_phantom_components() {
    // (a) Type references resolve — these lines compile only when the
    //     imports match the canonical types.
    let _: PhantomBolt = PhantomBolt;
    let _: PhantomLifetime = PhantomLifetime(1.0);

    let mut app = TestAppBuilder::new().build();
    let dummy = app.world_mut().spawn_empty().id();
    let _: PhantomOwner = PhantomOwner(dummy);

    // (b) Afterimage's system module re-exports the canonical types —
    //     prove it by comparing type_name strings.
    let name = std::any::type_name::<super::super::system::PhantomBolt>();
    assert!(
        name.contains("effect_v3::effects::phantom_bolt"),
        "afterimage::system::PhantomBolt must be the canonical effect_v3 type, \
         got type_name = {name}"
    );
    assert_eq!(
        std::any::type_name::<PhantomBolt>(),
        std::any::type_name::<super::super::system::PhantomBolt>(),
        "afterimage's PhantomBolt import must be the SAME type as \
         effect_v3::effects::phantom_bolt::components::PhantomBolt"
    );
}

// ── B4 — PhantomBreaker entity round-trips with Position2D ─────────────────

#[test]
fn phantom_breaker_entity_round_trips_with_position2d() {
    let mut app = TestAppBuilder::new().build();

    app.world_mut().spawn((
        PhantomBreaker,
        PhantomBreakerLifetime(1.5),
        Position2D(Vec2::new(50.0, 75.0)),
    ));

    let world = app.world_mut();
    let mut q =
        world.query_filtered::<(&PhantomBreakerLifetime, &Position2D), With<PhantomBreaker>>();
    let rows: Vec<(PhantomBreakerLifetime, Vec2)> = q.iter(world).map(|(l, p)| (*l, p.0)).collect();
    assert_eq!(
        rows.len(),
        1,
        "expected exactly one PhantomBreaker row, got {}",
        rows.len()
    );
    assert!(
        (rows[0].0.0 - 1.5).abs() < f32::EPSILON,
        "lifetime expected 1.5, got {}",
        rows[0].0.0
    );
    assert!(
        (rows[0].1 - Vec2::new(50.0, 75.0)).length() < f32::EPSILON,
        "position expected (50.0, 75.0), got {:?}",
        rows[0].1
    );
}
