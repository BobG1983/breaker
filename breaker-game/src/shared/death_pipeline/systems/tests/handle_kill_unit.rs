//! Pure unit tests for `handle_kill<TestEntity>` (Behaviors 1-12).
//!
//! Each `#[test]` here uses `TestEntity` (not `Cell`/`Bolt`), so these tests
//! don't require the full `DeathPipelinePlugin` or `EffectV3Plugin`. Plugin
//! wiring is verified separately in `handle_kill_integration.rs`.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::{PendingKillRequests, TestEntity, build_handle_kill_app, kill_msg};
use crate::shared::{
    death_pipeline::{
        dead::Dead, despawn_entity::DespawnEntity, destroyed::Destroyed, hp::Hp,
        kill_yourself::KillYourself, killed_by::KilledBy,
    },
    test_utils::{MessageCollector, tick},
};

// ── Behavior 1: handle_kill inserts `Dead` marker on the victim ──────────

#[test]
fn handle_kill_inserts_dead_marker_on_victim() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(victim).is_some(),
        "handle_kill should insert Dead marker on the victim"
    );
}

#[test]
fn handle_kill_dead_marker_persists_across_idle_tick() {
    // Edge case: tick a second time with no new pending messages — `Dead`
    // remains present (not re-inserted, not removed).
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);
    // Clear pending so the second tick is idle.
    app.insert_resource(PendingKillRequests(vec![]));
    tick(&mut app);

    assert!(
        app.world().get::<Dead>(victim).is_some(),
        "Dead marker should persist across an idle tick"
    );
}

// ── Behavior 2: handle_kill writes Destroyed<T> with correct position ───

#[test]
fn handle_kill_writes_destroyed_with_victim_position() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "handle_kill should write exactly one Destroyed message"
    );
    let msg = &destroyed.0[0];
    assert_eq!(msg.victim, victim);
    assert!((msg.victim_pos.x - 10.0).abs() < f32::EPSILON);
    assert!((msg.victim_pos.y - 20.0).abs() < f32::EPSILON);
    assert_eq!(msg.killer, None);
    assert_eq!(msg.killer_pos, None);
}

#[test]
fn handle_kill_records_victim_pos_zero_not_as_missing() {
    // Edge case: victim at Vec2::ZERO — victim_pos is exactly (0.0, 0.0),
    // not treated as "no position".
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].victim_pos, Vec2::ZERO);
}

// ── Behavior 3: handle_kill captures killer position ────────────────────

#[test]
fn handle_kill_captures_killer_position_when_present() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, Some(killer))]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    let msg = &destroyed.0[0];
    assert_eq!(msg.killer, Some(killer));
    let killer_pos = msg.killer_pos.expect("killer_pos should be Some");
    assert!((killer_pos.x - 50.0).abs() < f32::EPSILON);
    assert!((killer_pos.y - 60.0).abs() < f32::EPSILON);
}

#[test]
fn handle_kill_captures_negative_killer_position() {
    // Edge case: negative coordinates captured exactly.
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(-100.0, -200.0)))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, Some(killer))]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    assert_eq!(destroyed.0[0].killer_pos, Some(Vec2::new(-100.0, -200.0)));
}

// ── Behavior 4: killer_pos = None when killer lacks Position2D ───────────

#[test]
fn handle_kill_sets_killer_pos_none_when_killer_has_no_position() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    let killer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, Some(killer))]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    let msg = &destroyed.0[0];
    assert_eq!(
        msg.killer,
        Some(killer),
        "killer id is still carried — attribution is preserved"
    );
    assert_eq!(msg.killer_pos, None);
}

#[test]
fn handle_kill_per_message_killer_pos_no_cross_contamination() {
    // Edge case: two messages in the same tick — one with positioned killer,
    // one with unpositioned killer. Each Destroyed records its own correct
    // killer_pos (no cross-contamination between iterations).
    let mut app = build_handle_kill_app();
    let victim_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(1.0, 2.0)),
        ))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(3.0, 4.0)),
        ))
        .id();
    let positioned_killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();
    let unpositioned_killer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingKillRequests(vec![
        kill_msg(victim_a, Some(positioned_killer)),
        kill_msg(victim_b, Some(unpositioned_killer)),
    ]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 2);

    // Find each message by victim (order independent).
    let a = destroyed
        .0
        .iter()
        .find(|m| m.victim == victim_a)
        .expect("victim_a Destroyed missing");
    let b = destroyed
        .0
        .iter()
        .find(|m| m.victim == victim_b)
        .expect("victim_b Destroyed missing");

    assert_eq!(a.killer_pos, Some(Vec2::new(50.0, 60.0)));
    assert_eq!(b.killer_pos, None);
}

// ── Behavior 5: killer_pos = None when killer is None ───────────────────

#[test]
fn handle_kill_sets_killer_pos_none_when_killer_is_none() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    let msg = &destroyed.0[0];
    assert_eq!(msg.killer, None);
    assert_eq!(msg.killer_pos, None);
}

// ── Behavior 6: handle_kill writes exactly one DespawnEntity ────────────

#[test]
fn handle_kill_writes_despawn_entity_for_victim() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawns.0.len(),
        1,
        "handle_kill should write exactly one DespawnEntity"
    );
    assert_eq!(despawns.0[0].entity, victim);
}

#[test]
fn handle_kill_writes_one_despawn_per_distinct_victim() {
    // Edge case: TWO KillYourself messages for TWO different victims in the
    // same tick — two DespawnEntity messages, each referencing its own victim.
    let mut app = build_handle_kill_app();
    let victim_a = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(1.0, 2.0)),
        ))
        .id();
    let victim_b = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(3.0, 4.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![
        kill_msg(victim_a, None),
        kill_msg(victim_b, None),
    ]));
    tick(&mut app);

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(despawns.0.len(), 2);
    let entities: Vec<Entity> = despawns.0.iter().map(|m| m.entity).collect();
    assert!(entities.contains(&victim_a));
    assert!(entities.contains(&victim_b));
}

// ── Behavior 7: Same-frame dedupe ───────────────────────────────────────

#[test]
fn handle_kill_dedupes_same_frame_duplicate_kill_messages() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![
        kill_msg(victim, None),
        kill_msg(victim, None),
    ]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(
        destroyed.0.len(),
        1,
        "handle_kill must same-frame-dedupe — exactly one Destroyed emitted"
    );

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawns.0.len(),
        1,
        "handle_kill must same-frame-dedupe — exactly one DespawnEntity emitted"
    );

    assert!(
        app.world().get::<Dead>(victim).is_some(),
        "Dead should still be inserted after dedupe"
    );
}

#[test]
fn handle_kill_cross_frame_idempotency_skips_already_dead_victim() {
    // Edge case (cross-frame): same victim spawned with Dead already present;
    // one KillYourself pending; tick runs. No Destroyed, no DespawnEntity.
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
            Dead,
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(
        destroyed.0.is_empty(),
        "cross-frame idempotency: already-Dead victim should not emit Destroyed"
    );
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        despawns.0.is_empty(),
        "cross-frame idempotency: already-Dead victim should not emit DespawnEntity"
    );
    assert!(
        app.world().get::<Dead>(victim).is_some(),
        "Dead marker still present"
    );
}

// ── Behavior 8: Skips victims despawned before system runs ──────────────

#[test]
fn handle_kill_skips_victim_despawned_before_system_runs() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    app.world_mut().despawn(victim);

    app.insert_resource(PendingKillRequests(vec![KillYourself::<TestEntity> {
        victim,
        killer: None,
        _marker: PhantomData,
    }]));
    // Should not panic
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(destroyed.0.is_empty());
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(despawns.0.is_empty());
}

#[test]
fn handle_kill_skips_when_both_victim_and_killer_despawned() {
    // Edge case: both victim and killer despawned before the tick.
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(50.0, 60.0)))
        .id();
    app.world_mut().despawn(victim);
    app.world_mut().despawn(killer);

    app.insert_resource(PendingKillRequests(vec![KillYourself::<TestEntity> {
        victim,
        killer: Some(killer),
        _marker: PhantomData,
    }]));
    // Should not panic
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(destroyed.0.is_empty());
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(despawns.0.is_empty());
}

// ── Behavior 9: Silently skip victims missing Position2D ────────────────

#[test]
fn handle_kill_silently_skips_victim_without_position() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((TestEntity, Hp::new(0.0), KilledBy::default()))
        .id();

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, None)]));
    // Should not panic
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(
        destroyed.0.is_empty(),
        "victim without Position2D: no Destroyed emitted"
    );
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        despawns.0.is_empty(),
        "victim without Position2D: no DespawnEntity emitted"
    );
    assert!(
        app.world().get::<Dead>(victim).is_none(),
        "victim without Position2D: Dead is not inserted"
    );
}

// ── Behavior 10: No messages pending — does nothing ─────────────────────

#[test]
fn handle_kill_does_nothing_when_no_messages_pending() {
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    // Explicitly empty pending resource.
    app.insert_resource(PendingKillRequests(vec![]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(destroyed.0.is_empty());
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(despawns.0.is_empty());
    assert!(
        app.world().get::<Dead>(victim).is_none(),
        "handle_kill must NOT insert Dead unless a KillYourself message arrives"
    );
}

#[test]
fn handle_kill_multiple_idle_ticks_leave_state_unchanged() {
    // Edge case: multiple ticks in a row with no messages.
    let mut app = build_handle_kill_app();
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![]));
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert!(destroyed.0.is_empty());
    assert!(app.world().get::<Dead>(victim).is_none());
}

// ── Behavior 11: Multiple distinct victims in one tick ──────────────────

#[test]
fn handle_kill_processes_multiple_distinct_victims() {
    let mut app = build_handle_kill_app();
    let v1 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(1.0, 2.0)),
        ))
        .id();
    let v2 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(3.0, 4.0)),
        ))
        .id();
    let v3 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(5.0, 6.0)),
        ))
        .id();

    app.insert_resource(PendingKillRequests(vec![
        kill_msg(v1, None),
        kill_msg(v2, None),
        kill_msg(v3, None),
    ]));
    tick(&mut app);

    // All three should have Dead inserted.
    assert!(app.world().get::<Dead>(v1).is_some());
    assert!(app.world().get::<Dead>(v2).is_some());
    assert!(app.world().get::<Dead>(v3).is_some());

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 3);
    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(despawns.0.len(), 3);

    // Each Destroyed.victim_pos matches its victim's Position2D.
    let by_victim: std::collections::HashMap<Entity, Vec2> = destroyed
        .0
        .iter()
        .map(|m| (m.victim, m.victim_pos))
        .collect();
    assert_eq!(by_victim.get(&v1), Some(&Vec2::new(1.0, 2.0)));
    assert_eq!(by_victim.get(&v2), Some(&Vec2::new(3.0, 4.0)));
    assert_eq!(by_victim.get(&v3), Some(&Vec2::new(5.0, 6.0)));
}

#[test]
fn handle_kill_three_victims_one_with_positioned_killer() {
    // Edge case: one of three has a positioned killer, two don't — each
    // Destroyed.killer_pos is correct for its own victim.
    let mut app = build_handle_kill_app();
    let v1 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(1.0, 2.0)),
        ))
        .id();
    let v2 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(3.0, 4.0)),
        ))
        .id();
    let v3 = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(5.0, 6.0)),
        ))
        .id();
    let killer = app
        .world_mut()
        .spawn(Position2D(Vec2::new(99.0, 88.0)))
        .id();

    app.insert_resource(PendingKillRequests(vec![
        kill_msg(v1, None),
        kill_msg(v2, Some(killer)),
        kill_msg(v3, None),
    ]));
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 3);

    let by_victim: std::collections::HashMap<Entity, Option<Vec2>> = destroyed
        .0
        .iter()
        .map(|m| (m.victim, m.killer_pos))
        .collect();
    assert_eq!(by_victim.get(&v1), Some(&None));
    assert_eq!(by_victim.get(&v2), Some(&Some(Vec2::new(99.0, 88.0))));
    assert_eq!(by_victim.get(&v3), Some(&None));
}

// ── Behavior 12: Self-kill (victim is its own killer) ───────────────────

#[test]
fn handle_kill_handles_self_kill() {
    let mut app = build_handle_kill_app();
    // Spawn with a placeholder KilledBy then update it to point at self,
    // since KilledBy { dealer: Some(victim) } requires the entity id.
    let victim = app
        .world_mut()
        .spawn((
            TestEntity,
            Hp::new(0.0),
            KilledBy::default(),
            Position2D(Vec2::new(10.0, 20.0)),
        ))
        .id();
    app.world_mut().entity_mut(victim).insert(KilledBy {
        dealer: Some(victim),
    });

    app.insert_resource(PendingKillRequests(vec![kill_msg(victim, Some(victim))]));
    // Should not panic (overlapping read-only queries on the same entity).
    tick(&mut app);

    let destroyed = app
        .world()
        .resource::<MessageCollector<Destroyed<TestEntity>>>();
    assert_eq!(destroyed.0.len(), 1);
    let msg = &destroyed.0[0];
    assert_eq!(msg.victim, victim);
    assert_eq!(msg.killer, Some(victim));
    assert_eq!(msg.victim_pos, Vec2::new(10.0, 20.0));
    assert_eq!(msg.killer_pos, Some(Vec2::new(10.0, 20.0)));

    assert!(app.world().get::<Dead>(victim).is_some());

    let despawns = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(despawns.0.len(), 1);
    assert_eq!(despawns.0[0].entity, victim);
}
