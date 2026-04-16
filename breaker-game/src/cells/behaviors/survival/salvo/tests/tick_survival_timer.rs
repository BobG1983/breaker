//! Tests for `tick_survival_timer` — behaviors 7-12.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::survival::components::{SurvivalTimer, SurvivalTurret},
        definition::AttackPattern,
    },
    prelude::*,
};

// ── Behavior 7: Timer decrements by dt when started is true ──

#[test]
fn timer_decrements_by_dt_when_started() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    // dt = 1/64 = 0.015625, expected ~9.984375
    assert!(
        timer.remaining < 10.0,
        "remaining should have decremented from 10.0, got {}",
        timer.remaining
    );
    let expected = 10.0 - (1.0 / 64.0);
    assert!(
        (timer.remaining - expected).abs() < 0.001,
        "remaining should be approximately {expected}, got {}",
        timer.remaining
    );
}

// Behavior 7 edge: remaining just above one dt
#[test]
fn timer_just_above_one_dt_remains_positive() {
    let mut app = build_tick_survival_timer_app();

    let dt = 1.0 / 64.0; // 0.015625
    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 0.016,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    // 0.016 - 0.015625 = 0.000375 — positive, no self-destruct yet
    assert!(
        timer.remaining > 0.0,
        "remaining should be positive (just above zero), got {}",
        timer.remaining
    );
}

// ── Behavior 8: Timer does NOT decrement when started is false ──

#[test]
fn timer_does_not_decrement_when_not_started() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    assert!(
        (timer.remaining - 10.0).abs() < f32::EPSILON,
        "remaining should be unchanged at 10.0, got {}",
        timer.remaining
    );
}

// Behavior 8 edge: multiple ticks with started false
#[test]
fn timer_unchanged_after_multiple_ticks_when_not_started() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    assert!(
        (timer.remaining - 10.0).abs() < f32::EPSILON,
        "remaining should still be 10.0 after 3 ticks, got {}",
        timer.remaining
    );
}

// ── Behavior 9: Timer reaching <= 0 writes lethal DamageDealt<Cell> ──

#[test]
fn timer_expiry_writes_lethal_self_destruct_damage() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 0.001,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let msgs: Vec<_> = collector.0.iter().filter(|m| m.target == turret).collect();

    assert!(
        !msgs.is_empty(),
        "should have written DamageDealt<Cell> for expired timer"
    );
    let msg = msgs[0];
    assert_eq!(msg.target, turret, "target should be the turret itself");
    assert_eq!(
        msg.amount.to_bits(),
        f32::MAX.to_bits(),
        "amount should be f32::MAX (lethal)"
    );
    assert_eq!(
        msg.dealer,
        Some(turret),
        "dealer should be Some(turret_entity) — self-inflicted"
    );
}

// Behavior 9 edge: remaining exactly 0.0 and started true
#[test]
fn timer_exactly_zero_writes_lethal_damage() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 0.0,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let msgs: Vec<_> = collector.0.iter().filter(|m| m.target == turret).collect();

    assert!(
        !msgs.is_empty(),
        "remaining == 0.0 with started == true should write lethal damage"
    );
    assert_eq!(
        msgs[0].amount.to_bits(),
        f32::MAX.to_bits(),
        "amount should be f32::MAX"
    );
}

// ── Behavior 10: Already expired timer re-attempts kill each tick ──

#[test]
fn already_expired_timer_writes_lethal_damage_each_tick() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: -1.0,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    let msgs: Vec<_> = collector.0.iter().filter(|m| m.target == turret).collect();

    assert!(
        !msgs.is_empty(),
        "already-expired timer should re-attempt lethal damage"
    );
    assert_eq!(
        msgs[0].amount.to_bits(),
        f32::MAX.to_bits(),
        "amount should be f32::MAX"
    );
}

// ── Behavior 11: Entities without SurvivalTimer are not affected ──

#[test]
fn turret_without_survival_timer_not_affected() {
    let mut app = build_tick_survival_timer_app();

    // Spawn a turret with no SurvivalTimer (permanent turret)
    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        None, // no SurvivalTimer
        2.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    // Should have SurvivalTurret but no SurvivalTimer
    assert!(
        app.world().get::<SurvivalTurret>(turret).is_some(),
        "entity should still have SurvivalTurret"
    );
    assert!(
        app.world().get::<SurvivalTimer>(turret).is_none(),
        "entity should not have SurvivalTimer"
    );

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(
        collector.0.is_empty(),
        "no DamageDealt<Cell> should be written for turrets without SurvivalTimer"
    );
}

// ── Behavior 12: System is a no-op when not in NodeState::Playing ──

#[test]
fn timer_no_op_outside_node_playing() {
    let mut app = build_tick_survival_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(0.0, 100.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 0.001,
            started:   true,
        }),
        2.0,
        AttackPattern::StraightDown,
    );

    // Advance only to NodeState::Spawning (not Playing)
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
    // NodeState defaults to Spawning, not Playing

    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    assert!(
        (timer.remaining - 0.001).abs() < f32::EPSILON,
        "remaining should be unchanged at 0.001 outside Playing, got {}",
        timer.remaining
    );

    let collector = app
        .world()
        .resource::<MessageCollector<DamageDealt<Cell>>>();
    assert!(
        collector.0.is_empty(),
        "no damage should be written outside NodeState::Playing"
    );
}
