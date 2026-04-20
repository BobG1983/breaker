//! Group H — `register` wiring + run-condition gates (Behaviors H1–H14).
//!
//! Pins that `register`-wired systems run under the correct schedules,
//! gated by `protocol_active(Burnout)` + `in_state(NodeState::Playing)` on
//! the four `FixedUpdate` reader systems; that same-tick ordering anchors
//! to `BreakerSystems::GradeBump` and `BoltSystems::CellCollision`; that
//! the schedule is harness-safe under missing resources + quiet ticks; and
//! that `register` does NOT init any Burnout-specific resources
//! (Burnout has no plugin-owned resource — only components).

use bevy::prelude::*;

use super::{
    super::system::{BurnoutConfig, BurnoutDamageBoost, BurnoutHeat, BurnoutSpeedBoost, register},
    helpers::{
        build_burnout_app, build_burnout_app_in_chip_selecting, build_burnout_app_no_config,
        canonical_burnout_config, collected_burnout_damage, count_shockwave_sources,
        install_burnout_damage_boost, install_burnout_speed_boost, read_damage_boost_multiplier,
        read_heat, read_speed_boost_remaining, seed_active_protocols_with_burnout, set_heat_state,
        spawn_bolt_with_base_damage, spawn_breaker_moving, spawn_breaker_stationary,
        spawn_cell_empty, tick_n, ticks_for_seconds, write_bolt_impact_cell, write_bump_performed,
    },
};
use crate::{
    bolt::sets::BoltSystems,
    breaker::{messages::BumpGrade, sets::BreakerSystems},
    prelude::*,
    protocol::resources::ActiveProtocols,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── H1 — update_heat wired + gated on active + Playing ─────────────────────-

#[test]
fn register_wires_update_heat_gated_on_active_and_playing() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, Vec2::new(200.0, 0.0));

    tick_n(&mut app, ticks_for_seconds(0.5));

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.heat > 0.0,
        "heat must have increased — update system ran, got {}",
        h.heat
    );
}

// ── H2a — update_heat gated OFF when Burnout NOT active ────────────────────-

#[test]
fn update_heat_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, Vec2::new(200.0, 0.0));

    tick_n(&mut app, 32);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "heat must remain 0.0 — update system never ran, got {}",
        h.heat
    );
}

// ── H2b — update_heat gated OFF when NodeState != Playing ──────────────────-

#[test]
fn update_heat_gated_off_when_node_state_not_playing() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_moving(&mut app, Vec2::ZERO, Vec2::new(200.0, 0.0));

    tick_n(&mut app, 32);

    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "heat must remain 0.0 in ChipSelecting, got {}",
        h.heat
    );
}

// ── H3 — on_bump wired + gated ON — inserts boost when charged ─────────────-

#[test]
fn register_wires_on_bump_gated_on_active_and_playing() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        Some(4.0),
        "on_bump must run via register → boost inserted"
    );
}

// ── H4a — on_bump gated OFF when Burnout NOT active ────────────────────────-

#[test]
fn on_bump_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        None,
        "on_bump must not run when Burnout inactive"
    );
}

// ── H4b — on_bump gated OFF when NodeState != Playing ──────────────────────-

#[test]
fn on_bump_gated_off_when_node_state_not_playing() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        None,
        "on_bump must not run in ChipSelecting"
    );
}

// ── H5 — on_bump consumes BumpPerformed same-tick (after GradeBump) ────────-

#[test]
fn register_wires_on_bump_to_consume_bump_performed_same_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

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
        read_damage_boost_multiplier(&app, bolt),
        Some(4.0),
        "BumpPerformed written inside BreakerSystems::GradeBump must be visible to \
         burnout_on_bump on the same FixedUpdate tick via .after ordering"
    );
}

// ── H6 — amplify wired + gated ON — emits amplified damage ─────────────────-

#[test]
fn register_wires_amplify_gated_on_active_and_playing() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1, "amplify must run via register");
}

// ── H7a — amplify gated OFF when Burnout NOT active ────────────────────────-

#[test]
fn amplify_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no amplified damage when Burnout inactive"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_some(),
        "BurnoutDamageBoost must remain — system never ran"
    );
}

// ── H7b — amplify gated OFF when NodeState != Playing ──────────────────────-

#[test]
fn amplify_gated_off_when_node_state_not_playing() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no amplified damage in ChipSelecting"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_some(),
        "BurnoutDamageBoost must remain in ChipSelecting"
    );
}

// ── H8 — amplify consumes BoltImpactCell same-tick (after CellCollision) ───-

#[test]
fn register_wires_amplify_to_consume_bolt_impact_cell_same_tick() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

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

    let msgs = collected_burnout_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "BoltImpactCell written inside BoltSystems::CellCollision must be visible \
         to burnout_amplify_damage on the same tick"
    );
}

// ── H9 — tick_speed_boost wired + gated ON — decrements remaining ──────────-

#[test]
fn register_wires_tick_speed_boost_gated_on_active_and_playing() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick(&mut app);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        remaining < 2.0,
        "remaining must be strictly less than 2.0 after one tick, got {remaining}"
    );
}

// ── H10a — tick_speed_boost gated OFF when Burnout NOT active ──────────────-

#[test]
fn tick_speed_boost_gated_off_when_burnout_not_active() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 10);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 2.0).abs() < f32::EPSILON,
        "remaining must be 2.0 exactly when inactive, got {remaining}"
    );
}

// ── H10b — tick_speed_boost gated OFF when NodeState != Playing ────────────-

#[test]
fn tick_speed_boost_gated_off_when_node_state_not_playing() {
    let mut app = build_burnout_app_in_chip_selecting();
    seed_canonical(&mut app);
    let breaker = spawn_breaker_stationary(&mut app, Vec2::ZERO);
    install_burnout_speed_boost(&mut app, breaker, 2.0);

    tick_n(&mut app, 10);

    let remaining =
        read_speed_boost_remaining(&app, breaker).expect("BurnoutSpeedBoost should be present");
    assert!(
        (remaining - 2.0).abs() < f32::EPSILON,
        "remaining must be 2.0 exactly in ChipSelecting, got {remaining}"
    );
}

// ── H11 — cleanup_node fires on OnExit(Playing) — unconditional ────────────-

#[test]
fn register_wires_cleanup_node_to_fire_on_exit_playing_unconditional() {
    let mut app = build_burnout_app();
    // Do NOT seed ActiveProtocols.
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 0.7, 0.3, true);
    install_burnout_speed_boost(&mut app, breaker, 1.5);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);

    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();

    assert!(app.world().get::<BurnoutHeat>(breaker).is_none());
    assert!(app.world().get::<BurnoutSpeedBoost>(breaker).is_none());
    assert!(app.world().get::<BurnoutDamageBoost>(bolt).is_none());
}

// ── H12 — Quiet-tick safety — no messages, no entities, no panic ───────────-

#[test]
fn register_schedule_ticks_cleanly_with_no_messages_no_entities() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "quiet schedule must not produce amplified damage"
    );
    assert_eq!(
        *app.world().resource::<BurnoutConfig>(),
        canonical_burnout_config(),
        "BurnoutConfig must remain canonical across quiet ticks"
    );
}

// ── H13 — register does not panic when BurnoutConfig absent ────────────────-

#[test]
fn register_does_not_panic_when_config_absent() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<BurnoutConfig>().is_none(),
        "register must not side-effect-insert BurnoutConfig"
    );
    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no amplified damage captured"
    );
    assert_eq!(
        count_shockwave_sources(&mut app),
        0,
        "no shockwaves fired when config absent"
    );
}

// ── H14 — register does NOT init any resources (plugin owns init) ──────────-

#[test]
fn register_does_not_init_any_burnout_resources() {
    // Raw TestAppBuilder — no Burnout-specific inserts/inits and no
    // build_burnout_app sugar.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .build();

    register(&mut app);

    assert!(
        app.world().get_resource::<BurnoutConfig>().is_none(),
        "register must NOT side-effect-insert BurnoutConfig — \
         Burnout uses only components, not plugin-owned resources"
    );
}
