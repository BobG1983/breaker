//! Behavior 29 — reconciliation is idempotent across multiple ticks.

use super::super::{
    super::system::OverchargeKillCount,
    helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, run_fixed_update,
        spawn_bolt_with_count, test_app_playing, wire_apply_only,
    },
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 29 — reconciliation is idempotent across multiple ticks ────

#[test]
fn apply_speed_is_idempotent_across_five_ticks() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    let expected = 1.05_f32.powi(4);

    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - expected).abs() < 1e-6);
}

#[test]
fn apply_speed_catches_up_after_mid_sequence_kill_count_mutation() {
    // Edge: after three ticks, mutate kill count to 7. Next tick the
    // reconcile updates in one tick to 1.05^7.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);
    let bolt = spawn_bolt_with_count(&mut app, 4);

    run_fixed_update(&mut app);
    run_fixed_update(&mut app);
    run_fixed_update(&mut app);

    *app.world_mut()
        .get_mut::<OverchargeKillCount>(bolt)
        .unwrap() = OverchargeKillCount(7);

    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05_f32.powi(7)).abs() < 1e-5);

    // One more tick — still idempotent.
    run_fixed_update(&mut app);
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 1);
    assert!((stack.aggregate() - 1.05_f32.powi(7)).abs() < 1e-5);
}
