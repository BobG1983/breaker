//! W5 — whitelist regression pin for `echo_strike_emit_siblings`.
//!
//! `echo_strike_emit_siblings` is a ripple emitter that fires in
//! `DmgSystems::PostApplyDamage` after a primary `DamageDealt<Cell>` applies.
//! It must NEVER be moved into `DmgSystems::EmitDamage`.
//!
//! W8 §F — adds an end-to-end Hp-delta pin: a primed bolt with a 2-echo
//! `EchoNetwork` that takes a primary `DamageDealt<Cell>` produces echo
//! sibling damage that decrements the echo cells' `Hp` after pipeline
//! re-application. Pairs the message-emission pin in `emit_siblings.rs`
//! with the resulting `Hp` change so a future regression that breaks the
//! sibling amount-pass-through is caught here.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::{
    super::system::{EchoNetwork, EchoPrimed, echo_strike_emit_siblings, wire},
    helpers::{canonical_echo_strike_config, seed_active_protocols_with_echo_strike},
};
use crate::{
    mutators::{
        hazards::resources::ActiveHazards, plugin::wire_damage_chain,
        protocols::resources::ActiveProtocols,
    },
    prelude::*,
};

fn echo_strike_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<ActiveHazards>()
        .with_effects_pipeline()
        .build();
    wire(&mut app);
    wire_damage_chain(&mut app);
    app
}

#[test]
fn echo_strike_emit_siblings_is_pinned_to_post_apply_damage() {
    let mut app = echo_strike_scheduling_app();

    assert!(
        system_in_set(
            &mut app,
            FixedUpdate,
            echo_strike_emit_siblings,
            DmgSystems::PostApplyDamage,
        ),
        "echo_strike_emit_siblings must remain in DmgSystems::PostApplyDamage."
    );
}

#[test]
fn echo_strike_emit_siblings_is_not_in_emit_damage() {
    let mut app = echo_strike_scheduling_app();

    assert!(
        !system_in_set(
            &mut app,
            FixedUpdate,
            echo_strike_emit_siblings,
            DmgSystems::EmitDamage,
        ),
        "echo_strike_emit_siblings must NOT be a member of DmgSystems::EmitDamage."
    );
}

// ── W8 §F — end-to-end Hp-delta pin ─────────────────────────────────────────

/// Spawns a vulnerable (non-`Invulnerable`) cell with `Cell` + `Hp`. The
/// `apply_damage::<Cell>` query needs only `With<Cell>, Without<Dead>` plus
/// a mutable `Hp`, so this minimal shape is sufficient for end-to-end Hp
/// assertions. `_pos` is accepted to mirror the call-site signature in the
/// W8 §F plan but is unused by the dmg pipeline (no spatial query
/// involved).
fn spawn_vuln_cell_with_hp(app: &mut App, _pos: Vec2, hp: f32) -> Entity {
    app.world_mut().spawn((Cell, Hp::new(hp))).id()
}

/// W8 §F — pairs the message-emission pin in `emit_siblings.rs` with an
/// end-to-end `Hp` assertion. Setup: a primed bolt with a 2-echo network
/// receives a primary `DamageDealt<Cell>` of `amount = 20.0` against a
/// 100-Hp primary cell. Tick 1 applies the primary (`primary.Hp = 80.0`)
/// and emits echo siblings via `echo_strike_emit_siblings` in
/// `DmgSystems::PostApplyDamage`. The siblings re-enter the pipeline on
/// tick 2 and `apply_damage::<Cell>` decrements each echo's `Hp`.
///
/// With canonical `EchoStrikeConfig`
/// (`newest_fraction: 0.5, oldest_fraction: 0.1`) and a 2-entry network,
/// the fractions slice is `[oldest, newest] = [0.1, 0.5]`:
/// - echo at deque-front (oldest) → `20.0 × 0.1 = 2.0` damage → `Hp = 98.0`
/// - echo at deque-back  (newest) → `20.0 × 0.5 = 10.0` damage → `Hp = 90.0`
#[test]
fn echo_strike_sibling_applies_expected_hp_delta_end_to_end() {
    let mut app = echo_strike_scheduling_app();
    let cfg = canonical_echo_strike_config();
    app.world_mut().insert_resource(cfg);
    seed_active_protocols_with_echo_strike(
        &mut app,
        cfg.max_echoes,
        cfg.newest_fraction,
        cfg.middle_fraction,
        cfg.oldest_fraction,
    );

    // Primary cell — receives the primary `DamageDealt<Cell>` of amount 20.
    let primary = spawn_vuln_cell_with_hp(&mut app, Vec2::ZERO, 100.0);
    // echo_oldest sits at the deque front (oldest entry).
    let echo_oldest = spawn_vuln_cell_with_hp(&mut app, Vec2::new(60.0, 0.0), 100.0);
    // echo_newest sits at the deque back  (newest entry).
    let echo_newest = spawn_vuln_cell_with_hp(&mut app, Vec2::new(-60.0, 0.0), 100.0);

    // Bolt: primed, 2-echo network. `EchoNetwork.echoes` is a VecDeque —
    // `from_iter([oldest, newest])` keeps oldest at the front, newest at
    // the back, matching the `len() == 2 → [oldest_fraction,
    // newest_fraction]` mapping in `echo_strike_emit_siblings`.
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            EchoPrimed,
            EchoNetwork {
                echoes: [echo_oldest, echo_newest].into_iter().collect(),
            },
        ))
        .id();

    // Write the primary `DamageDealt<Cell>` directly. `apply_damage::<Cell>`
    // applies the 20.0 to `primary` in tick 1; `echo_strike_emit_siblings`
    // runs in `DmgSystems::PostApplyDamage` in the same tick and emits
    // the two sibling messages.
    app.world_mut()
        .resource_mut::<Messages<DamageDealt<Cell>>>()
        .write(DamageDealt::<Cell> {
            dealer:        Some(bolt),
            attributed_to: None,
            target:        primary,
            amount:        20.0,
            source:        None,
            _marker:       PhantomData,
        });

    // Tick 1: primary applied; echo siblings emitted (post-apply).
    tick(&mut app);
    assert!(
        (app.world()
            .get::<Hp>(primary)
            .expect("cell should still have Hp")
            .current
            - 80.0)
            .abs()
            < 1e-5,
        "tick 1: primary.Hp = 100.0 − 20.0 == 80.0"
    );

    // Tick 2: echo sibling messages traverse the pipeline; `apply_damage`
    // decrements each echo cell's `Hp`.
    tick(&mut app);

    let hp_oldest = app
        .world()
        .get::<Hp>(echo_oldest)
        .expect("cell should still have Hp")
        .current;
    let hp_newest = app
        .world()
        .get::<Hp>(echo_newest)
        .expect("cell should still have Hp")
        .current;
    let hp_primary = app
        .world()
        .get::<Hp>(primary)
        .expect("cell should still have Hp")
        .current;

    // primary unchanged after tick 2 (no further damage targets it).
    assert!(
        (hp_primary - 80.0).abs() < 1e-5,
        "primary.Hp must remain 80.0 after tick 2, got {hp_primary}"
    );
    // echo_oldest: 100.0 − (20.0 × 0.1) == 98.0
    assert!(
        (hp_oldest - 98.0).abs() < 1e-5,
        "echo_oldest.Hp = 100.0 − (20.0 × oldest_fraction[0.1]) == 98.0, got {hp_oldest}"
    );
    // echo_newest: 100.0 − (20.0 × 0.5) == 90.0
    assert!(
        (hp_newest - 90.0).abs() < 1e-5,
        "echo_newest.Hp = 100.0 − (20.0 × newest_fraction[0.5]) == 90.0, got {hp_newest}"
    );
}
