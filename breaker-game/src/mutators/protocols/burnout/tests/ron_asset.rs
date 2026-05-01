//! RON-asset structural shape for the Burnout protocol.
//!
//! Pins that `burnout.protocol.ron` parses into a `ProtocolDefinition` with
//! `ProtocolKind::Burnout`, that all numeric tuning fields are finite and
//! non-negative, and that string/identity fields match expected values.
//! Specific numeric values are exercised by design-behaviour tests.

use crate::mutators::protocols::definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning};

#[test]
fn burnout_ron_asset_deserializes_to_protocol_definition() {
    let ron_str = include_str!("../../../../../assets/protocols/burnout.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("burnout.protocol.ron should parse");

    assert_eq!(
        def.kind(),
        ProtocolKind::Burnout,
        "parsed definition must report ProtocolKind::Burnout"
    );

    let ProtocolTuning::Burnout {
        fill_duration,
        drain_duration,
        still_threshold,
        full_heat_damage_multiplier,
        speed_boost_duration,
        shockwave_base_range,
        shockwave_range_per_level,
        shockwave_stacks,
        shockwave_speed,
    } = def.tuning.clone()
    else {
        panic!("expected ProtocolTuning::Burnout, got {:?}", def.tuning);
    };

    assert!(
        fill_duration.is_finite() && fill_duration >= 0.0,
        "fill_duration must be finite and non-negative, got {fill_duration}"
    );
    assert!(
        drain_duration.is_finite() && drain_duration >= 0.0,
        "drain_duration must be finite and non-negative, got {drain_duration}"
    );
    assert!(
        still_threshold.is_finite() && still_threshold >= 0.0,
        "still_threshold must be finite and non-negative, got {still_threshold}"
    );
    assert!(
        full_heat_damage_multiplier.is_finite() && full_heat_damage_multiplier >= 0.0,
        "full_heat_damage_multiplier must be finite and non-negative, got {full_heat_damage_multiplier}"
    );
    assert!(
        speed_boost_duration.is_finite() && speed_boost_duration >= 0.0,
        "speed_boost_duration must be finite and non-negative, got {speed_boost_duration}"
    );
    assert!(
        shockwave_base_range.is_finite() && shockwave_base_range >= 0.0,
        "shockwave_base_range must be finite and non-negative, got {shockwave_base_range}"
    );
    assert!(
        shockwave_range_per_level.is_finite() && shockwave_range_per_level >= 0.0,
        "shockwave_range_per_level must be finite and non-negative, got {shockwave_range_per_level}"
    );
    assert!(
        shockwave_stacks > 0,
        "shockwave_stacks must be > 0, got {shockwave_stacks}"
    );
    assert!(
        shockwave_speed.is_finite() && shockwave_speed >= 0.0,
        "shockwave_speed must be finite and non-negative, got {shockwave_speed}"
    );
}

#[test]
fn burnout_ron_name_description_unlock_tier_pinned_exactly() {
    let ron_str = include_str!("../../../../../assets/protocols/burnout.protocol.ron");
    let def: ProtocolDefinition =
        ron::de::from_str(ron_str).expect("burnout.protocol.ron should parse");

    assert_eq!(def.name, "Burnout", "burnout name drift guard");
    assert_eq!(
        def.description,
        "Move to build heat, stop to drain it into a bump-window speed and damage surge.",
        "burnout description drift guard"
    );
    assert_eq!(def.unlock_tier, 0, "burnout unlock_tier drift guard");
}
