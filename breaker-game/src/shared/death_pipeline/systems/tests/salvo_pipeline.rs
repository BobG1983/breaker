//! Integration tests for the Salvo monomorphization of the death pipeline.
//!
//! These tests verify that `apply_damage<Salvo>`, `detect_deaths<Salvo>`, and
//! `handle_kill<Salvo>` work correctly when monomorphized for the `Salvo` type.
//! The tests construct dedicated test apps rather than using the full
//! `DeathPipelinePlugin` in order to test each stage in isolation.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::{apply_damage, apply_heal, detect_deaths, handle_kill},
    helpers::TestEntity,
};
use crate::{
    cells::behaviors::survival::salvo::components::Salvo,
    shared::{
        death_pipeline::{
            damage_dealt::DamageDealt,
            dead::Dead,
            despawn_entity::DespawnEntity,
            destroyed::Destroyed,
            heal_dealt::{HealCap, HealDealt},
            hp::Hp,
            kill_yourself::KillYourself,
            killed_by::KilledBy,
        },
        test_utils::{MessageCollector, TestAppBuilder, tick},
    },
};

// ── Helpers ──────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct PendingSalvoDamage(Vec<DamageDealt<Salvo>>);

fn enqueue_salvo_damage(
    pending: Res<PendingSalvoDamage>,
    mut writer: MessageWriter<DamageDealt<Salvo>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

#[derive(Resource, Default)]
struct PendingSalvoKills(Vec<KillYourself<Salvo>>);

fn enqueue_salvo_kills(
    pending: Res<PendingSalvoKills>,
    mut writer: MessageWriter<KillYourself<Salvo>>,
) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

fn salvo_damage_msg(target: Entity, amount: f32, dealer: Option<Entity>) -> DamageDealt<Salvo> {
    DamageDealt {
        dealer,
        target,
        amount,
        source_chip: None,
        _marker: PhantomData,
    }
}

fn salvo_kill_msg(victim: Entity, killer: Option<Entity>) -> KillYourself<Salvo> {
    KillYourself {
        victim,
        killer,
        _marker: PhantomData,
    }
}

fn spawn_salvo_entity(app: &mut App, hp_value: f32) -> Entity {
    app.world_mut()
        .spawn((
            Salvo,
            Hp::new(hp_value),
            KilledBy::default(),
            Position2D(Vec2::ZERO),
        ))
        .id()
}

/// Builds an app with `apply_damage::<Salvo>` wired.
fn build_salvo_apply_damage_app() -> App {
    TestAppBuilder::new()
        .with_message::<DamageDealt<Salvo>>()
        .with_resource::<PendingSalvoDamage>()
        .with_system(
            FixedUpdate,
            enqueue_salvo_damage.before(apply_damage::<Salvo>),
        )
        .with_system(FixedUpdate, apply_damage::<Salvo>)
        .build()
}

/// Builds an app with `detect_deaths::<Salvo>` and captures `KillYourself<Salvo>`.
fn build_salvo_detect_deaths_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<KillYourself<Salvo>>()
        .with_system(FixedUpdate, detect_deaths::<Salvo>)
        .build()
}

/// Builds an app with `handle_kill::<Salvo>` and captures `Destroyed<Salvo>` + `DespawnEntity`.
fn build_salvo_handle_kill_app() -> App {
    TestAppBuilder::new()
        .with_message::<KillYourself<Salvo>>()
        .with_message_capture::<Destroyed<Salvo>>()
        .with_message_capture::<DespawnEntity>()
        .with_resource::<PendingSalvoKills>()
        .with_system(
            FixedUpdate,
            enqueue_salvo_kills.before(handle_kill::<Salvo>),
        )
        .with_system(FixedUpdate, handle_kill::<Salvo>)
        .build()
}

// ── Behavior 5 (partial): DamageDealt<Salvo> reduces Hp ──

#[test]
fn apply_damage_salvo_reduces_hp() {
    let mut app = build_salvo_apply_damage_app();
    let entity = spawn_salvo_entity(&mut app, 10.0);

    app.insert_resource(PendingSalvoDamage(vec![salvo_damage_msg(
        entity, 5.0, None,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Hp should be 5.0 after 5.0 damage to 10.0-HP salvo, got {}",
        hp.current
    );
}

#[test]
fn apply_damage_salvo_zero_amount_leaves_hp_unchanged() {
    let mut app = build_salvo_apply_damage_app();
    let entity = spawn_salvo_entity(&mut app, 10.0);

    app.insert_resource(PendingSalvoDamage(vec![salvo_damage_msg(
        entity, 0.0, None,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "Hp should remain 10.0 after 0.0 damage, got {}",
        hp.current
    );
}

// ── Behavior 9: detect_deaths<Salvo> emits KillYourself<Salvo> at zero Hp ──

#[test]
fn detect_deaths_salvo_emits_kill_yourself_at_zero_hp() {
    let mut app = build_salvo_detect_deaths_app();
    let entity = app
        .world_mut()
        .spawn((
            Salvo,
            Hp {
                current:  0.0,
                starting: 1.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<Salvo>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "detect_deaths<Salvo> should emit KillYourself when Hp is 0.0"
    );
    assert_eq!(collector.0[0].victim, entity);
}

#[test]
fn detect_deaths_salvo_does_not_emit_above_zero_hp() {
    let mut app = build_salvo_detect_deaths_app();
    let _entity = app
        .world_mut()
        .spawn((
            Salvo,
            Hp {
                current:  1.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy::default(),
        ))
        .id();

    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<KillYourself<Salvo>>>();
    assert!(
        collector.0.is_empty(),
        "detect_deaths<Salvo> should NOT emit KillYourself when Hp is above 0.0"
    );
}

// ── Behavior 10: handle_kill<Salvo> marks Dead and emits Destroyed<Salvo> ──

#[test]
fn handle_kill_salvo_marks_dead_and_emits_destroyed() {
    let mut app = build_salvo_handle_kill_app();
    let entity = spawn_salvo_entity(&mut app, 1.0);
    let killer = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingSalvoKills(vec![salvo_kill_msg(
        entity,
        Some(killer),
    )]));
    tick(&mut app);

    // Entity should be marked Dead
    let dead = app.world().get::<Dead>(entity);
    assert!(
        dead.is_some(),
        "handle_kill<Salvo> should insert Dead component"
    );

    // Destroyed<Salvo> should be emitted
    let collector = app.world().resource::<MessageCollector<Destroyed<Salvo>>>();
    assert_eq!(
        collector.0.len(),
        1,
        "handle_kill<Salvo> should emit exactly 1 Destroyed<Salvo>"
    );
    assert_eq!(collector.0[0].victim, entity);
    assert_eq!(collector.0[0].killer, Some(killer));

    // DespawnEntity should be emitted
    let despawn_collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert_eq!(
        despawn_collector.0.len(),
        1,
        "handle_kill<Salvo> should emit DespawnEntity"
    );
}

#[test]
fn handle_kill_salvo_skips_already_dead_entity() {
    let mut app = build_salvo_handle_kill_app();
    let entity = spawn_salvo_entity(&mut app, 1.0);

    // Pre-insert Dead to simulate already-dead entity
    app.world_mut().entity_mut(entity).insert(Dead);

    app.insert_resource(PendingSalvoKills(vec![salvo_kill_msg(entity, None)]));
    tick(&mut app);

    // Destroyed<Salvo> should NOT be emitted (entity was already Dead)
    let collector = app.world().resource::<MessageCollector<Destroyed<Salvo>>>();
    assert!(
        collector.0.is_empty(),
        "handle_kill<Salvo> should NOT emit Destroyed for an already-Dead entity"
    );
}

// ── Behavior 30: apply_heal::<Salvo> heals Salvo entities ──

#[derive(Resource, Default)]
struct PendingSalvoHeal(Vec<HealDealt<Salvo>>);

fn enqueue_salvo_heal(pending: Res<PendingSalvoHeal>, mut writer: MessageWriter<HealDealt<Salvo>>) {
    for msg in &pending.0 {
        writer.write(msg.clone());
    }
}

fn salvo_heal_msg(target: Entity, amount: f32, cap: HealCap) -> HealDealt<Salvo> {
    HealDealt {
        healer: None,
        target,
        amount,
        source: None,
        cap,
        _marker: PhantomData,
    }
}

/// Builds an app with `apply_heal::<Salvo>` wired.
fn build_salvo_apply_heal_app() -> App {
    TestAppBuilder::new()
        .with_message::<HealDealt<Salvo>>()
        .with_resource::<PendingSalvoHeal>()
        .with_system(FixedUpdate, enqueue_salvo_heal.before(apply_heal::<Salvo>))
        .with_system(FixedUpdate, apply_heal::<Salvo>)
        .build()
}

#[test]
fn apply_heal_salvo_heals_hp() {
    let mut app = build_salvo_apply_heal_app();
    let entity = spawn_salvo_entity(&mut app, 10.0);
    // Pre-damage salvo to 2.0 (spawn_salvo_entity creates current=starting=10.0).
    app.world_mut().get_mut::<Hp>(entity).unwrap().current = 2.0;

    app.insert_resource(PendingSalvoHeal(vec![salvo_heal_msg(
        entity,
        3.0,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Salvo Hp should be 5.0 after 3.0 heal on 2.0 current, got {}",
        hp.current
    );
}

#[test]
fn apply_heal_salvo_heals_hp_starting_cap_matches_max_when_max_none() {
    let mut app = build_salvo_apply_heal_app();
    let entity = spawn_salvo_entity(&mut app, 10.0);
    // Pre-damage salvo to 2.0.
    app.world_mut().get_mut::<Hp>(entity).unwrap().current = 2.0;

    app.insert_resource(PendingSalvoHeal(vec![salvo_heal_msg(
        entity,
        3.0,
        HealCap::Starting,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "Salvo Hp should be 5.0 with Starting cap (max=None round-trip), got {}",
        hp.current
    );
}

// ── Behavior 20: Heal of the wrong monomorphization is silently skipped ──
//
// Canonical per-T queue isolation proof from the Salvo side: a `TestEntity`
// (NOT `Salvo`) receives a `HealDealt<Salvo>` message in a Salvo-only harness.
// `apply_heal::<Salvo>`'s `With<Salvo>` filter must exclude the TestEntity, so
// Hp stays unchanged.
//
// This harness deliberately does NOT register `TestEntity` systems or messages
// — it reuses `build_salvo_apply_heal_app` exactly as the Salvo-side tests do.

#[test]
fn apply_heal_salvo_skips_test_entity_target_wrong_monomorphization() {
    let mut app = build_salvo_apply_heal_app();

    // Spawn a TestEntity (NOT Salvo) with Hp(5.0, 10.0): starting 10.0, current 5.0.
    let entity = app
        .world_mut()
        .spawn((TestEntity, Hp::new(10.0), KilledBy::default()))
        .id();
    app.world_mut().get_mut::<Hp>(entity).unwrap().current = 5.0;

    // Enqueue a HealDealt<Salvo> targeting the TestEntity (wrong monomorphization).
    app.insert_resource(PendingSalvoHeal(vec![salvo_heal_msg(
        entity,
        3.0,
        HealCap::Max,
    )]));
    tick(&mut app);

    let hp = app.world().get::<Hp>(entity).unwrap();
    assert!(
        (hp.current - 5.0).abs() < f32::EPSILON,
        "TestEntity Hp should remain 5.0 — apply_heal::<Salvo>'s With<Salvo> \
         filter must exclude entities without the Salvo marker, got {}",
        hp.current
    );
}
