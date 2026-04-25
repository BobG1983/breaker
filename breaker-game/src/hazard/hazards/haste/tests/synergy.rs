//! Group G — Cross-hazard / chip synergy pinning.
//!
//! Pins the multiplicative semantics called out in the design doc
//! `docs/design/hazards/haste.md` §"Multiplicative
//! with other speed modifiers" and §"Edge Cases → Haste + Overcharge synergy".

use ordered_float::OrderedFloat;

use super::{
    super::system::HasteConfig,
    helpers::{
        add_haste_stacks, install_haste_config, run_fixed_update, spawn_bolt_with_stack,
        test_app_playing, wire_apply_only,
    },
};
use crate::{
    chips::definition::Rarity,
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack},
    hazard::definition::HazardKind,
    prelude::*,
};

fn chip_overclock() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

fn hazard_overcharge() -> SourceId {
    SourceId::hazard(HazardKind::Overcharge).build()
}

// ── Behavior 26 — Haste 1.30x + chip 1.50x + Overcharge 1.15x ─────────

#[test]
fn haste_chip_and_overcharge_aggregate_multiplicatively() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 2); // Haste multiplier = 1.30

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.15),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 3, "chip + overcharge + haste = 3 entries");
    // 1.5 * 1.15 * 1.30 = 2.2425
    assert!((stack.aggregate() - 2.2425).abs() < 1e-4);
}

#[test]
fn bumping_haste_stack_updates_only_the_haste_entry_in_synergy() {
    // Edge: bump Haste to 3 stacks (multiplier 1.40) between two ticks.
    // After the second tick, aggregate == 1.5 * 1.15 * 1.40 = 2.415.
    // The chip and Overcharge entries are unchanged.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 2);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    seed.push(
        hazard_overcharge(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.15),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);
    add_haste_stacks(&mut app, 1); // 2 → 3 → 1.40x
    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert_eq!(stack.len(), 3);
    // 1.5 * 1.15 * 1.40 = 2.415
    assert!((stack.aggregate() - 2.415).abs() < 1e-4);

    // Non-Haste entries retain their original multipliers.
    let chip_entry = stack
        .iter()
        .find(|(s, _)| s == &chip_overclock())
        .expect("chip entry must survive");
    assert_eq!(chip_entry.1.multiplier, OrderedFloat(1.5));
    let overcharge_entry = stack
        .iter()
        .find(|(s, _)| s == &hazard_overcharge())
        .expect("overcharge entry must survive");
    assert_eq!(overcharge_entry.1.multiplier, OrderedFloat(1.15));
}

// ── Behavior 27 — modifiers multiply, not add (regression guard) ──────

#[test]
fn modifiers_multiply_not_add() {
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    // Multiplicative: 2.0 * 1.20 = 2.40 — distinguishes from additive
    // aggregates that would yield 3.20 (2.0 + 1.20) or 2.20 (2.0 + 0.20).
    assert!((stack.aggregate() - 2.40).abs() < 1e-5);
}

#[test]
fn sub_one_chip_entry_carries_through_product_aggregation() {
    // Edge: a 0.5x chip entry with 1.20x Haste aggregates to 0.60. A
    // buggy additive aggregate would yield 1.70.
    let mut app = test_app_playing();
    wire_apply_only(&mut app);
    install_haste_config(
        &mut app,
        HasteConfig {
            base_percent:      20.0,
            per_level_percent: 10.0,
        },
    );
    add_haste_stacks(&mut app, 1);

    let mut seed = EffectStack::<SpeedBoostConfig>::default();
    seed.push(
        chip_overclock(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(0.5),
        },
    );
    let bolt = spawn_bolt_with_stack(&mut app, seed);

    run_fixed_update(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(bolt)
        .unwrap();
    assert!((stack.aggregate() - 0.60).abs() < 1e-5);
}
