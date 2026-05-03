//! Tests #12a–#12d: phantom lifespan expiry must NOT emit kill/damage/run-loss messages.

use bevy::prelude::*;

use super::helpers::spawn_phantom;
use crate::{breaker::components::Breaker, prelude::*};

/// Builds an `App` with `RantzDmgPlugin` (registers `KillYourself<Breaker>`,
/// `Destroyed<Breaker>`, `DamageDealt<Breaker>`, `HealDealt<Breaker>`),
/// `RunLost` message, and all relevant collectors. The test apps share this
/// shape and only vary in which extra collectors are attached.
fn negative_test_app() -> App {
    use super::super::tick_phantom_breaker_lifespan;
    let mut app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .with_message_capture::<DespawnEntity>()
        .with_system(FixedUpdate, tick_phantom_breaker_lifespan)
        .build();
    // KillYourself<Breaker>, Destroyed<Breaker>, DamageDealt<Breaker>, HealDealt<Breaker>
    // are registered by register_dmgable::<Breaker>() inside with_dmg_pipeline().
    attach_message_capture::<KillYourself<Breaker>>(&mut app);
    attach_message_capture::<Destroyed<Breaker>>(&mut app);
    attach_message_capture::<DamageDealt<Breaker>>(&mut app);
    attach_message_capture::<HealDealt<Breaker>>(&mut app);
    // RunLost is a game-domain message not registered by RantzDmgPlugin.
    app.add_message::<RunLost>();
    attach_message_capture::<RunLost>(&mut app);
    app
}

// ── Test #12a — No KillYourself<Breaker> emitted on lifespan expiry ──

#[test]
fn lifespan_expiry_does_not_emit_kill_yourself_breaker() {
    let mut app = negative_test_app();
    spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    let kill = app
        .world()
        .resource::<MessageCollector<KillYourself<Breaker>>>();
    assert!(
        kill.0.is_empty(),
        "KillYourself<Breaker> must NOT be emitted on lifespan expiry"
    );

    // Positive control: DespawnEntity was emitted (test is not a false pass from missing registration).
    let despawn = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn.0.len(),
        1,
        "positive control: DespawnEntity was emitted"
    );
}

// ── Test #12a edge — KillYourself<Breaker> message resource actually exists ──

#[test]
fn kill_yourself_breaker_message_resource_is_registered() {
    let app = negative_test_app();
    assert!(
        app.world()
            .get_resource::<Messages<KillYourself<Breaker>>>()
            .is_some(),
        "Messages<KillYourself<Breaker>> must be registered; empty collector is meaningful"
    );
}

// ── Test #12b — No Destroyed<Breaker> emitted on lifespan expiry ──

#[test]
fn lifespan_expiry_does_not_emit_destroyed_breaker() {
    let mut app = negative_test_app();
    spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Breaker>>>();
    assert!(
        destroyed.0.is_empty(),
        "Destroyed<Breaker> must NOT be emitted on lifespan expiry"
    );

    let despawn = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn.0.len(),
        1,
        "positive control: DespawnEntity was emitted"
    );
}

// ── Test #12b edge — Destroyed<Breaker> message resource is registered ──

#[test]
fn destroyed_breaker_message_resource_is_registered() {
    let app = negative_test_app();
    assert!(
        app.world()
            .get_resource::<Messages<Destroyed<Breaker>>>()
            .is_some(),
        "Messages<Destroyed<Breaker>> must be registered; empty collector is meaningful"
    );
}

// ── Test #12c — No RunLost emitted on lifespan expiry ──

#[test]
fn lifespan_expiry_does_not_emit_run_lost() {
    let mut app = negative_test_app();
    spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    let run_lost = app.world().resource::<MessageCollector<RunLost>>();
    assert!(
        run_lost.0.is_empty(),
        "RunLost must NOT be emitted on phantom lifespan expiry"
    );

    let despawn = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn.0.len(),
        1,
        "positive control: DespawnEntity was emitted"
    );
}

// ── Test #12c edge — RunLost message resource is registered ──

#[test]
fn run_lost_message_resource_is_registered() {
    let app = negative_test_app();
    assert!(
        app.world().get_resource::<Messages<RunLost>>().is_some(),
        "Messages<RunLost> must be registered; empty collector is meaningful"
    );
}

// ── Test #12d — Only DespawnEntity emitted; all other message channels are empty ──

#[test]
fn lifespan_expiry_emits_only_despawn_entity_nothing_else() {
    let mut app = negative_test_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    // Load-bearing positive: exactly one DespawnEntity for this phantom.
    let despawn = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn.0.len(),
        1,
        "exactly one DespawnEntity must be emitted"
    );
    assert_eq!(
        despawn.0[0].entity, phantom,
        "DespawnEntity.entity must match the phantom"
    );

    // All others must be empty.
    let kill = app
        .world()
        .resource::<MessageCollector<KillYourself<Breaker>>>();
    assert!(kill.0.is_empty(), "KillYourself<Breaker> must be empty");

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Breaker>>>();
    assert!(destroyed.0.is_empty(), "Destroyed<Breaker> must be empty");

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Breaker>>>();
    assert!(damage.0.is_empty(), "DamageDealt<Breaker> must be empty");

    let heal = app
        .world()
        .resource::<MessageCollector<HealDealt<Breaker>>>();
    assert!(heal.0.is_empty(), "HealDealt<Breaker> must be empty");

    let run_lost = app.world().resource::<MessageCollector<RunLost>>();
    assert!(run_lost.0.is_empty(), "RunLost must be empty");
}

// ── Test #12d edge — two phantoms expiring together: two DespawnEntities, nothing else ──

#[test]
fn two_phantoms_expiring_emit_two_despawn_entities_nothing_else() {
    let mut app = negative_test_app();
    let p1 = spawn_phantom(&mut app, 0.01);
    let p2 = spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    let despawn = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn.0.len(),
        2,
        "two phantoms expiring in one tick must emit two DespawnEntity messages"
    );

    // Order-independent check: both entities appear in the captured set.
    let entities: std::collections::HashSet<Entity> = despawn.0.iter().map(|d| d.entity).collect();
    assert!(
        entities.contains(&p1),
        "DespawnEntity for phantom 1 must be present"
    );
    assert!(
        entities.contains(&p2),
        "DespawnEntity for phantom 2 must be present"
    );

    let kill = app
        .world()
        .resource::<MessageCollector<KillYourself<Breaker>>>();
    assert!(
        kill.0.is_empty(),
        "KillYourself<Breaker> must be empty with two expirees"
    );

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<Breaker>>>();
    assert!(
        destroyed.0.is_empty(),
        "Destroyed<Breaker> must be empty with two expirees"
    );

    let damage = app
        .world()
        .resource::<MessageCollector<DamageDealt<Breaker>>>();
    assert!(
        damage.0.is_empty(),
        "DamageDealt<Breaker> must be empty with two expirees"
    );

    let heal = app
        .world()
        .resource::<MessageCollector<HealDealt<Breaker>>>();
    assert!(
        heal.0.is_empty(),
        "HealDealt<Breaker> must be empty with two expirees"
    );

    let run_lost = app.world().resource::<MessageCollector<RunLost>>();
    assert!(
        run_lost.0.is_empty(),
        "RunLost must be empty with two expirees"
    );
}
