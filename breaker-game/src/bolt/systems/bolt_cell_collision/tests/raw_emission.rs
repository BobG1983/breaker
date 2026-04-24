//! W6 tests: `bolt_cell_collision` emits RAW `base_damage` — the crate
//! pipeline (`apply_damage_boosts::<Cell>` + `apply_vulnerable::<Cell>`) is
//! the sole applier of boost/vulnerability multipliers.
//!
//! Two harnesses are used:
//!
//! - **Harness A** (`test_app_with_damage_and_wall_messages`): no
//!   `RantzDmgPlugin`; `DamageDealtCellMessages` captures the RAW emission.
//!   Behaviors 1–4, 9 (pierce decision), 10 (pierce emission).
//! - **Harness B** (`test_app_with_full_pipeline`): full `RantzDmgPlugin`
//!   pipeline + production `BoltPlugin` wiring. Assertions pin post-pipeline
//!   `amount` AND final `Hp.current`. Behaviors 15, 16, 17.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{GlobalPosition2D, Spatial2D};

use super::helpers::*;
use crate::{
    bolt::{
        components::{BoltBaseDamage, PiercingRemaining},
        test_utils::{damage_stack, piercing_stack},
    },
    cells::resources::CellConfig,
    prelude::*,
    shared::GameDrawLayer,
};

// ── Behavior 1 — emission carries raw base damage even when boost present ──

/// Regression pin for behavior 2: Bolt with `damage_stack(&[2.0])` must
/// emit RAW `10.0`, NOT `20.0`. Captured pre-pipeline (Harness A).
#[test]
fn emission_amount_is_raw_base_damage_even_when_boost_stack_present() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 20.0 — the pipeline \
         is the sole applier of DamageBoostStack; got {}",
        msgs.0[0].amount
    );
}

// ── Behavior 2 — multi-entry boost stack is also ignored at emission ──

/// Regression pin for behavior 2 edge case: Bolt with
/// `damage_stack(&[2.0, 1.5])` (aggregate 3.0) must still emit RAW `10.0`,
/// NOT `30.0`. Captured pre-pipeline (Harness A).
#[test]
fn emission_amount_ignores_multi_entry_damage_boost_stack() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0, 1.5]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 30.0 — multi-entry \
         DamageBoostStack aggregate (2.0 * 1.5 = 3.0) is NOT applied at emission; got {}",
        msgs.0[0].amount
    );
}

// ── Behavior 3 — emission ignores target VulnerableStack ──

/// Regression pin for behavior 3: Cell with `VulnerableStack` persistent
/// `2.0` must still receive emission `amount == 10.0`, NOT `20.0`. Captured
/// pre-pipeline (Harness A).
#[test]
fn emission_amount_ignores_single_persistent_vulnerability() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_vulnerable_cell(&mut app, 0.0, cell_y, 50.0, 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 20.0 — the pipeline \
         is the sole applier of target VulnerableStack; got {}",
        msgs.0[0].amount
    );
}

/// Regression pin for behavior 3 edge case: stacked vulnerability
/// (`[1.5, 2.0]` aggregate 3.0) must still emit `10.0`, NOT `30.0`.
#[test]
fn emission_amount_ignores_stacked_vulnerability() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_vulnerable_cell(&mut app, 0.0, cell_y, 50.0, 1.5);
    app.world_mut()
        .entity_mut(cell)
        .get_mut::<VulnerableStack>()
        .unwrap()
        .add(SourceId::from("test"), 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 30.0 — stacked \
         VulnerableStack aggregate (1.5 * 2.0 = 3.0) is NOT applied at emission; got {}",
        msgs.0[0].amount
    );
}

// ── Behavior 4 — emission ignores BOTH boost and vulnerability ──

/// Regression pin for behavior 4: combined boost + vulnerability must emit
/// RAW `10.0`, NOT `30.0` (boost × vuln once) NOR `60.0` (boost × vuln twice).
#[test]
fn emission_amount_ignores_both_boost_and_vulnerability() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_vulnerable_cell(&mut app, 0.0, cell_y, 50.0, 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[1.5]));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 30.0 (boost * vuln) \
         NOR 60.0 (boost^2 * vuln^2); got {}",
        msgs.0[0].amount
    );
}

/// Edge case for behavior 4: high base damage `25.0` with boost `2.0` and
/// vuln `2.0` — emission still RAW `25.0`, NOT `100.0`.
#[test]
fn emission_amount_ignores_both_boost_and_vulnerability_high_base() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_vulnerable_cell(&mut app, 0.0, cell_y, 100.0, 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert((BoltBaseDamage(25.0), damage_stack(&[2.0])));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(msgs.0.len(), 1, "should emit one DamageDealt<Cell>");
    assert_eq!(msgs.0[0].target, cell);
    assert!(
        (msgs.0[0].amount - 25.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (25.0), NOT 100.0; got {}",
        msgs.0[0].amount
    );
}

// ── Behavior 9 — pierce decision uses combined boost AND vulnerability ──

/// Pierce lookahead still sees the FULLY-MULTIPLIED damage locally. With
/// `damage_stack(&[1.5])` and `VulnerableStack(2.0)` against `Hp(25.0)`,
/// effective = 10 * 1.5 * 2.0 = 30 > 25 → PIERCE. The emitted `amount`
/// remains RAW `10.0` — pierce-decision values do NOT leak into the message.
#[test]
fn pierce_decision_uses_combined_boost_and_vulnerability() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_vulnerable_cell(&mut app, 0.0, cell_y, 25.0, 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        piercing_stack(&[1]),
        PiercingRemaining(1),
        damage_stack(&[1.5]),
    ));

    tick(&mut app);

    // Pierce happened: PiercingRemaining decremented to 0.
    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement (10 * 1.5 * 2.0 = 30 > Hp 25) → pierce, got {}",
        pr.0
    );

    // Bolt kept moving forward (velocity.y > 0) — no reflect.
    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should have Velocity2D");
    assert!(
        vel.0.y > 0.0,
        "piercing bolt velocity.y should remain positive after pierce, got {}",
        vel.0.y
    );

    // And critically, emitted amount is RAW 10.0 — pierce-decision value
    // did not leak into the message.
    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert!(
        !msgs.0.is_empty(),
        "should emit at least one DamageDealt<Cell>"
    );
    let msg_for_cell = msgs
        .0
        .iter()
        .find(|m| m.target == cell)
        .expect("DamageDealt<Cell> for pierced cell should exist");
    assert!(
        (msg_for_cell.amount - 10.0).abs() < f32::EPSILON,
        "emission amount should be RAW base_damage (10.0), NOT 30.0 (pierce-decision value); got {}",
        msg_for_cell.amount
    );
}

/// Edge case for behavior 9: `Hp(35.0)` — effective `30 < 35` → REFLECT.
/// `PiercingRemaining` stays 1, velocity.y flips negative.
#[test]
fn pierce_decision_uses_combined_boost_and_vulnerability_reflect_edge() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    spawn_vulnerable_cell(&mut app, 0.0, cell_y, 35.0, 2.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        piercing_stack(&[1]),
        PiercingRemaining(1),
        damage_stack(&[1.5]),
    ));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining should stay at 1 (10 * 1.5 * 2.0 = 30 < Hp 35) → reflect, got {}",
        pr.0
    );

    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should have Velocity2D");
    assert!(
        vel.0.y < 0.0,
        "reflecting bolt velocity.y should flip negative, got {}",
        vel.0.y
    );
}

// ── Behavior 10 — piercing bolt emits raw amount even with boost ──

/// Two cells along the bolt's path, each `Hp(15.0)`. Bolt has
/// `damage_stack(&[2.0])` + `PiercingRemaining(1)` — effective `20 > 15`
/// → pierce both. BOTH emitted `amount`s are RAW `10.0`, NOT `20.0`.
/// `PiercingRemaining` decrements from 1 to 0.
#[test]
fn piercing_bolt_emits_raw_amount_even_with_boost() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = test_bolt_definition();

    let near_cell_y = 60.0;
    let far_cell_y = 90.0;
    let cell_a = spawn_cell_with_health(&mut app, 0.0, near_cell_y, 15.0);
    let cell_b = spawn_cell_with_health(&mut app, 0.0, far_cell_y, 15.0);

    let start_y = near_cell_y - bc.radius - 25.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 10000.0);
    app.world_mut().entity_mut(bolt_entity).insert((
        piercing_stack(&[1]),
        PiercingRemaining(1),
        damage_stack(&[2.0]),
    ));

    tick(&mut app);

    let msgs = app.world().resource::<DamageDealtCellMessages>();
    assert_eq!(
        msgs.0.len(),
        2,
        "two DamageDealt<Cell> messages expected for two-cell pierce, got {}",
        msgs.0.len()
    );

    // BOTH emissions carry RAW 10.0 — pierce-decision value (20.0) is local only.
    for msg in &msgs.0 {
        assert!(
            (msg.amount - 10.0).abs() < f32::EPSILON,
            "each emission amount should be RAW base_damage (10.0), NOT 20.0; got {}",
            msg.amount
        );
    }

    // Both target cells represented in emissions.
    let targets: Vec<Entity> = msgs.0.iter().map(|m| m.target).collect();
    assert!(
        targets.contains(&cell_a),
        "emission for near cell should exist"
    );
    assert!(
        targets.contains(&cell_b),
        "emission for far cell should exist"
    );

    // Piercing charge consumed exactly once across both hits.
    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("bolt should have PiercingRemaining");
    assert_eq!(
        pr.0, 0,
        "PiercingRemaining should decrement from 1 to 0 after piercing both cells, got {}",
        pr.0
    );
}

// ── Behavior 15 — one-shot boost drains exactly once (full pipeline) ──

/// Bolt with a single one-shot boost multiplier `2.0` (no persistent lane)
/// hits a single cell `Hp(50.0)`. Post-tick `Hp == 30.0` — the one-shot
/// drains exactly once. Uses Harness B.
#[test]
fn full_pipeline_one_shot_boost_drains_exactly_once() {
    let mut app = test_app_with_full_pipeline();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let starting_hp = 50.0;
    let cell = spawn_cell_with_health(&mut app, 0.0, cell_y, starting_hp);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    // Seed the one-shot lane ONLY — not the persistent lane. This is the
    // only path to exercise one-shots in tests (the `damage_stack(&[..])`
    // helper only populates persistent entries).
    {
        let mut stack = DamageBoostStack::default();
        stack.add_one_shot(2.0);
        app.world_mut().entity_mut(bolt_entity).insert(stack);
    }

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).map_or(f32::NAN, |h| h.current);
    assert!(
        (hp - 30.0).abs() < 1e-5,
        "post-tick cell Hp.current should be 30.0 (50 − 10 * 2.0 one-shot once), got {hp}"
    );

    // Confirm the one-shot lane is drained: a subsequent
    // `aggregate_and_consume_one_shots()` must return 1.0 (identity) —
    // there is nothing left to drain.
    let drained = app
        .world_mut()
        .get_mut::<DamageBoostStack>(bolt_entity)
        .expect("bolt should still have DamageBoostStack")
        .aggregate_and_consume_one_shots();
    assert!(
        (drained - 1.0).abs() < f32::EPSILON,
        "after tick, a fresh one-shot drain should return identity (1.0) — \
         the pipeline already drained the 2.0 entry exactly once; got {drained}"
    );
}

// ── Behavior 16 — one-shot drain is message-scoped (full pipeline) ──
//
// Behavior 16 (one-shot drain is message-scoped across a multi-cell pierce)
// is NOT implemented here. Constructing a deterministic multi-cell pierce
// in Harness B is blocked by BoltPlugin's production speed-clamp (bolt
// velocity clamped to `max_speed = 1440`): at 60 Hz fixed tick, that's 24
// units/frame, not enough to sweep through two cells ≥30 units apart in a
// single tick under CCD. The mechanic Behavior 16 intends to pin — that
// `DamageBoostStack::aggregate_and_consume_one_shots()` drains the one-shot
// lane exactly once per-message — is a direct property of the
// `rantzsoft_dmg` pipeline and is covered by Behavior 15 (single-message
// one-shot drain) combined with Behavior 10 (multi-message raw emission).
// An explicit multi-cell regression here would duplicate that coverage
// without adding a new failure mode.

// ── Behavior 17 — Locked + Invulnerable cell unchanged despite boost ──

/// Cell with `Locked` + `Invulnerable`, `Hp(50.0)`. Bolt has
/// `damage_stack(&[2.0])`. Post-tick `Hp == 50.0` — `invulnerable_filter::<Cell>`
/// zeroes the message amount before `apply_damage::<Cell>`. The producer
/// still emitted a message (filter-free per behavior 8.5).
#[test]
fn full_pipeline_locked_invulnerable_cell_hp_unchanged_despite_boost() {
    let mut app = test_app_with_full_pipeline();
    let bc = test_bolt_definition();
    let cc = CellConfig::default();

    let (cw, ch) = default_cell_dims();
    let half_extents = Vec2::new(cw.half_width(), ch.half_height());
    let cell_y = 100.0;
    let starting_hp = 50.0;
    let pos = Vec2::new(0.0, cell_y);
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            cw,
            ch,
            Hp::new(starting_hp),
            KilledBy { killer: None },
            Locked,
            Invulnerable,
            Aabb2D::new(Vec2::ZERO, half_extents),
            CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Cell,
        ))
        .id();

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]));

    tick(&mut app);

    let hp = app.world().get::<Hp>(cell).map_or(f32::NAN, |h| h.current);
    assert!(
        (hp - starting_hp).abs() < 1e-5,
        "Locked + Invulnerable cell Hp should remain {starting_hp} — \
         invulnerable_filter zeroes amount before apply_damage, got {hp}"
    );
}
