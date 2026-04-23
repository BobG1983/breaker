use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::system::*;
use crate::{
    components::Dead,
    messages::{DespawnEntity, Destroyed, KillYourself},
    traits::Dmgable,
};

#[derive(Component)]
struct TestT;
impl Dmgable for TestT {}

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    if expected.is_infinite() {
        assert!(
            actual.is_infinite() && actual.is_sign_positive() == expected.is_sign_positive(),
            "expected {expected}, got {actual}"
        );
    } else {
        assert!(
            (actual - expected).abs() < f32::EPSILON,
            "expected {expected}, got {actual}"
        );
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<KillYourself<TestT>>();
    app.add_message::<Destroyed<TestT>>();
    app.add_message::<DespawnEntity>();
    app.add_systems(FixedUpdate, handle_kill::<TestT>);
    app
}

fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn enqueue_kill(app: &mut App, msg: KillYourself<TestT>) {
    app.world_mut()
        .resource_mut::<Messages<KillYourself<TestT>>>()
        .write(msg);
}

fn drain_destroyed(app: &mut App) -> Vec<Destroyed<TestT>> {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<TestT>>>()
        .drain()
        .collect()
}

fn drain_despawn(app: &mut App) -> Vec<DespawnEntity> {
    app.world_mut()
        .resource_mut::<Messages<DespawnEntity>>()
        .drain()
        .collect()
}

fn mk_kill(victim: Entity, killer: Option<Entity>) -> KillYourself<TestT> {
    KillYourself::<TestT> {
        victim,
        killer,
        _marker: PhantomData,
    }
}

// ── Behavior 111: handle_kill inserts Dead on the victim ──

#[test]
fn inserts_dead_on_victim() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    assert!(app.world().get::<Dead>(victim).is_some());
}

#[test]
fn dead_persists_across_idle_tick() {
    // Edge case 111a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);
    tick(&mut app);

    assert!(app.world().get::<Dead>(victim).is_some());
}

// ── Behavior 112: writes Destroyed with victim_pos ──

#[test]
fn writes_destroyed_with_victim_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].victim, victim);
    assert_f32_eq(drained[0].victim_pos.x, 10.0);
    assert_f32_eq(drained[0].victim_pos.y, 20.0);
    assert!(drained[0].killer.is_none());
    assert!(drained[0].killer_pos.is_none());
}

#[test]
fn writes_destroyed_with_zero_victim_pos() {
    // Edge case 112a: Vec2::ZERO is recorded verbatim, not treated as missing.
    let mut app = test_app();
    let victim = app.world_mut().spawn((TestT, Position2D(Vec2::ZERO))).id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_f32_eq(drained[0].victim_pos.x, 0.0);
    assert_f32_eq(drained[0].victim_pos.y, 0.0);
}

// ── Behavior 113: killer_pos from killer's Position2D ──

#[test]
fn writes_destroyed_with_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].killer, Some(killer));
    let killer_pos = drained[0].killer_pos.expect("killer_pos should be Some");
    assert_f32_eq(killer_pos.x, 50.0);
    assert_f32_eq(killer_pos.y, 60.0);
}

#[test]
fn writes_destroyed_with_negative_killer_pos() {
    // Edge case 113a.
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(-100.0, -200.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    let kp = drained[0].killer_pos.unwrap();
    assert_f32_eq(kp.x, -100.0);
    assert_f32_eq(kp.y, -200.0);
}

// ── Behavior 114: killer without Position2D → killer_pos: None ──

#[test]
fn killer_without_position_emits_none_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();
    let killer = app.world_mut().spawn_empty().id();

    enqueue_kill(&mut app, mk_kill(victim, Some(killer)));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].killer, Some(killer));
    assert!(drained[0].killer_pos.is_none());
}

// ── Behavior 115: killer: None → killer_pos: None ──

#[test]
fn none_killer_yields_none_killer_pos() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_destroyed(&mut app);
    assert_eq!(drained.len(), 1);
    assert!(drained[0].killer.is_none());
    assert!(drained[0].killer_pos.is_none());
}

// ── Behavior 116: one DespawnEntity per unique victim ──

#[test]
fn writes_one_despawn_per_victim() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let drained = drain_despawn(&mut app);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].entity, victim);
}

#[test]
fn writes_one_despawn_per_each_of_three_victims() {
    // Edge case 116a.
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 1.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(2.0, 2.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 3.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, None));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let drained = drain_despawn(&mut app);
    assert_eq!(drained.len(), 3);
    let entities: Vec<Entity> = drained.iter().map(|d| d.entity).collect();
    assert!(entities.contains(&v1));
    assert!(entities.contains(&v2));
    assert!(entities.contains(&v3));
}

// ── Behavior 117: same-frame duplicate for same victim dedupes ──

#[test]
fn same_frame_duplicate_dedupes() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 1);
    assert_eq!(despawns.len(), 1);
    assert!(app.world().get::<Dead>(victim).is_some());
}

// ── Behavior 118: cross-frame idempotency via Without<Dead> ──

#[test]
fn cross_frame_dead_victim_is_filtered() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0)), Dead))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert!(destroyed.is_empty());
    assert!(despawns.is_empty());
    assert!(app.world().get::<Dead>(victim).is_some());
}

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

// ── Behavior 122: multiple distinct victims in one tick ──

#[test]
fn multiple_victims_in_one_tick() {
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 4.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(5.0, 6.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, None));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 3);
    assert_eq!(despawns.len(), 3);
    assert!(app.world().get::<Dead>(v1).is_some());
    assert!(app.world().get::<Dead>(v2).is_some());
    assert!(app.world().get::<Dead>(v3).is_some());

    // Per-victim victim_pos correctness.
    let pos_for = |victim: Entity| -> Vec2 {
        destroyed
            .iter()
            .find(|d| d.victim == victim)
            .map(|d| d.victim_pos)
            .expect("destroyed for victim")
    };
    let p1 = pos_for(v1);
    let p2 = pos_for(v2);
    let p3 = pos_for(v3);
    assert_f32_eq(p1.x, 1.0);
    assert_f32_eq(p1.y, 2.0);
    assert_f32_eq(p2.x, 3.0);
    assert_f32_eq(p2.y, 4.0);
    assert_f32_eq(p3.x, 5.0);
    assert_f32_eq(p3.y, 6.0);
}

#[test]
fn multiple_victims_with_mixed_killers_no_crossleak() {
    // Edge case 122a.
    let mut app = test_app();
    let v1 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(1.0, 2.0))))
        .id();
    let v2 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(3.0, 4.0))))
        .id();
    let v3 = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(5.0, 6.0))))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(99.0, 99.0)))
        .id();

    enqueue_kill(&mut app, mk_kill(v1, None));
    enqueue_kill(&mut app, mk_kill(v2, Some(killer)));
    enqueue_kill(&mut app, mk_kill(v3, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    assert_eq!(destroyed.len(), 3);

    let find = |v: Entity| destroyed.iter().find(|d| d.victim == v).unwrap();
    assert!(find(v1).killer_pos.is_none());
    let kp2 = find(v2).killer_pos.unwrap();
    assert_f32_eq(kp2.x, 99.0);
    assert_f32_eq(kp2.y, 99.0);
    assert!(find(v3).killer_pos.is_none());
}

// ── Behavior 123: self-kill (victim == killer) records both fields ──

#[test]
fn self_kill_records_both_fields() {
    let mut app = test_app();
    let victim = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(10.0, 20.0))))
        .id();

    enqueue_kill(&mut app, mk_kill(victim, Some(victim)));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    let despawns = drain_despawn(&mut app);
    assert_eq!(destroyed.len(), 1);
    assert_eq!(destroyed[0].victim, victim);
    assert_eq!(destroyed[0].killer, Some(victim));
    let kp = destroyed[0].killer_pos.unwrap();
    assert_f32_eq(kp.x, 10.0);
    assert_f32_eq(kp.y, 20.0);
    assert_f32_eq(destroyed[0].victim_pos.x, 10.0);
    assert_f32_eq(destroyed[0].victim_pos.y, 20.0);
    assert_eq!(despawns.len(), 1);
    assert!(app.world().get::<Dead>(victim).is_some());
}

// ── Behavior 124: Local<HashSet<Entity>> cleared at top of each tick ──

#[test]
fn survives_ten_ticks_with_fresh_victim_each_tick() {
    let mut app = test_app();

    for n in 0..10 {
        let v = app
            .world_mut()
            .spawn((TestT, Position2D(Vec2::new(n as f32, 0.0))))
            .id();
        enqueue_kill(&mut app, mk_kill(v, None));
        tick(&mut app);

        let destroyed = drain_destroyed(&mut app);
        assert_eq!(
            destroyed.len(),
            1,
            "tick {n}: expected exactly one Destroyed"
        );
        assert_eq!(destroyed[0].victim, v);
    }
}

#[test]
fn after_many_ticks_same_frame_dedupe_still_works() {
    // Edge case 124a.
    let mut app = test_app();

    for n in 0..5 {
        let v = app
            .world_mut()
            .spawn((TestT, Position2D(Vec2::new(n as f32, 0.0))))
            .id();
        enqueue_kill(&mut app, mk_kill(v, None));
        tick(&mut app);
        drop(drain_destroyed(&mut app));
        drop(drain_despawn(&mut app));
    }

    // Now a fresh victim with triplicate kills same tick.
    let v = app
        .world_mut()
        .spawn((TestT, Position2D(Vec2::new(99.0, 0.0))))
        .id();
    enqueue_kill(&mut app, mk_kill(v, None));
    enqueue_kill(&mut app, mk_kill(v, None));
    enqueue_kill(&mut app, mk_kill(v, None));
    tick(&mut app);

    let destroyed = drain_destroyed(&mut app);
    assert_eq!(destroyed.len(), 1);
}
