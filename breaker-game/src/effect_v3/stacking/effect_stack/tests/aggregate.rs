use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::{
    chips::definition::Rarity,
    effect_v3::{
        effects::{
            BumpForceConfig, PiercingConfig, QuickStopConfig, RampingDamageConfig, SizeBoostConfig,
            SpeedBoostConfig,
        },
        traits::PassiveEffect,
    },
    prelude::{SourceId, SourceIdExt},
};

fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-5,
        "expected {expected}, got {actual}"
    );
}

// Builder-format `SourceId` fixtures — replace arbitrary string literals with
// canonical `chip:<template>:<rarity>` keys so tests demonstrate the shape
// production code uses.

fn overclock_source() -> SourceId {
    SourceId::chip("Overclock").rarity(Rarity::Common).build()
}

fn augment_source() -> SourceId {
    SourceId::chip("Augment").rarity(Rarity::Common).build()
}

fn feedback_loop_source() -> SourceId {
    SourceId::chip("FeedbackLoop")
        .rarity(Rarity::Common)
        .build()
}

fn amp_source() -> SourceId {
    SourceId::chip("Amp").rarity(Rarity::Common).build()
}

fn splinter_source() -> SourceId {
    SourceId::chip("Splinter").rarity(Rarity::Common).build()
}

fn piercing_bolt_source() -> SourceId {
    SourceId::chip("PiercingBolt")
        .rarity(Rarity::Common)
        .build()
}

fn chrono_passive_source() -> SourceId {
    SourceId::chip("ChronoPassive")
        .rarity(Rarity::Common)
        .build()
}

// Three distinct chip sources used by ordering / multi-entry tests where the
// specific chip identity is irrelevant.
fn alpha_source() -> SourceId {
    SourceId::chip("Alpha").rarity(Rarity::Common).build()
}

fn beta_source() -> SourceId {
    SourceId::chip("Beta").rarity(Rarity::Common).build()
}

fn gamma_source() -> SourceId {
    SourceId::chip("Gamma").rarity(Rarity::Common).build()
}

// ---------------------------------------------------------------
// EffectStack aggregate delegation tests (behaviors 12-17)
// ---------------------------------------------------------------

#[test]
fn aggregate_on_empty_multiplicative_stack_returns_identity_one() {
    let stack = EffectStack::<SpeedBoostConfig>::default();
    assert_f32_eq(stack.aggregate(), 1.0);
}

#[test]
fn aggregate_on_empty_additive_stack_returns_identity_zero() {
    let stack = EffectStack::<PiercingConfig>::default();
    assert_f32_eq(stack.aggregate(), 0.0);
}

#[test]
fn aggregate_delegates_to_passive_effect_and_returns_product() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        augment_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn aggregate_delegates_to_passive_effect_and_returns_sum() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push(splinter_source(), PiercingConfig { charges: 3 });
    stack.push(piercing_bolt_source(), PiercingConfig { charges: 2 });
    assert_f32_eq(stack.aggregate(), 5.0);
}

#[test]
fn removing_entry_from_multiplicative_stack_updates_aggregate() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        feedback_loop_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        amp_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_f32_eq(stack.aggregate(), 6.0);

    stack.remove(
        &amp_source(),
        &SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn removing_all_multiplicative_entries_returns_aggregate_to_identity() {
    let mut stack = EffectStack::<SpeedBoostConfig>::default();
    let config = SpeedBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.push(amp_source(), config.clone());
    stack.remove(&amp_source(), &config);
    assert_f32_eq(stack.aggregate(), 1.0);
}

#[test]
fn removing_entry_from_additive_stack_updates_aggregate() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push(splinter_source(), PiercingConfig { charges: 3 });
    stack.push(piercing_bolt_source(), PiercingConfig { charges: 2 });

    assert_f32_eq(stack.aggregate(), 5.0);

    stack.remove(&splinter_source(), &PiercingConfig { charges: 3 });

    assert_f32_eq(stack.aggregate(), 2.0);
}

// ---------------------------------------------------------------
// Multiplicative PassiveEffect::aggregate tests (behaviors 18-25)
// ---------------------------------------------------------------

#[test]
fn speed_boost_aggregate_empty_returns_one() {
    let entries: &[(SourceId, SpeedBoostConfig)] = &[];
    assert_f32_eq(SpeedBoostConfig::aggregate(entries), 1.0);
}

#[test]
fn speed_boost_aggregate_single_entry_returns_multiplier() {
    let entries = [(
        overclock_source(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    )];
    assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 1.5);
}

#[test]
fn speed_boost_aggregate_two_entries_returns_product() {
    let entries = [
        (
            overclock_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
        (
            feedback_loop_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 3.0);
}

#[test]
fn speed_boost_aggregate_three_entries_returns_product() {
    let entries = [
        (
            alpha_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
        (
            beta_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
        (
            gamma_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 3.75);
}

#[test]
fn speed_boost_aggregate_identity_multiplier_does_not_change_product() {
    let entries = [
        (
            alpha_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.0),
            },
        ),
        (
            beta_source(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 2.0);
}

#[test]
fn size_boost_aggregate_empty_returns_one() {
    let entries: &[(SourceId, SizeBoostConfig)] = &[];
    assert_f32_eq(SizeBoostConfig::aggregate(entries), 1.0);
}

#[test]
fn size_boost_aggregate_two_entries_returns_product() {
    let entries = [
        (
            augment_source(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.2),
            },
        ),
        (
            augment_source(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.3),
            },
        ),
    ];
    assert_f32_eq(SizeBoostConfig::aggregate(&entries), 1.56);
}

#[test]
fn size_boost_aggregate_identity_multiplier_unchanged() {
    let entries = [
        (
            alpha_source(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.0),
            },
        ),
        (
            beta_source(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
    ];
    assert_f32_eq(SizeBoostConfig::aggregate(&entries), 1.5);
}

// NOTE: `damage_boost_aggregate_*` tests were removed in W3 — `DamageBoostConfig`
// no longer implements `PassiveEffect`. Its multiplier math moved to the crate-
// owned `DamageBoostStack::aggregate_persistent()` (exercised by the
// damage_boost config tests and the bolt_cell_collision integration tests).

#[test]
fn bump_force_aggregate_empty_returns_one() {
    let entries: &[(SourceId, BumpForceConfig)] = &[];
    assert_f32_eq(BumpForceConfig::aggregate(entries), 1.0);
}

#[test]
fn bump_force_aggregate_two_entries_returns_product() {
    let entries = [
        (
            augment_source(),
            BumpForceConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
        (
            augment_source(),
            BumpForceConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
    ];
    assert_f32_eq(BumpForceConfig::aggregate(&entries), 1.5625);
}

#[test]
fn quick_stop_aggregate_empty_returns_one() {
    let entries: &[(SourceId, QuickStopConfig)] = &[];
    assert_f32_eq(QuickStopConfig::aggregate(entries), 1.0);
}

#[test]
fn quick_stop_aggregate_two_entries_returns_product() {
    let entries = [
        (
            chrono_passive_source(),
            QuickStopConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
        (
            chrono_passive_source(),
            QuickStopConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
    ];
    assert_f32_eq(QuickStopConfig::aggregate(&entries), 3.0);
}

// NOTE: `vulnerable_aggregate_*` tests were removed in W3 — `VulnerableConfig`
// no longer implements `PassiveEffect`. Its multiplier math moved to the crate-
// owned `VulnerableStack::aggregate_persistent()` (exercised by the vulnerable
// config tests and the bolt_cell_collision integration tests).

// ---------------------------------------------------------------
// Additive PassiveEffect::aggregate tests (behaviors 26-31)
// ---------------------------------------------------------------

#[test]
fn piercing_aggregate_empty_returns_zero() {
    let entries: &[(SourceId, PiercingConfig)] = &[];
    assert_f32_eq(PiercingConfig::aggregate(entries), 0.0);
}

#[test]
fn piercing_aggregate_single_entry_returns_charges_as_f32() {
    let entries = [(splinter_source(), PiercingConfig { charges: 3 })];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
}

#[test]
fn piercing_aggregate_two_entries_returns_sum() {
    let entries = [
        (splinter_source(), PiercingConfig { charges: 3 }),
        (piercing_bolt_source(), PiercingConfig { charges: 2 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 5.0);
}

#[test]
fn piercing_aggregate_zero_charges_does_not_change_sum() {
    let entries = [
        (alpha_source(), PiercingConfig { charges: 0 }),
        (beta_source(), PiercingConfig { charges: 3 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
}

#[test]
fn piercing_aggregate_all_zero_charges_returns_zero() {
    let entries = [
        (alpha_source(), PiercingConfig { charges: 0 }),
        (beta_source(), PiercingConfig { charges: 0 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 0.0);
}

#[test]
fn ramping_damage_aggregate_empty_returns_zero() {
    let entries: &[(SourceId, RampingDamageConfig)] = &[];
    assert_f32_eq(RampingDamageConfig::aggregate(entries), 0.0);
}

#[test]
fn ramping_damage_aggregate_single_entry_returns_increment() {
    let entries = [(
        amp_source(),
        RampingDamageConfig {
            increment: OrderedFloat(0.5),
        },
    )];
    assert_f32_eq(RampingDamageConfig::aggregate(&entries), 0.5);
}

#[test]
fn ramping_damage_aggregate_two_entries_returns_sum() {
    let entries = [
        (
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        ),
        (
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(0.25),
            },
        ),
    ];
    assert_f32_eq(RampingDamageConfig::aggregate(&entries), 0.75);
}

#[test]
fn ramping_damage_aggregate_three_entries_returns_sum() {
    let entries = [
        (
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        ),
        (
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(0.25),
            },
        ),
        (
            amp_source(),
            RampingDamageConfig {
                increment: OrderedFloat(1.0),
            },
        ),
    ];
    assert_f32_eq(RampingDamageConfig::aggregate(&entries), 1.75);
}
