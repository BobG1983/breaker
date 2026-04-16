//! Tests for `tick_salvo_fire_timer` — behaviors 13-14.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{behaviors::survival::salvo::components::SalvoFireTimer, definition::AttackPattern},
    prelude::*,
};

// ── Behavior 13: SalvoFireTimer decrements by dt each tick ──

#[test]
fn fire_timer_decrements_by_dt() {
    let mut app = build_tick_salvo_fire_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        None,
        2.0, // SalvoFireTimer(2.0)
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    // dt = 1/64 = 0.015625, expected ~1.984375
    let expected = 2.0 - (1.0 / 64.0);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "SalvoFireTimer should be approximately {expected}, got {}",
        timer.0
    );
}

// Behavior 13 edge: SalvoFireTimer(0.0) goes negative
#[test]
fn fire_timer_zero_goes_negative_after_tick() {
    let mut app = build_tick_salvo_fire_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        None,
        0.0, // SalvoFireTimer(0.0)
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    let expected = -(1.0 / 64.0);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "SalvoFireTimer(0.0) should become approximately {expected} after one tick, got {}",
        timer.0
    );
}

// ── Behavior 14: SalvoFireTimer continues decrementing into negative values ──

#[test]
fn fire_timer_continues_decrementing_negative() {
    let mut app = build_tick_salvo_fire_timer_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        None,
        -0.5, // SalvoFireTimer(-0.5)
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    let expected = -0.5 - (1.0 / 64.0);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "SalvoFireTimer(-0.5) should become approximately {expected}, got {}",
        timer.0
    );
}
