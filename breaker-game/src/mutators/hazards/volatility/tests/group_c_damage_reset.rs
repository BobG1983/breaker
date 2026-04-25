use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    mutators::hazards::{definition::HazardKind, resources::hazard_active},
    prelude::*,
};

// ══════════════════════════════════════════════════════════════════════
// Group C — reset_volatility_on_damage
// ══════════════════════════════════════════════════════════════════════

// Behavior 10 — damage resets timer to 0.0
#[test]
fn damage_resets_timer_to_zero() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 8.0, 10.0, 4.5);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, 2.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 6.0).abs() < f32::EPSILON,
        "Hp.current should be 6.0 after 2.0 damage on 8.0, got {}",
        hp.current
    );
}

// Behavior 10 edge — reset from 0.0 stays 0.0
#[test]
fn damage_reset_from_zero_is_idempotent() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 0.0);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, 1.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
}

// Behavior 11 — batch reset: damaged cells reset, undamaged cell advances
#[test]
fn batch_damage_only_damaged_cells_reset() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

    {
        let mut pending = app.world_mut().resource_mut::<PendingCellDamage>();
        pending.0.push((a, 1.0));
        pending.0.push((c, 1.0));
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let ta = app.world().get::<VolatilityTimer>(a).unwrap();
    let tc = app.world().get::<VolatilityTimer>(c).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        ta.elapsed < 0.02,
        "A.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        ta.elapsed
    );
    assert!(
        tc.elapsed < 0.02,
        "C.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        tc.elapsed
    );
    let tb = app.world().get::<VolatilityTimer>(b).unwrap();
    assert!(
        tb.elapsed >= 3.0 && tb.elapsed <= 3.016 + 1e-4,
        "B.elapsed should be in [3.0, 3.016+eps], got {}",
        tb.elapsed
    );
}

// Behavior 11 edge — all three damaged, all reset
#[test]
fn batch_all_three_damaged_all_reset() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

    {
        let mut pending = app.world_mut().resource_mut::<PendingCellDamage>();
        pending.0.push((a, 1.0));
        pending.0.push((b, 1.0));
        pending.0.push((c, 1.0));
    }
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    for entity in [a, b, c] {
        let t = app.world().get::<VolatilityTimer>(entity).unwrap();
        // After a reset in the same tick, grow advances elapsed by dt.
        // Tolerate up to one fixed-update tick's worth of accumulation.
        assert!(
            t.elapsed < 0.02,
            "elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
            t.elapsed
        );
    }
}

// Behavior 11 edge — zero damage, all advance
#[test]
fn batch_zero_damaged_all_advance() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let a = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let b = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);
    let c = spawn_cell_with_timer(&mut app, 5.0, 5.0, 3.0);

    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    for entity in [a, b, c] {
        let t = app.world().get::<VolatilityTimer>(entity).unwrap();
        assert!(
            t.elapsed >= 3.0 && t.elapsed <= 3.016 + 1e-4,
            "elapsed should be in [3.0, 3.016+eps], got {}",
            t.elapsed
        );
    }
}

// Behavior 12 — ordering: reset runs after apply_damage
#[test]
fn reset_runs_after_apply_damage_both_effects_applied() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 2.0);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, 3.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 7.0).abs() < f32::EPSILON,
        "Hp.current should be 7.0 after 3.0 damage, got {}",
        hp.current
    );
    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
}

// Behavior 13 — zero-damage hit still resets the timer
#[test]
fn zero_damage_hit_resets_timer() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, 0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!(
        (hp.current - 10.0).abs() < f32::EPSILON,
        "zero damage should leave Hp unchanged at 10.0, got {}",
        hp.current
    );
}

// Behavior 13 edge — negative-zero damage resets
#[test]
fn negative_zero_damage_resets_timer() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, -0.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
}

// Behavior 13 edge — NaN damage still resets
#[test]
fn nan_damage_resets_timer() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    register_volatility_systems(&mut app);
    let cell = spawn_cell_with_timer(&mut app, 10.0, 10.0, 4.8);

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, f32::NAN));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let timer = app.world().get::<VolatilityTimer>(cell).unwrap();
    // After a reset in the same tick, grow advances elapsed by dt.
    // Tolerate up to one fixed-update tick's worth of accumulation.
    assert!(
        timer.elapsed < 0.02,
        "timer.elapsed should be near 0.0 after reset (≤ one tick's dt), got {}",
        timer.elapsed
    );
}

// Behavior 14 — reset silently skips targets lacking VolatilityTimer
#[test]
fn reset_silently_skips_cell_without_timer() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    // Register only reset (we deliberately skip the attach system so the
    // cell genuinely has no VolatilityTimer when damage arrives).
    app.add_systems(
        FixedUpdate,
        reset_volatility_on_damage
            .after(DmgSystems::ApplyDamage)
            .run_if(hazard_active(HazardKind::Volatility))
            .run_if(in_state(NodeState::Playing)),
    );
    let cell = app
        .world_mut()
        .spawn((
            Cell,
            Hp {
                current:  5.0,
                starting: 10.0,
                max:      None,
            },
            KilledBy { killer: None },
        ))
        .id();

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((cell, 1.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    assert!(app.world().get::<VolatilityTimer>(cell).is_none());
    let hp = app.world().get::<Hp>(cell).unwrap();
    assert!((hp.current - 4.0).abs() < f32::EPSILON);
}

// Behavior 14 edge — target is non-cell (a bare entity, not a Cell)
#[test]
fn reset_silently_skips_non_cell_target() {
    let mut app = test_app_playing_with_damage();
    install_default_config(&mut app);
    add_volatility_stacks(&mut app, 1);
    app.add_systems(
        FixedUpdate,
        reset_volatility_on_damage
            .after(DmgSystems::ApplyDamage)
            .run_if(hazard_active(HazardKind::Volatility))
            .run_if(in_state(NodeState::Playing)),
    );
    let not_a_cell = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<PendingCellDamage>()
        .0
        .push((not_a_cell, 1.0));
    // Should not panic.
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));
}
