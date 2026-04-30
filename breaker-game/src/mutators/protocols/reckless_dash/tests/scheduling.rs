//! Scheduling tests for `reckless_dash_amplify_damage` and (Wave 5)
//! `reckless_dash_on_dash_transition`.
//!
//! Production guarantee: `reckless_dash_amplify_damage` is tagged
//! `.after(BoltSystems::CellCollision).in_set(DmgSystems::EmitDamage)`,
//! placing it ahead of `MutateDamage → ApplyDamage`. These tests pin the functional
//! consequence: when the bolt carries `DamageBoostStack` and
//! `RiskyDamageBoost`, a single `tick(...)` applies the dealer's
//! `DamageBoostStack` multiplier to the amplified `DamageDealt<Cell>` in
//! the same tick as the emission.
//!
//! These scenarios pair the protocol emission with `bolt_cell_collision`'s
//! own emission in the same tick. Post-W6 both emissions are single-apply:
//! `starting_hp − (base × boost) − (base × reckless_dash_mul × boost)`.
//!
//! Behaviors 21–23 pin the ordering guarantees for `reckless_dash_on_dash_transition`:
//! - 21: runs AFTER `BreakerSystems::UpdateState` (sees new `DashState`)
//! - 22: runs BEFORE `BreakerSystems::UpdatePreviousState` (sees old `PreviousDashState`)
//! - 23: runs BEFORE `BreakerSystems::HandleBoltLost` (mutated `BoltLossBehavior` is live)

use bevy::prelude::*;

use super::{
    super::system::{RecklessDashConfig, RiskyDamageBoost, wire},
    helpers::{
        build_reckless_dash_transition_app, captured_reduce_node_timer, force_dash_state,
        seed_active_protocols_with_reckless_dash, spawn_breaker_for_e2e,
        spawn_breaker_for_transition, write_bolt_lost,
    },
};
use crate::{
    bolt::{
        BoltPlugin,
        test_utils::{damage_stack, default_bolt_definition, spawn_bolt},
    },
    breaker::components::{BoltLossBehavior, DashState},
    cells::resources::CellConfig,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
        test_utils::spawn_cell_with_hp,
    },
    prelude::*,
};

fn reckless_dash_scheduling_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_protocol_scaffolding()
        .insert_resource(RecklessDashConfig {
            risky_zone_start:  0.0,
            damage_multiplier: 3.0,
            double_penalty:    false,
        })
        .build();
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "RecklessDash".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::RecklessDash {
                risky_zone_start:  0.0,
                damage_multiplier: 3.0,
                double_penalty:    false,
            },
        });
    app.add_plugins(BoltPlugin);
    wire(&mut app);
    app
}

#[test]
fn reckless_dash_amplify_damage_applies_damage_boost_in_same_tick() {
    let mut app = reckless_dash_scheduling_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_cell_with_hp(&mut app, 0.0, cell_y, 200.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(damage_stack(&[2.0]))
        .insert(RiskyDamageBoost { multiplier: 3.0 });

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    // Post-W6 formula (single-apply on baseline):
    //   200.0 − (bolt_base × boost) − (bolt_base × reckless_dash_mul × boost)
    //     = 200.0 − (10.0 × 2.0) − (10.0 × 3.0 × 2.0)
    //     = 200.0 − 20.0 − 60.0
    //     = 120.0
    assert!(
        (hp - 120.0).abs() < 1e-5,
        "final_hp = 120.0 (baseline 20.0 + risky 60.0 dropped from 200.0), got {hp}"
    );
}

#[test]
fn reckless_dash_amplify_damage_without_boost_uses_identity() {
    let mut app = reckless_dash_scheduling_app();
    let bc = default_bolt_definition();
    let cc = CellConfig::default();

    let cell_y = 100.0;
    let cell = spawn_cell_with_hp(&mut app, 0.0, cell_y, 200.0);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, 400.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(RiskyDamageBoost { multiplier: 3.0 });

    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(cell)
        .expect("cell should still have Hp")
        .current;
    // Without boost: baseline 10.0 (no double-apply) + risky single-apply 30.0
    // = 40.0 damage; final_hp = 200.0 − 40.0 == 160.0.
    assert!(
        (hp - 160.0).abs() < 1e-5,
        "final_hp = 200.0 − 10.0 − 30.0 == 160.0 (no boost), got {hp}"
    );
}

// ── Behavior 21 — reckless_dash_on_dash_transition runs after BreakerSystems::UpdateState ──
//
// If transition ran BEFORE UpdateState, it would see stale DashState (Idle) and not
// fire the Changed<DashState> path. The live BoltLossBehavior doubling proves the
// system observed the NEW DashState written by UpdateState's slot.

#[test]
fn on_dash_transition_runs_after_update_state_sees_new_dash_state() {
    let mut app = build_reckless_dash_transition_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    // Force DashState → Dashing (causes Changed<DashState> to fire next tick).
    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    let live = *app
        .world()
        .get::<BoltLossBehavior>(breaker)
        .expect("breaker must have BoltLossBehavior");
    assert_eq!(
        live,
        BoltLossBehavior::LifeLoss(2),
        "reckless_dash_on_dash_transition must run AFTER UpdateState so it observes \
         DashState::Dashing and doubles BoltLossBehavior to LifeLoss(2), got {live:?}"
    );
}

// ── Behavior 22 — reckless_dash_on_dash_transition runs before BreakerSystems::UpdatePreviousState ──
//
// If transition ran AFTER UpdatePreviousState, PreviousDashState would already be
// overwritten to Dashing, making was_dashing == true and the enter branch would not
// fire. The correct overlay carrying LifeLoss(1) proves the system observed the OLD
// PreviousDashState(Idle).

#[test]
fn on_dash_transition_runs_before_update_previous_state_sees_old_previous_dash_state() {
    let mut app = build_reckless_dash_transition_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, breaker, DashState::Dashing);
    tick(&mut app);

    // The overlay must carry the original LifeLoss(1) value, which is only
    // possible if the system ran before UpdatePreviousState overwrote the snapshot.
    let overlay = app
        .world()
        .get::<super::super::system::OriginalBoltLossBehavior>(breaker)
        .map(|o| o.0);
    assert_eq!(
        overlay,
        Some(BoltLossBehavior::LifeLoss(1)),
        "OriginalBoltLossBehavior must carry LifeLoss(1) — system must run BEFORE \
         UpdatePreviousState to correctly detect the Idle → Dashing transition, got {overlay:?}"
    );
}

// ── Behavior 22 (edge case) — entity already mid-dash has no transition, its behavior unchanged ─

#[test]
fn mid_dash_entity_without_transition_is_unaffected_by_ordering() {
    let mut app = build_reckless_dash_transition_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);

    // This breaker is already Dashing with PreviousDashState(Dashing) — no transition.
    // We never call force_dash_state for it, so Changed<DashState> does NOT fire.
    let steady = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(2),
        DashState::Dashing,
        DashState::Dashing,
    );
    // This breaker transitions Idle → Dashing.
    let transitioning = spawn_breaker_for_transition(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        DashState::Idle,
        DashState::Idle,
    );

    force_dash_state(&mut app, transitioning, DashState::Dashing);
    tick(&mut app);

    let steady_live = *app
        .world()
        .get::<BoltLossBehavior>(steady)
        .expect("steady breaker must have BoltLossBehavior");
    assert_eq!(
        steady_live,
        BoltLossBehavior::LifeLoss(2),
        "steady mid-dash breaker must remain LifeLoss(2) — no Changed<DashState> fired, \
         got {steady_live:?}"
    );
}

// ── Behavior 23 — reckless_dash_on_dash_transition runs before BreakerSystems::HandleBoltLost ──
//
// If transition ran AFTER HandleBoltLost, handle_bolt_lost would see the undoubled
// LifeLoss(1) and only decrement Hp by 1. The Hp decrement of 2 (5.0 → 3.0) proves
// the mutated LifeLoss(2) was live when handle_bolt_lost ran.

#[test]
fn on_dash_transition_runs_before_handle_bolt_lost_doubles_hp_decrement() {
    // Use the e2e app from helpers which wires the full chain.
    let mut app = super::helpers::build_reckless_dash_e2e_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::LifeLoss(1),
        Some(Hp {
            current:  5.0,
            starting: 5.0,
            max:      None,
        }),
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    // Same tick: transition Idle → Dashing AND write BoltLost.
    // If ordering is correct, transition runs first (doubles to LifeLoss(2)),
    // then handle_bolt_lost applies LifeLoss(2) → Hp 5.0 - 2.0 = 3.0.
    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker must have Hp")
        .current;
    assert!(
        (hp - 3.0).abs() < f32::EPSILON,
        "reckless_dash_on_dash_transition must run BEFORE handle_bolt_lost so the \
         doubled LifeLoss(2) is applied (Hp 5.0 - 2.0 = 3.0), got {hp}"
    );
}

// ── Behavior 23 (edge case) — TimeLoss(5.0) with dash enter writes ReduceNodeTimer delta 10.0 ──

#[test]
fn on_dash_transition_before_handle_bolt_lost_timeloss_writes_reduce_node_timer_delta_ten() {
    let mut app = super::helpers::build_reckless_dash_e2e_app();
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 4.0, true);
    let breaker = spawn_breaker_for_e2e(
        &mut app,
        BoltLossBehavior::TimeLoss(5.0),
        None,
        DashState::Idle,
        DashState::Idle,
    );
    let bolt = app.world_mut().spawn_empty().id();

    force_dash_state(&mut app, breaker, DashState::Dashing);
    write_bolt_lost(&mut app, bolt, breaker);
    tick(&mut app);

    let msgs = captured_reduce_node_timer(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly one ReduceNodeTimer must be written (TimeLoss BoltLost)"
    );
    assert!(
        (msgs[0].delta - 10.0).abs() < f32::EPSILON,
        "ReduceNodeTimer delta must be 10.0 (doubled from 5.0), got {}",
        msgs[0].delta
    );
}
