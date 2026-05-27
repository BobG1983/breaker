//! T24 — `SpawnPhantomConfig::fire` phantoms despawn on expiry via
//! `LifetimeEndBehavior::Despawn → DespawnEntity` (Behavior 3 + edge cases).

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::super::config_impl::SpawnPhantomConfig;
use crate::{
    bolt::{
        BoltPlugin,
        components::{Bolt, LifetimeEndBehavior, PhantomBolt, PhantomDedupKey},
    },
    effect_v3::{
        commands::FireEffectCommand, effects::phantom_bolt::components::PhantomLifetime,
        types::EffectType,
    },
    prelude::*,
    shared::Lifespan,
    state::run::resources::NodeOutcome,
};

const FIXED_DT: f32 = 1.0 / 64.0;

fn fire_phantom_in_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_resource::<NodeOutcome>()
        .with_effects_pipeline()
        .build();
    app.add_plugins(BoltPlugin);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

/// Accumulates one fixed timestep then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// ── T24 primary — short-lived phantom despawns via DespawnEntity ─────────────

#[test]
fn phantom_with_short_duration_despawns_via_despawn_entity() {
    let mut app = fire_phantom_in_app();

    let real_bolt = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    // Use FireEffectCommand (visible inside effect_v3)
    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(0.01),
        max_active: 3,
    };
    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(config),
        source: "phantom_bolt".to_string(),
    }
    .apply(app.world_mut());
    app.world_mut().flush();

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(app.world())
        .next()
        .expect("phantom must be spawned after fire()");

    tick(&mut app);

    // Exactly one DespawnEntity emitted for the phantom
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "exactly one DespawnEntity must be emitted on expiry"
    );
    assert_eq!(
        collector.0[0].entity, phantom,
        "DespawnEntity.entity must be the spawned phantom"
    );

    // Phantom entity is gone (process_despawn_requests ran in the same tick)
    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom entity must be despawned after the tick"
    );
}

// ── T24 edge case 3a — long duration leaves phantom alive after one tick ──────

#[test]
fn phantom_with_long_duration_survives_one_tick() {
    let mut app = fire_phantom_in_app();

    let real_bolt = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(2.0),
        max_active: 3,
    };
    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(config),
        source: "phantom_bolt".to_string(),
    }
    .apply(app.world_mut());
    app.world_mut().flush();

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(app.world())
        .next()
        .expect("phantom must be spawned");

    tick(&mut app);

    // No DespawnEntity emitted
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity must be emitted after one tick with duration=2.0"
    );

    // Phantom is still alive with decremented remaining
    let lifespan = app
        .world()
        .get::<Lifespan>(phantom)
        .expect("Lifespan must remain");
    assert!(
        (lifespan.remaining - (2.0 - FIXED_DT)).abs() < 1e-4,
        "remaining must be ~2.0 - FIXED_DT after one tick, got {}",
        lifespan.remaining
    );
}

// ── T24 edge case 3b — legacy tick_phantom_lifetime is a no-op for W4B phantoms

#[test]
fn phantom_does_not_carry_phantom_lifetime_at_expiry() {
    let mut app = fire_phantom_in_app();

    let real_bolt = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(0.01),
        max_active: 3,
    };
    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(config),
        source: "phantom_bolt".to_string(),
    }
    .apply(app.world_mut());
    app.world_mut().flush();

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(app.world())
        .next()
        .expect("phantom must be spawned");

    // Before any tick: the entity must have LifetimeEndBehavior::Despawn and Lifespan
    // but NOT PhantomLifetime (the legacy component that tick_phantom_lifetime reads).
    assert!(
        app.world().get::<PhantomLifetime>(phantom).is_none(),
        "W4B phantom must NOT carry PhantomLifetime (legacy component)"
    );
    assert!(
        app.world().get::<Lifespan>(phantom).is_some(),
        "W4B phantom must carry shared Lifespan"
    );
    assert_eq!(
        app.world().get::<LifetimeEndBehavior>(phantom).copied(),
        Some(LifetimeEndBehavior::Despawn),
        "W4B phantom must carry LifetimeEndBehavior::Despawn"
    );
}

// ── T24 edge case 3c — duration=0.0 despawns on first tick ───────────────────

#[test]
fn phantom_with_zero_duration_despawns_on_first_tick() {
    let mut app = fire_phantom_in_app();

    let real_bolt = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::ZERO), Velocity2D(Vec2::ZERO)))
        .id();

    let config = SpawnPhantomConfig {
        duration:   OrderedFloat(0.0),
        max_active: 1,
    };
    FireEffectCommand {
        entity: real_bolt,
        effect: EffectType::SpawnPhantom(config),
        source: "phantom_bolt".to_string(),
    }
    .apply(app.world_mut());
    app.world_mut().flush();

    let phantom = app
        .world_mut()
        .query_filtered::<Entity, (With<PhantomBolt>, With<PhantomDedupKey>)>()
        .iter(app.world())
        .next()
        .expect("phantom must be spawned with duration=0.0");

    tick(&mut app);

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        collector.0.len(),
        1,
        "duration=0.0 phantom must emit one DespawnEntity on first tick"
    );
    assert!(
        app.world().get_entity(phantom).is_err(),
        "duration=0.0 phantom must be gone after one tick"
    );
}
