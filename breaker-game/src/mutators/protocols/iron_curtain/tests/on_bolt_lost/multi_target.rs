//! Multi-target wave fan-out and multi-bolt independence
//! (Behaviors 9, 10, 11, 11-edge). Pins:
//!
//! - One `BoltLost` produces one message per cell that survives the
//!   falloff math; zero-damage cells are silently skipped.
//! - `.abs()` on the y-distance means cells below the breaker take
//!   damage symmetrically with cells above.
//! - Two `BoltLost` in the same frame produce two independent waves —
//!   each carries its own bolt's base damage.

use bevy::prelude::*;

use super::super::helpers::{
    amount_for_target, build_iron_curtain_app, build_iron_curtain_app_with_playfield_height,
    collected_iron_curtain_damage, seed_active_protocols_with_iron_curtain,
    spawn_bolt_with_base_damage, spawn_breaker_at, spawn_cell_at, write_bolt_lost,
};
use crate::prelude::*;

// ── Behavior 9 — multi-cell wave: full + falloff + zero ────────────────────-

#[test]
fn multi_cell_wave_mixes_full_damage_falloff_and_zero_damage() {
    let mut app = build_iron_curtain_app_with_playfield_height(400.0);
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // Distance 20.0 → full damage 10.0.
    let cell_close = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    // Distance 200.0 → factor (1 - 150/350) ≈ 0.5714286 → amount ≈ 5.7142857.
    let cell_mid = spawn_cell_at(&mut app, Vec2::new(0.0, 0.0));
    // Distance 350.0 → falloff_distance 300.0, factor 50/350 ≈ 0.142857,
    // amount ≈ 1.42857.
    let cell_edge = spawn_cell_at(&mut app, Vec2::new(0.0, 150.0));
    // Distance 400.0 → factor 0.0 → no message.
    let cell_max = spawn_cell_at(&mut app, Vec2::new(0.0, 200.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        3,
        "expected 3 damage messages (cell_max yields zero → no message), got {}",
        msgs.len()
    );

    let a_close =
        amount_for_target(&msgs, cell_close).expect("cell_close must have a damage message");
    assert!(
        (a_close - 10.0).abs() < 1e-4,
        "cell_close amount expected 10.0, got {a_close}"
    );

    let a_mid = amount_for_target(&msgs, cell_mid).expect("cell_mid must have a damage message");
    assert!(
        (a_mid - 5.714_286).abs() < 1e-4,
        "cell_mid amount expected ≈5.7143, got {a_mid}"
    );

    let a_edge = amount_for_target(&msgs, cell_edge).expect("cell_edge must have a damage message");
    assert!(
        (a_edge - 1.428_571).abs() < 1e-4,
        "cell_edge amount expected ≈1.4286, got {a_edge}"
    );

    assert!(
        amount_for_target(&msgs, cell_max).is_none(),
        "cell_max must have NO damage message (zero-damage skip)"
    );
}

// ── Behavior 10 — .abs() makes cells below the breaker also take damage ────-

#[test]
fn cells_below_breaker_are_damaged_by_abs_symmetric_falloff() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    // Cell y=-230 is 30 below breaker, abs distance = 30.0 ≤ 50.0 → full damage.
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -230.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 1, "below-breaker cell must take damage");
    assert_eq!(msgs[0].target, cell);
    assert!(
        (msgs[0].amount - 10.0).abs() < 1e-4,
        "abs-distance ≤ falloff_start → full origin damage 10.0, got {}",
        msgs[0].amount
    );
}

// ── Behavior 11 — two `BoltLost` in one frame produce two independent waves ──-

#[test]
fn two_bolts_lost_same_frame_produce_two_independent_waves() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 20.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt_a);
    write_bolt_lost(&mut app, bolt_b);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(msgs.len(), 2, "two `BoltLost` → two waves → two messages");
    for m in &msgs {
        assert_eq!(m.target, cell);
        assert!(
            (m.amount - 10.0).abs() < 1e-4,
            "each wave must emit 10.0, got {}",
            m.amount
        );
    }
}

// ── Behavior 11 (edge case) — two bolts with different BoltBaseDamage ──────-

#[test]
fn two_bolts_lost_same_frame_with_different_base_damages_produce_two_independent_waves() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    let cell = spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 20.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 40.0);

    write_bolt_lost(&mut app, bolt_a);
    write_bolt_lost(&mut app, bolt_b);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        2,
        "two `BoltLost` → two messages, got {}",
        msgs.len()
    );
    let mut amounts: Vec<f32> = msgs.iter().map(|m| m.amount).collect();
    amounts.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    assert!(
        (amounts[0] - 10.0).abs() < 1e-4,
        "smaller amount expected 10.0 (= 20.0 * 0.5), got {}",
        amounts[0]
    );
    assert!(
        (amounts[1] - 20.0).abs() < 1e-4,
        "larger amount expected 20.0 (= 40.0 * 0.5), got {}",
        amounts[1]
    );
    for m in &msgs {
        assert_eq!(m.target, cell);
    }
}
