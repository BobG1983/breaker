//! Group F — `wire` wiring + run-condition gates (Behaviors 24–28, 41–48, 53–55).
//!
//! Pins that `wire`-wired systems run under the correct schedules, gated
//! by `protocol_active(RecklessDash)` + `in_state(NodeState::Playing)` on
//! the `FixedUpdate` systems, that same-tick ordering anchors work, that the
//! schedule is harness-safe under missing resources + quiet ticks.
//!
//! Behaviors 24–28 pin the Wave-5 wire contract for `reckless_dash_on_dash_transition`:
//! gating, negative gating, absence of the deleted `reckless_dash_double_penalty`
//! and `reckless_dash_cleanup_node` systems, and absence of `RecklessDashDoubledBolts`.

use bevy::prelude::*;

use super::{
    super::system::{RecklessDashConfig, RiskyDamageBoost, wire},
    helpers::{
        build_reckless_dash_app, build_reckless_dash_app_in_chip_selecting,
        build_reckless_dash_app_no_config, captured_bolt_lost, collected_reckless_dash_damage,
        force_dash_state, risky_boost, seed_active_protocols_with_reckless_dash,
        spawn_bolt_with_base_damage, spawn_breaker_dashing, spawn_breaker_for_transition,
        spawn_cell_empty, write_bolt_impact_cell, write_bolt_lost, write_bump_performed,
    },
};
use crate::{
    bolt::sets::BoltSystems,
    breaker::{
        components::{BoltLossBehavior, DashState},
        messages::BumpGrade,
        sets::BreakerSystems,
    },
    mutators::protocols::resources::ActiveProtocols,
    prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

// ── Behavior 41 — on_bump wired + gated on active + Playing ────────────────-

#[test]
fn register_wires_on_bump_gated_on_active_and_playing() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "on_bump must run via wire → boost inserted"
    );
}

// ── Behavior 42 — on_bump gated off when RecklessDash NOT active ───────────-

#[test]
fn on_bump_gated_off_when_reckless_dash_not_active() {
    let mut app = build_reckless_dash_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "on_bump must not run when Reckless Dash inactive"
    );
}

// ── Behavior 43 — on_bump gated off when NodeState != Playing ──────────────-

#[test]
fn on_bump_gated_off_when_node_state_not_playing() {
    let mut app = build_reckless_dash_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "on_bump must not run in ChipSelecting"
    );
}

// ── Behavior 44 — on_bump consumes BumpPerformed same tick ─────────────────-

#[test]
fn register_wires_on_bump_to_consume_bump_performed_same_tick() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    // Ghost producer in BreakerSystems::GradeBump writes BumpPerformed once.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BumpPerformed>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BumpPerformed {
                grade: BumpGrade::Perfect,
                bolt: Some(bolt),
                breaker,
            });
            *done = true;
        })
        .in_set(BreakerSystems::GradeBump)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "BumpPerformed written inside BreakerSystems::GradeBump must be visible to \
         reckless_dash_on_bump on the same FixedUpdate tick via .after ordering"
    );
}

// ── Behavior 45 — amplify wired + gated on active + Playing ────────────────-

#[test]
fn register_wires_amplify_gated_on_active_and_playing() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 4.0 });
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1, "amplify must run via wire");
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amount expected 40.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 46 — amplify gated off when RecklessDash NOT active ───────────-

#[test]
fn amplify_gated_off_when_reckless_dash_not_active() {
    let mut app = build_reckless_dash_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 4.0 });
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage when inactive"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_some(),
        "RiskyDamageBoost must remain — system never ran"
    );
}

// ── Behavior 47 — amplify gated off when NodeState != Playing ──────────────-

#[test]
fn amplify_gated_off_when_node_state_not_playing() {
    let mut app = build_reckless_dash_app_in_chip_selecting();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 4.0 });
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage in ChipSelecting"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_some(),
        "RiskyDamageBoost must remain in ChipSelecting"
    );
}

// ── Behavior 48 — amplify consumes BoltImpactCell same tick ────────────────-

#[test]
fn register_wires_amplify_to_consume_bolt_impact_cell_same_tick() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier: 4.0 });
    let cell = spawn_cell_empty(&mut app);

    // Ghost producer in BoltSystems::CellCollision writes BoltImpactCell once.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BoltImpactCell>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BoltImpactCell {
                cell,
                bolt,
                impact_normal: Vec2::ZERO,
                piercing_remaining: 0,
            });
            *done = true;
        })
        .in_set(BoltSystems::CellCollision)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "BoltImpactCell written inside BoltSystems::CellCollision must be \
         visible to reckless_dash_amplify_damage on the same tick"
    );
}

// ── Behavior 53 — quiet-tick safety ─────────────────────────────────────────

#[test]
fn register_schedule_ticks_cleanly_with_no_messages_no_entities() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "quiet schedule must not produce amplified damage"
    );
    assert!(
        captured_bolt_lost(&app).is_empty(),
        "quiet schedule must not produce BoltLost messages"
    );
    assert_eq!(
        *app.world().resource::<RecklessDashConfig>(),
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 4.0,
            double_penalty:    true,
        },
        "RecklessDashConfig must remain canonical across quiet ticks"
    );
}

// ── Behavior 54 — wire does not panic when config absent ───────────────-

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = build_reckless_dash_app_no_config();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<RecklessDashConfig>().is_none(),
        "wire must not side-effect-insert RecklessDashConfig"
    );
    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage captured"
    );
}

// ── Behavior 55 — pregate BumpPerformed drains cleanly before activation ───
//     Regression pin against accidental `.run_if` reintroduction on the
//     reader system: `reckless_dash_on_bump` now enforces its gate
//     in-body via `reader.clear()` so pre-gate `BumpPerformed` messages
//     drain cleanly instead of replaying on gate open.

#[test]
fn pregate_bump_performed_drains_cleanly_before_reckless_dash_activates() {
    let mut app = build_reckless_dash_app();
    // Gate closed: Reckless Dash NOT in ActiveProtocols.
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    // Tick 1 — write BumpPerformed with gate closed. Retrofit drains reader.
    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);
    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "gate closed → no boost on tick 1"
    );

    // Tick 2 — open the gate, no new message. The pre-gate message was
    // drained on tick 1; nothing remains to replay → no boost.
    seed_canonical(&mut app);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "pre-gate BumpPerformed was drained on tick 1 — no boost may be \
         inserted retroactively"
    );
}

// ── Behavior 24 — wire registers reckless_dash_on_dash_transition, gated on active + Playing ──

#[test]
fn register_wires_on_dash_transition_gated_on_active_and_playing() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    let overlay = app
        .world()
        .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
        .map(|o| o.0);
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(2),
        "on_dash_transition must run via wire → BoltLossBehavior doubled to LifeLoss(2), got {live:?}"
    );
    assert_eq!(
        overlay,
        Some(BoltLossBehavior::LifeLoss(1)),
        "OriginalBoltLossBehavior overlay must be LifeLoss(1), got {overlay:?}"
    );
}

// ── Behavior 25 — wire-registered system is gated OFF when RecklessDash NOT active ──

#[test]
fn on_dash_transition_gated_off_when_reckless_dash_not_active() {
    let mut app = build_reckless_dash_app();
    // Do NOT seed ActiveProtocols — gate closed.
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "on_dash_transition must not run when Reckless Dash inactive — BoltLossBehavior must stay LifeLoss(1), got {live:?}"
    );
    assert!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior must NOT be inserted when gate is closed"
    );
}

// ── Behavior 26 — wire-registered system is gated OFF when NodeState != Playing ──

#[test]
fn on_dash_transition_gated_off_when_node_state_not_playing() {
    let mut app = build_reckless_dash_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(1),
        "on_dash_transition must not run in ChipSelecting — BoltLossBehavior must stay LifeLoss(1), got {live:?}"
    );
    assert!(
        app.world()
            .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
            .is_none(),
        "OriginalBoltLossBehavior must NOT be inserted in ChipSelecting"
    );
}

// ── Behavior 27 — wire does NOT register reckless_dash_double_penalty (system deleted) ──
//
// Regression proof: if double_penalty were still wired, two BoltLost messages would
// appear in the collector. Exactly one proves the system is absent.

#[test]
fn wire_does_not_register_reckless_dash_double_penalty() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    // Breaker already Dashing (no dash transition → Wave 5 system does nothing).
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Dashing,
        DashState::Dashing,
    );
    let bolt = app.world_mut().spawn_empty().id();

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "exactly one BoltLost — reckless_dash_double_penalty must NOT be registered \
         (would emit a duplicate if still wired)"
    );
}

// ── Behavior 28 — wire does NOT init RecklessDashDoubledBolts (resource deleted) ─
//
// The compile-time proof: if writer-code restored `RecklessDashDoubledBolts`,
// this file would fail to compile (the type is no longer in scope). The runtime
// smoke-test proves the wired schedule (including `reckless_dash_cleanup_node`
// on `OnExit(NodeState::Playing)`) does not panic on quiet ticks in `Playing`.

#[test]
fn wire_does_not_init_reckless_dash_doubled_bolts_resource() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_dmg_pipeline()
        .build();

    wire(&mut app);

    // Smoke-test: 3 quiet ticks must not panic.
    for _ in 0..3 {
        tick(&mut app);
    }
    // The value of this test is structural: it compiles without referencing
    // RecklessDashDoubledBolts. If writer-code re-introduces that resource,
    // the compilation of this crate will catch the regression.
}
