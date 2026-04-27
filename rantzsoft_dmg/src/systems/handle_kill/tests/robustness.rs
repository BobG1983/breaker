use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::handle_kill,
    helpers::{TestT, drain_despawn, drain_destroyed, enqueue_kill, mk_kill, test_app, tick},
};
use crate::{
    components::Dead,
    messages::{DespawnEntity, Destroyed, KillYourself},
};

// ── Behavior 119: despawned victim is silently skipped ──

#[test]
fn despawned_victim_is_skipped() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    app.world_mut().despawn(victim);

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
}

#[test]
fn despawned_victim_and_killer_both_skipped() {
    // Edge case 119a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();
    app.world_mut().despawn(victim);
    app.world_mut().despawn(killer);

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
}

// ── Behavior 120: victim missing Position2D still transitions to
//     Dead but emits no Destroyed / DespawnEntity ──

#[test]
fn victim_missing_position_is_skipped() {
    let mut app = test_app();
    let victim = app.world_mut().spawn(TestT).id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    // Dead IS inserted — essential so `detect_deaths` doesn't
    // re-emit KillYourself for this entity every tick.
    assert!(app.world().get::<Dead>(victim).is_some());
    // But no Destroyed / DespawnEntity — those require Position2D.
    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
}

// ── Behavior 120b (regression): position-less zero-HP victim does
//     NOT cause detect_deaths → handle_kill to loop indefinitely ──

#[test]
fn position_less_victim_does_not_cause_infinite_kill_loop() {
    use crate::{components::Hp, systems::detect_deaths};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<KillYourself<TestT>>();
    app.add_message::<Destroyed<TestT>>();
    app.add_message::<DespawnEntity>();
    app.add_systems(
        FixedUpdate,
        (detect_deaths::<TestT>, handle_kill::<TestT>).chain(),
    );

    // Victim at zero HP with no Position2D: detect_deaths would
    // emit KillYourself, handle_kill would previously skip without
    // marking Dead, and detect_deaths would re-emit on every tick.
    app.world_mut().spawn((TestT, Hp::new(0.0)));

    // Tick 1: detect_deaths emits KillYourself (victim is alive,
    // non-Dead, hp=0); handle_kill consumes and inserts Dead.
    tick(&mut app);
    // Drain tick 1's emission to get a clean baseline for the next
    // tick's observation (the message is in the buffer because
    // MessageReader advances a cursor but does not remove).
    let _ = app
        .world_mut()
        .resource_mut::<Messages<KillYourself<TestT>>>()
        .drain()
        .count();

    // Tick 2: Dead is set; detect_deaths's `Without<Dead>` filter
    // excludes the entity; no KillYourself is emitted. Proof that
    // the infinite loop bug is fixed.
    tick(&mut app);
    assert!(
        app.world_mut()
            .resource_mut::<Messages<KillYourself<TestT>>>()
            .drain()
            .next()
            .is_none(),
        "tick 2 must not re-emit KillYourself — entity is Dead"
    );

    // Tick 3: stays clean — proves the no-emission invariant holds
    // not just once.
    tick(&mut app);
    assert!(
        app.world_mut()
            .resource_mut::<Messages<KillYourself<TestT>>>()
            .drain()
            .next()
            .is_none()
    );
}

// ── Behavior 121: no pending messages → no-op ──

#[test]
fn no_pending_messages_is_noop() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
    assert!(app.world().get::<Dead>(victim).is_none());
}

#[test]
fn three_idle_ticks_emit_nothing() {
    // Edge case 121a.
    let mut app = test_app();
    app.world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))));

    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
}
