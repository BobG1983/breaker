//! Behavior 27 — non-Overcharge entries are preserved after reconciliation.

use ordered_float::OrderedFloat;

use super::{
    super::helpers::{
        add_overcharge_stacks, canonical_config, install_overcharge_config, overcharge_entries,
        run_fixed_update, spawn_bolt_with_stack, test_app_playing, wire_apply_only,
    },
    helpers::{chip_feedback_loop, chip_overclock, hazard_haste},
};
use crate::effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack};

// ── Behavior 27 — non-Overcharge entries are preserved ───────────────────

#[test]
fn apply_speed_preserves_non_overcharge_entries() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 2, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 2);
    // 1.5 (chip) * 1.05^2 (overcharge) = 1.65375
    let expected = 1.5 * 1.05_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-5);
}

#[test]
fn apply_speed_preserves_three_foreign_entries() {
    // Edge: seed three non-Overcharge entries + Overcharge.
    // Aggregate = 1.5 * 1.25 * 1.20 * 1.05^2 = 2.480625.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_overcharge_config(&mut app, canonical_config());
    add_overcharge_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        chip_feedback_loop(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.25),
        },
    );
    seed.push(
        hazard_haste(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.20),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, 2, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 4);
    let expected = 1.5 * 1.25 * 1.20 * 1.05_f32.powi(2);
    assert!((stack.aggregate() - expected).abs() < 1e-4);

    // Foreign sources retain their original multipliers.
    let entries = overcharge_entries(stack);
    let chip = entries
        .iter()
        .find(|(s, _)| s == &chip_overclock())
        .expect("chip:Overclock survives");
    assert_eq!(chip.1.multiplier, OrderedFloat(1.5));
    let feedback = entries
        .iter()
        .find(|(s, _)| s == &chip_feedback_loop())
        .expect("chip:FeedbackLoop survives");
    assert_eq!(feedback.1.multiplier, OrderedFloat(1.25));
    let haste = entries
        .iter()
        .find(|(s, _)| s == &hazard_haste())
        .expect("hazard:haste survives");
    assert_eq!(haste.1.multiplier, OrderedFloat(1.20));
}
