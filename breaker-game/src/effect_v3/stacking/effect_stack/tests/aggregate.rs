use ordered_float::OrderedFloat;

use super::super::component::*;
use crate::effect_v3::{
    effects::{
        BumpForceConfig, DamageBoostConfig, PiercingConfig, QuickStopConfig, RampingDamageConfig,
        SizeBoostConfig, SpeedBoostConfig, VulnerableConfig,
    },
    traits::PassiveEffect,
};

fn assert_f32_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-5,
        "expected {expected}, got {actual}"
    );
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
        "overclock".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "augment".into(),
        SpeedBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn aggregate_delegates_to_passive_effect_and_returns_sum() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push("splinter".into(), PiercingConfig { charges: 3 });
    stack.push("piercing_bolt".into(), PiercingConfig { charges: 2 });
    assert_f32_eq(stack.aggregate(), 5.0);
}

#[test]
fn removing_entry_from_multiplicative_stack_updates_aggregate() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );
    stack.push(
        "feedback_loop".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(1.5),
        },
    );
    stack.push(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_f32_eq(stack.aggregate(), 6.0);

    stack.remove(
        "amp",
        &DamageBoostConfig {
            multiplier: OrderedFloat(2.0),
        },
    );

    assert_f32_eq(stack.aggregate(), 3.0);
}

#[test]
fn removing_all_multiplicative_entries_returns_aggregate_to_identity() {
    let mut stack = EffectStack::<DamageBoostConfig>::default();
    let config = DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    };
    stack.push("amp".into(), config.clone());
    stack.remove("amp", &config);
    assert_f32_eq(stack.aggregate(), 1.0);
}

#[test]
fn removing_entry_from_additive_stack_updates_aggregate() {
    let mut stack = EffectStack::<PiercingConfig>::default();
    stack.push("splinter".into(), PiercingConfig { charges: 3 });
    stack.push("piercing_bolt".into(), PiercingConfig { charges: 2 });

    assert_f32_eq(stack.aggregate(), 5.0);

    stack.remove("splinter", &PiercingConfig { charges: 3 });

    assert_f32_eq(stack.aggregate(), 2.0);
}

// ---------------------------------------------------------------
// Multiplicative PassiveEffect::aggregate tests (behaviors 18-25)
// ---------------------------------------------------------------

#[test]
fn speed_boost_aggregate_empty_returns_one() {
    let entries: &[(String, SpeedBoostConfig)] = &[];
    assert_f32_eq(SpeedBoostConfig::aggregate(entries), 1.0);
}

#[test]
fn speed_boost_aggregate_single_entry_returns_multiplier() {
    let entries = [(
        "overclock".into(),
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
            "overclock".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
        (
            "feedback_loop".into(),
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
            "a".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
        (
            "b".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
        (
            "c".into(),
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
            "a".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(1.0),
            },
        ),
        (
            "b".into(),
            SpeedBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(SpeedBoostConfig::aggregate(&entries), 2.0);
}

#[test]
fn size_boost_aggregate_empty_returns_one() {
    let entries: &[(String, SizeBoostConfig)] = &[];
    assert_f32_eq(SizeBoostConfig::aggregate(entries), 1.0);
}

#[test]
fn size_boost_aggregate_two_entries_returns_product() {
    let entries = [
        (
            "augment".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.2),
            },
        ),
        (
            "augment".into(),
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
            "a".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.0),
            },
        ),
        (
            "b".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
    ];
    assert_f32_eq(SizeBoostConfig::aggregate(&entries), 1.5);
}

#[test]
fn damage_boost_aggregate_empty_returns_one() {
    let entries: &[(String, DamageBoostConfig)] = &[];
    assert_f32_eq(DamageBoostConfig::aggregate(entries), 1.0);
}

#[test]
fn damage_boost_aggregate_two_entries_returns_product() {
    let entries = [
        (
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
        (
            "amp".into(),
            DamageBoostConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(DamageBoostConfig::aggregate(&entries), 4.0);
}

#[test]
fn damage_boost_aggregate_single_half_multiplier() {
    let entries = [(
        "amp".into(),
        DamageBoostConfig {
            multiplier: OrderedFloat(0.5),
        },
    )];
    assert_f32_eq(DamageBoostConfig::aggregate(&entries), 0.5);
}

#[test]
fn bump_force_aggregate_empty_returns_one() {
    let entries: &[(String, BumpForceConfig)] = &[];
    assert_f32_eq(BumpForceConfig::aggregate(entries), 1.0);
}

#[test]
fn bump_force_aggregate_two_entries_returns_product() {
    let entries = [
        (
            "augment".into(),
            BumpForceConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
        (
            "augment".into(),
            BumpForceConfig {
                multiplier: OrderedFloat(1.25),
            },
        ),
    ];
    assert_f32_eq(BumpForceConfig::aggregate(&entries), 1.5625);
}

#[test]
fn quick_stop_aggregate_empty_returns_one() {
    let entries: &[(String, QuickStopConfig)] = &[];
    assert_f32_eq(QuickStopConfig::aggregate(entries), 1.0);
}

#[test]
fn quick_stop_aggregate_two_entries_returns_product() {
    let entries = [
        (
            "chrono_passive".into(),
            QuickStopConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
        (
            "chrono_passive".into(),
            QuickStopConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
    ];
    assert_f32_eq(QuickStopConfig::aggregate(&entries), 3.0);
}

#[test]
fn vulnerable_aggregate_empty_returns_one() {
    let entries: &[(String, VulnerableConfig)] = &[];
    assert_f32_eq(VulnerableConfig::aggregate(entries), 1.0);
}

#[test]
fn vulnerable_aggregate_two_entries_returns_product() {
    let entries = [
        (
            "decay".into(),
            VulnerableConfig {
                multiplier: OrderedFloat(1.5),
            },
        ),
        (
            "decay".into(),
            VulnerableConfig {
                multiplier: OrderedFloat(2.0),
            },
        ),
    ];
    assert_f32_eq(VulnerableConfig::aggregate(&entries), 3.0);
}

#[test]
fn vulnerable_aggregate_below_one_reduces() {
    let entries = [(
        "shield_effect".into(),
        VulnerableConfig {
            multiplier: OrderedFloat(0.5),
        },
    )];
    assert_f32_eq(VulnerableConfig::aggregate(&entries), 0.5);
}

// ---------------------------------------------------------------
// Additive PassiveEffect::aggregate tests (behaviors 26-31)
// ---------------------------------------------------------------

#[test]
fn piercing_aggregate_empty_returns_zero() {
    let entries: &[(String, PiercingConfig)] = &[];
    assert_f32_eq(PiercingConfig::aggregate(entries), 0.0);
}

#[test]
fn piercing_aggregate_single_entry_returns_charges_as_f32() {
    let entries = [("splinter".into(), PiercingConfig { charges: 3 })];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
}

#[test]
fn piercing_aggregate_two_entries_returns_sum() {
    let entries = [
        ("splinter".into(), PiercingConfig { charges: 3 }),
        ("piercing_bolt".into(), PiercingConfig { charges: 2 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 5.0);
}

#[test]
fn piercing_aggregate_zero_charges_does_not_change_sum() {
    let entries = [
        ("a".into(), PiercingConfig { charges: 0 }),
        ("b".into(), PiercingConfig { charges: 3 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 3.0);
}

#[test]
fn piercing_aggregate_all_zero_charges_returns_zero() {
    let entries = [
        ("a".into(), PiercingConfig { charges: 0 }),
        ("b".into(), PiercingConfig { charges: 0 }),
    ];
    assert_f32_eq(PiercingConfig::aggregate(&entries), 0.0);
}

#[test]
fn ramping_damage_aggregate_empty_returns_zero() {
    let entries: &[(String, RampingDamageConfig)] = &[];
    assert_f32_eq(RampingDamageConfig::aggregate(entries), 0.0);
}

#[test]
fn ramping_damage_aggregate_single_entry_returns_increment() {
    let entries = [(
        "amp".into(),
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
            "amp".into(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        ),
        (
            "amp".into(),
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
            "amp".into(),
            RampingDamageConfig {
                increment: OrderedFloat(0.5),
            },
        ),
        (
            "amp".into(),
            RampingDamageConfig {
                increment: OrderedFloat(0.25),
            },
        ),
        (
            "amp".into(),
            RampingDamageConfig {
                increment: OrderedFloat(1.0),
            },
        ),
    ];
    assert_f32_eq(RampingDamageConfig::aggregate(&entries), 1.75);
}
