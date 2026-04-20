//! Group F — `register` wiring + run-condition gates (Behaviors 41–55, 63).
//!
//! Pins that `register`-wired systems run under the correct schedules, gated
//! by `protocol_active(RecklessDash)` + `in_state(NodeState::Playing)` on
//! the three `FixedUpdate` reader systems, that same-tick ordering anchors
//! work, that the schedule is harness-safe under missing resources + quiet
//! ticks, and that `register` does NOT initialise `RecklessDashDoubledBolts`
//! (plugin owns init).

use bevy::prelude::*;

use super::{
    super::system::{RecklessDashConfig, RecklessDashDoubledBolts, RiskyDamageBoost, register},
    helpers::{
        build_reckless_dash_app, build_reckless_dash_app_in_chip_selecting,
        build_reckless_dash_app_no_config, captured_bolt_lost, collected_reckless_dash_damage,
        risky_boost, seed_active_protocols_with_reckless_dash, spawn_bolt_with_base_damage,
        spawn_breaker_dashing, spawn_cell_empty, write_bolt_impact_cell, write_bolt_lost,
        write_bump_performed,
    },
};
use crate::{
    bolt::sets::BoltSystems,
    breaker::{messages::BumpGrade, sets::BreakerSystems},
    prelude::*,
    protocol::resources::ActiveProtocols,
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
        "on_bump must run via register → boost inserted"
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
    assert_eq!(msgs.len(), 1, "amplify must run via register");
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

// ── Behavior 49 — double_penalty wired + gated on active + Playing ─────────-

#[test]
fn register_wires_double_penalty_gated_on_active_and_playing() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        2,
        "double_penalty must run via register → extra BoltLost emitted"
    );
}

// ── Behavior 50 — double_penalty gated off when inactive ───────────────────-

#[test]
fn double_penalty_gated_off_when_reckless_dash_not_active() {
    let mut app = build_reckless_dash_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "no duplicate when Reckless Dash inactive"
    );
}

// ── Behavior 51 — double_penalty gated off when NodeState != Playing ───────-

#[test]
fn double_penalty_gated_off_when_node_state_not_playing() {
    let mut app = build_reckless_dash_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        1,
        "no duplicate in ChipSelecting"
    );
}

// ── Behavior 52 — double_penalty consumes BoltLost same tick ───────────────-

#[test]
fn register_wires_double_penalty_to_consume_bolt_lost_same_tick() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    // Ghost producer in BoltSystems::BoltLost writes BoltLost once.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BoltLost>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BoltLost { bolt, breaker });
            *done = true;
        })
        .in_set(BoltSystems::BoltLost)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    assert_eq!(
        captured_bolt_lost(&app).len(),
        2,
        "BoltLost written inside BoltSystems::BoltLost must be visible to \
         reckless_dash_double_penalty on the same tick"
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

// ── Behavior 54 — register does not panic when config absent ───────────────-

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = build_reckless_dash_app_no_config();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<RecklessDashConfig>().is_none(),
        "register must not side-effect-insert RecklessDashConfig"
    );
    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage captured"
    );
}

// ── Behavior 55 — pregate BumpPerformed accumulates until activation ───────-

#[test]
fn pregate_bump_performed_accumulates_until_reckless_dash_activates() {
    let mut app = build_reckless_dash_app();
    // Gate closed: Reckless Dash NOT in ActiveProtocols.
    let breaker = spawn_breaker_dashing(&mut app, 1.0, 0.2); // progress 0.8
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    // Tick 1 — write BumpPerformed with gate closed. No boost inserted.
    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);
    assert_eq!(
        risky_boost(&app, bolt),
        None,
        "gate closed → no boost on tick 1"
    );

    // Tick 2 — open the gate. Pre-gate BumpPerformed is still buffered and
    // the system consumes it, firing a retroactive boost insert. This is
    // expected Bevy semantics under `run_if` gating (retained-reader).
    seed_canonical(&mut app);
    tick(&mut app);

    assert_eq!(
        risky_boost(&app, bolt),
        Some(4.0),
        "pre-gate BumpPerformed fires on first post-activate tick (systemic run_if behavior)"
    );
}

// ── Behavior 63 — register does NOT init RecklessDashDoubledBolts ──────────-

#[test]
fn register_does_not_init_reckless_dash_doubled_bolts_resource() {
    // Crucial: use RAW TestAppBuilder (not build_reckless_dash_app which
    // inits the resource). Do NOT insert RecklessDashDoubledBolts, do NOT
    // call init_resource on it.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .build();

    register(&mut app);

    assert!(
        app.world()
            .get_resource::<RecklessDashDoubledBolts>()
            .is_none(),
        "register must NOT init_resource::<RecklessDashDoubledBolts>() — \
         the plugin owns init (matches Greed / Siphon / Fission pattern)"
    );
}
