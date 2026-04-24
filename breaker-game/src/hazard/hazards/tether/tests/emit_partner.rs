//! W2 Behaviors 32–36: `tether_emit_partner` in `DmgSystems::PostApplyDamage`.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::{TetherConfig, tether_emit_partner},
    helpers::{
        add_tether_stacks, canonical_tether_config, install_tether_config, spawn_cell_at,
        spawn_linked_pair,
    },
};
use crate::{
    hazard::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

#[track_caller]
fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

/// Build a Tether test app wired for the new `PostApplyDamage` system.
pub(super) fn build_tether_app(active: bool) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_effects_pipeline()
        .with_resource::<ActiveHazards>()
        .build();
    install_tether_config(&mut app, canonical_tether_config());
    if active {
        add_tether_stacks(&mut app, 1);
    }
    app.add_systems(
        FixedUpdate,
        tether_emit_partner
            .in_set(DmgSystems::PostApplyDamage)
            .run_if(hazard_active(HazardKind::Tether))
            .run_if(in_state(NodeState::Playing)),
    );
    app
}

// ── W2 Behavior 32: tether_emit_partner inactive gate ──

#[test]
fn tether_emit_partner_inactive_emits_nothing() {
    let mut app = build_tether_app(false);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let _ = (a, b);
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    // Drain and confirm no tether-sourced sibling was emitted.
    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("hazard:tether")))
    );
}

// ── W2 Behavior 33: partner sibling is msg.amount * damage_pct / 100 ──

#[test]
fn tether_emit_partner_emits_sibling_at_damage_pct() {
    // damage_percent(1) = 25.0 with canonical config.
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let sibling = drained
        .iter()
        .find(|m| m.source == Some(SourceId::from("hazard:tether")))
        .expect("exactly one tether sibling expected");
    assert_eq!(sibling.target, b);
    assert_f32_eq(sibling.amount, 25.0);
    assert_eq!(sibling.dealer, None);
    assert_eq!(sibling.attributed_to, Some(bolt));
}

#[test]
fn tether_emit_partner_with_higher_damage_pct() {
    // damage_percent set to 30.0 via custom config.
    let mut app = build_tether_app(true);
    // Override config.
    install_tether_config(
        &mut app,
        TetherConfig {
            base_damage:        30.0,
            damage_per_level:   0.0,
            base_coverage:      40.0,
            coverage_per_level: 0.0,
        },
    );
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let sibling = drained
        .iter()
        .find(|m| m.source == Some(SourceId::from("hazard:tether")))
        .expect("sibling expected");
    assert_eq!(sibling.target, b);
    assert_f32_eq(sibling.amount, 30.0);
}

// ── W2 Behavior 35: tether skips emission when msg.amount == 0 (invulnerable) ──

#[test]
fn tether_emit_partner_skips_when_amount_zero() {
    // msg.amount is 0.0 at PostApplyDamage — invulnerable_filter zeroed it.
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let _ = b;
    let bolt = app.world_mut().spawn_empty().id();

    // Simulate an invulnerable source by writing a zero-amount message.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        0.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("hazard:tether"))),
        "no partner sibling when msg.amount == 0"
    );
}

#[test]
fn tether_no_link_no_emission() {
    // A has no TetherLink → no partner message emitted.
    let mut app = build_tether_app(true);
    let a = spawn_cell_at(&mut app, Vec2::ZERO);
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    assert!(
        !drained
            .iter()
            .any(|m| m.source == Some(SourceId::from("hazard:tether")))
    );
}

// ── W2 Behavior 36: loop protection — don't re-emit tether on tether source ──

#[test]
fn tether_loop_protection_skips_on_tether_source() {
    // Primary already carries "hazard:tether" source → no further partner emitted.
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let _ = b;
    let bolt = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        None,
            attributed_to: Some(bolt),
            target:        a,
            amount:        25.0,
            source:        Some(SourceId::from("hazard:tether")),
            _marker:       PhantomData,
        });

    tick(&mut app);

    let drained: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    // Only the original tether-sourced message should appear; no new
    // tether-sourced sibling.
    let tethered: Vec<_> = drained
        .iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")))
        .collect();
    assert_eq!(tethered.len(), 1, "only the original msg — no re-emission");
    assert_eq!(tethered[0].target, a);
}

// ── Cursor-advance regression: `tether_emit_partner` uses a `Local<MessageCursor>`
//    to avoid re-emitting on a primary that was already observed. A second
//    tick with no new primary must NOT produce a second partner sibling. ──

#[test]
fn tether_cursor_does_not_re_emit_on_second_tick_without_new_primary() {
    let mut app = build_tether_app(true);
    let (a, b) = spawn_linked_pair(&mut app, Vec2::ZERO, Vec2::new(30.0, 0.0));
    let bolt = app.world_mut().spawn_empty().id();

    // Tick N: write one primary, observe exactly one tether sibling emitted.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        a,
            amount:        100.0,
            source:        None,
            _marker:       PhantomData,
        });
    tick(&mut app);

    let after_tick_n: Vec<DamageDealt<Cell>> = app
        .world()
        .resource::<Messages<DamageDealt<Cell>>>()
        .iter_current_update_messages()
        .cloned()
        .collect();
    let siblings_n: Vec<_> = after_tick_n
        .iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")))
        .collect();
    assert_eq!(
        siblings_n.len(),
        1,
        "tick N must emit exactly one tether sibling"
    );
    assert_eq!(siblings_n[0].target, b);

    // Tick N+1: no new primary written. Messages<DamageDealt<Cell>> retains
    // the prior frame's messages until update(), but the cursor must NOT
    // re-read them and emit another sibling.
    tick(&mut app);

    let after_tick_n_plus_1: Vec<DamageDealt<Cell>> = app
        .world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .drain()
        .collect();
    let tether_count = after_tick_n_plus_1
        .iter()
        .filter(|m| m.source == Some(SourceId::from("hazard:tether")))
        .count();
    assert_eq!(
        tether_count, 1,
        "only one tether sibling across both ticks — cursor must not re-emit"
    );
}
