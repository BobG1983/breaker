//! Tests for the `Locked ↔ Invulnerable` coupling and the end-to-end lock
//! immunity contract. These tests assert observable state transitions on the
//! cell entity regardless of the coupling implementation (component hooks,
//! observer, or explicit system).

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    cells::{
        behaviors::locked::{
            components::Locked, systems::sync_lock_invulnerable::sync_lock_invulnerable,
        },
        components::Cell,
    },
    prelude::*,
    shared::death_pipeline::{
        damage_dealt::DamageDealt, invulnerable::Invulnerable, systems::apply_damage,
    },
};

fn coupling_test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, sync_lock_invulnerable)
        .build()
}

// ── L1: inserting Locked inserts Invulnerable ───────────────────────────────

#[test]
fn inserting_locked_on_a_cell_inserts_invulnerable() {
    let mut app = coupling_test_app();

    let cell = app
        .world_mut()
        .spawn((Cell, Hp::new(10.0), KilledBy::default()))
        .id();

    app.world_mut().entity_mut(cell).insert(Locked);

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(cell).is_some(),
        "Locked should have been inserted"
    );
    assert!(
        app.world().get::<Invulnerable>(cell).is_some(),
        "Invulnerable should have been inserted by the coupling system"
    );
}

/// L1 edge: spawning a cell with `Locked` at spawn time — `Invulnerable` is
/// present after one tick.
#[test]
fn spawning_cell_with_locked_at_spawn_inserts_invulnerable_after_tick() {
    let mut app = coupling_test_app();

    let cell = app
        .world_mut()
        .spawn((Cell, Hp::new(10.0), KilledBy::default(), Locked))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<Invulnerable>(cell).is_some(),
        "Invulnerable should be present after first tick for Added<Locked>"
    );
}

/// L1 edge: inserting `Locked` on a cell that already has `Invulnerable` is
/// idempotent — no panic, `Invulnerable` stays present.
#[test]
fn inserting_locked_when_invulnerable_already_present_is_idempotent() {
    let mut app = coupling_test_app();

    let cell = app
        .world_mut()
        .spawn((Cell, Hp::new(10.0), KilledBy::default(), Invulnerable))
        .id();

    app.world_mut().entity_mut(cell).insert(Locked);

    tick(&mut app);

    assert!(
        app.world().get::<Invulnerable>(cell).is_some(),
        "Invulnerable should remain when Locked is re-applied"
    );
}

// ── L2: removing Locked removes Invulnerable ────────────────────────────────

#[test]
fn removing_locked_from_a_cell_removes_invulnerable() {
    let mut app = coupling_test_app();

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(10.0),
            KilledBy::default(),
            Locked,
            Invulnerable,
        ))
        .id();

    app.world_mut().entity_mut(cell).remove::<Locked>();

    tick(&mut app);

    assert!(
        app.world().get::<Locked>(cell).is_none(),
        "Locked should be removed"
    );
    assert!(
        app.world().get::<Invulnerable>(cell).is_none(),
        "Invulnerable should be removed by the coupling system"
    );
}

/// L2 edge: removing `Invulnerable` directly (without removing `Locked`) — the
/// coupling system does NOT re-insert `Invulnerable` on the next tick.
#[test]
fn removing_invulnerable_directly_does_not_reinsert() {
    let mut app = coupling_test_app();

    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp::new(10.0),
            KilledBy::default(),
            Locked,
            Invulnerable,
        ))
        .id();

    app.world_mut().entity_mut(cell).remove::<Invulnerable>();
    tick(&mut app);

    assert!(
        app.world().get::<Locked>(cell).is_some(),
        "Locked should still be present"
    );
    assert!(
        app.world().get::<Invulnerable>(cell).is_none(),
        "coupling is unidirectional — removing Invulnerable should not reinsert it"
    );
}

// ── L3: end-to-end — locked cell absorbs damage; unlocking restores damage ──

#[derive(Resource, Default)]
struct PendingCellDamage(Vec<DamageDealt<Cell>>);

fn enqueue_cell_damage(
    pending: Res<PendingCellDamage>,
    mut writer: MessageWriter<DamageDealt<Cell>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[test]
fn locked_cell_absorbs_damage_and_unlocked_cell_takes_damage() {
    let mut app = TestAppBuilder::new()
        .with_message::<DamageDealt<Cell>>()
        .with_resource::<PendingCellDamage>()
        .with_system(FixedUpdate, sync_lock_invulnerable)
        .with_system(
            FixedUpdate,
            enqueue_cell_damage.before(apply_damage::<Cell>),
        )
        .with_system(FixedUpdate, apply_damage::<Cell>)
        .build();

    let cell = app
        .world_mut()
        .spawn((Cell, Hp::new(3.0), KilledBy::default(), Locked))
        .id();

    // Run one tick to flush the coupling — Invulnerable should be present.
    tick(&mut app);
    assert!(
        app.world().get::<Invulnerable>(cell).is_some(),
        "Invulnerable should be present after first tick"
    );

    // Phase A: enqueue damage — should be absorbed.
    app.insert_resource(PendingCellDamage(vec![DamageDealt::<Cell> {
        dealer:      None,
        target:      cell,
        amount:      5.0,
        source_chip: None,
        _marker:     PhantomData,
    }]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 3.0).abs() < f32::EPSILON,
        "Hp should be unchanged at 3.0 when cell is locked, got {}",
        hp.current
    );
    assert!(
        app.world().get::<Dead>(cell).is_none(),
        "Dead should not be present — apply_damage<Cell> never touched the cell"
    );
    assert!(
        app.world().get_entity(cell).is_ok(),
        "cell should still be alive"
    );

    // Phase B: unlock — tick to flush the coupling removal.
    app.world_mut().entity_mut(cell).remove::<Locked>();
    // Clear pending damage before the flush tick so no new damage is enqueued.
    app.insert_resource(PendingCellDamage(vec![]));
    tick(&mut app);
    assert!(
        app.world().get::<Invulnerable>(cell).is_none(),
        "Invulnerable should be removed alongside Locked"
    );

    // Phase B continued: enqueue the same damage — now it should apply.
    app.insert_resource(PendingCellDamage(vec![DamageDealt::<Cell> {
        dealer:      None,
        target:      cell,
        amount:      5.0,
        source_chip: None,
        _marker:     PhantomData,
    }]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - (-2.0)).abs() < f32::EPSILON,
        "Hp should be -2.0 (3.0 - 5.0 raw, not clamped), got {}",
        hp.current
    );
}
