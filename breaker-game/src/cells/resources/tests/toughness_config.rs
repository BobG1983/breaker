use super::super::data::*;
use crate::cells::definition::Toughness;

// ── ToughnessConfig::validate() ─────────────────────────────────

#[test]
fn toughness_config_validate_default_passes() {
    assert!(
        ToughnessConfig::default().validate().is_ok(),
        "default ToughnessConfig should pass validation"
    );
}

#[test]
fn toughness_config_validate_rejects_zero_weak_base() {
    let config = ToughnessConfig {
        weak_base: 0.0,
        ..Default::default()
    };
    assert!(
        config.validate().is_err(),
        "weak_base: 0.0 should be rejected"
    );
}

#[test]
fn toughness_config_validate_rejects_nan_tier_multiplier() {
    let config = ToughnessConfig {
        tier_multiplier: f32::NAN,
        ..Default::default()
    };
    assert!(
        config.validate().is_err(),
        "tier_multiplier: NaN should be rejected"
    );
}

#[test]
fn toughness_config_validate_rejects_negative_boss_multiplier() {
    let config = ToughnessConfig {
        boss_multiplier: -1.0,
        ..Default::default()
    };
    assert!(
        config.validate().is_err(),
        "boss_multiplier: -1.0 should be rejected"
    );
}

#[test]
fn toughness_config_validate_rejects_negative_node_multiplier() {
    let config = ToughnessConfig {
        node_multiplier: -0.5,
        ..Default::default()
    };
    assert!(
        config.validate().is_err(),
        "node_multiplier: -0.5 should be rejected"
    );
}

// ── Part D: ToughnessConfig resource tests ──────────────────────

// Behavior 11: ToughnessConfig has correct default fields
#[test]
fn toughness_config_defaults() {
    let config = ToughnessConfig::default();
    assert!((config.weak_base - 10.0).abs() < f32::EPSILON);
    assert!((config.standard_base - 20.0).abs() < f32::EPSILON);
    assert!((config.tough_base - 30.0).abs() < f32::EPSILON);
    assert!((config.tier_multiplier - 1.2).abs() < f32::EPSILON);
    assert!((config.node_multiplier - 0.05).abs() < f32::EPSILON);
    assert!((config.boss_multiplier - 3.0).abs() < f32::EPSILON);
}

// Behavior 12: ToughnessConfig::base_hp() returns base for each variant
#[test]
fn toughness_config_base_hp_weak() {
    let config = ToughnessConfig::default();
    assert!(
        (config.base_hp(Toughness::Weak) - 10.0).abs() < f32::EPSILON,
        "base_hp(Weak) should be 10.0, got {}",
        config.base_hp(Toughness::Weak)
    );
}

#[test]
fn toughness_config_base_hp_standard() {
    let config = ToughnessConfig::default();
    assert!(
        (config.base_hp(Toughness::Standard) - 20.0).abs() < f32::EPSILON,
        "base_hp(Standard) should be 20.0, got {}",
        config.base_hp(Toughness::Standard)
    );
}

#[test]
fn toughness_config_base_hp_tough() {
    let config = ToughnessConfig::default();
    assert!(
        (config.base_hp(Toughness::Tough) - 30.0).abs() < f32::EPSILON,
        "base_hp(Tough) should be 30.0, got {}",
        config.base_hp(Toughness::Tough)
    );
}

// Behavior 12 edge case: custom config
#[test]
fn toughness_config_base_hp_custom() {
    let config = ToughnessConfig {
        weak_base: 15.0,
        ..Default::default()
    };
    assert!(
        (config.base_hp(Toughness::Weak) - 15.0).abs() < f32::EPSILON,
        "custom weak_base should return 15.0, got {}",
        config.base_hp(Toughness::Weak)
    );
}

// Behavior 13: ToughnessConfig::tier_scale()
#[test]
fn toughness_config_tier_scale_identity() {
    let config = ToughnessConfig::default();
    assert!(
        (config.tier_scale(0, 0) - 1.0).abs() < f32::EPSILON,
        "tier_scale(0, 0) should be 1.0, got {}",
        config.tier_scale(0, 0)
    );
}

#[test]
fn toughness_config_tier_scale_position_only() {
    let config = ToughnessConfig::default();
    // 1.2^0 * (1.0 + 0.05*4) = 1.0 * 1.2 = 1.2
    assert!(
        (config.tier_scale(0, 4) - 1.2).abs() < f32::EPSILON,
        "tier_scale(0, 4) should be 1.2, got {}",
        config.tier_scale(0, 4)
    );
}

#[test]
fn toughness_config_tier_scale_tier_only() {
    let config = ToughnessConfig::default();
    // 1.2^3 * (1.0 + 0.05*0) = 1.728
    assert!(
        (config.tier_scale(3, 0) - 1.728).abs() < 0.001,
        "tier_scale(3, 0) should be ~1.728, got {}",
        config.tier_scale(3, 0)
    );
}

#[test]
fn toughness_config_tier_scale_both() {
    let config = ToughnessConfig::default();
    // 1.2^3 * (1.0 + 0.05*4) = 1.728 * 1.2 = 2.0736
    assert!(
        (config.tier_scale(3, 4) - 2.0736).abs() < 0.001,
        "tier_scale(3, 4) should be ~2.0736, got {}",
        config.tier_scale(3, 4)
    );
}

// Behavior 13 edge case: high tier
#[test]
fn toughness_config_tier_scale_high_tier() {
    let config = ToughnessConfig::default();
    // 1.2^10 ≈ 6.1917
    assert!(
        (config.tier_scale(10, 0) - 6.1917).abs() < 0.01,
        "tier_scale(10, 0) should be ~6.1917, got {}",
        config.tier_scale(10, 0)
    );
}

// Behavior 14: ToughnessConfig::hp_for()
#[test]
fn toughness_config_hp_for_weak_tier0_pos0() {
    let config = ToughnessConfig::default();
    assert!(
        (config.hp_for(Toughness::Weak, 0, 0) - 10.0).abs() < f32::EPSILON,
        "hp_for(Weak, 0, 0) should be 10.0, got {}",
        config.hp_for(Toughness::Weak, 0, 0)
    );
}

#[test]
fn toughness_config_hp_for_standard_tier0_pos0() {
    let config = ToughnessConfig::default();
    assert!(
        (config.hp_for(Toughness::Standard, 0, 0) - 20.0).abs() < f32::EPSILON,
        "hp_for(Standard, 0, 0) should be 20.0, got {}",
        config.hp_for(Toughness::Standard, 0, 0)
    );
}

#[test]
fn toughness_config_hp_for_tough_tier0_pos0() {
    let config = ToughnessConfig::default();
    assert!(
        (config.hp_for(Toughness::Tough, 0, 0) - 30.0).abs() < f32::EPSILON,
        "hp_for(Tough, 0, 0) should be 30.0, got {}",
        config.hp_for(Toughness::Tough, 0, 0)
    );
}

#[test]
fn toughness_config_hp_for_standard_tier3_pos4() {
    let config = ToughnessConfig::default();
    // 20.0 * 2.0736 ≈ 41.472
    assert!(
        (config.hp_for(Toughness::Standard, 3, 4) - 41.472).abs() < 0.01,
        "hp_for(Standard, 3, 4) should be ~41.472, got {}",
        config.hp_for(Toughness::Standard, 3, 4)
    );
}

// Behavior 14 (additional): hp_for with position-only scaling
#[test]
fn toughness_config_hp_for_weak_tier0_pos4() {
    let config = ToughnessConfig::default();
    // 10.0 * 1.2 = 12.0
    assert!(
        (config.hp_for(Toughness::Weak, 0, 4) - 12.0).abs() < 0.001,
        "hp_for(Weak, 0, 4) should be 12.0, got {}",
        config.hp_for(Toughness::Weak, 0, 4)
    );
}

#[test]
fn toughness_config_hp_for_standard_tier0_pos4() {
    let config = ToughnessConfig::default();
    // 20.0 * 1.2 = 24.0
    assert!(
        (config.hp_for(Toughness::Standard, 0, 4) - 24.0).abs() < 0.001,
        "hp_for(Standard, 0, 4) should be 24.0, got {}",
        config.hp_for(Toughness::Standard, 0, 4)
    );
}

#[test]
fn toughness_config_hp_for_tough_tier0_pos4() {
    let config = ToughnessConfig::default();
    // 30.0 * 1.2 = 36.0
    assert!(
        (config.hp_for(Toughness::Tough, 0, 4) - 36.0).abs() < 0.001,
        "hp_for(Tough, 0, 4) should be 36.0, got {}",
        config.hp_for(Toughness::Tough, 0, 4)
    );
}

// Behavior 14 (additional): hp_for with tier-only scaling
#[test]
fn toughness_config_hp_for_weak_tier3_pos0() {
    let config = ToughnessConfig::default();
    // 10.0 * 1.728 ≈ 17.28
    assert!(
        (config.hp_for(Toughness::Weak, 3, 0) - 17.28).abs() < 0.001,
        "hp_for(Weak, 3, 0) should be ~17.28, got {}",
        config.hp_for(Toughness::Weak, 3, 0)
    );
}

#[test]
fn toughness_config_hp_for_standard_tier3_pos0() {
    let config = ToughnessConfig::default();
    // 20.0 * 1.728 ≈ 34.56
    assert!(
        (config.hp_for(Toughness::Standard, 3, 0) - 34.56).abs() < 0.001,
        "hp_for(Standard, 3, 0) should be ~34.56, got {}",
        config.hp_for(Toughness::Standard, 3, 0)
    );
}

#[test]
fn toughness_config_hp_for_tough_tier3_pos0() {
    let config = ToughnessConfig::default();
    // 30.0 * 1.728 ≈ 51.84
    assert!(
        (config.hp_for(Toughness::Tough, 3, 0) - 51.84).abs() < 0.001,
        "hp_for(Tough, 3, 0) should be ~51.84, got {}",
        config.hp_for(Toughness::Tough, 3, 0)
    );
}

// Behavior 14 (additional): hp_for with both tier and position scaling
#[test]
fn toughness_config_hp_for_weak_tier3_pos4() {
    let config = ToughnessConfig::default();
    // 10.0 * 2.0736 ≈ 20.736
    assert!(
        (config.hp_for(Toughness::Weak, 3, 4) - 20.736).abs() < 0.001,
        "hp_for(Weak, 3, 4) should be ~20.736, got {}",
        config.hp_for(Toughness::Weak, 3, 4)
    );
}

#[test]
fn toughness_config_hp_for_tough_tier3_pos4() {
    let config = ToughnessConfig::default();
    // 30.0 * 2.0736 ≈ 62.208
    assert!(
        (config.hp_for(Toughness::Tough, 3, 4) - 62.208).abs() < 0.001,
        "hp_for(Tough, 3, 4) should be ~62.208, got {}",
        config.hp_for(Toughness::Tough, 3, 4)
    );
}

// Behavior 15: ToughnessConfig::hp_for_boss()
#[test]
fn toughness_config_hp_for_boss_standard_tier0_pos0() {
    let config = ToughnessConfig::default();
    // 20.0 * 1.0 * 3.0 = 60.0
    assert!(
        (config.hp_for_boss(Toughness::Standard, 0, 0) - 60.0).abs() < f32::EPSILON,
        "hp_for_boss(Standard, 0, 0) should be 60.0, got {}",
        config.hp_for_boss(Toughness::Standard, 0, 0)
    );
}

#[test]
fn toughness_config_hp_for_boss_tough_tier0_pos0() {
    let config = ToughnessConfig::default();
    // 30.0 * 1.0 * 3.0 = 90.0
    assert!(
        (config.hp_for_boss(Toughness::Tough, 0, 0) - 90.0).abs() < f32::EPSILON,
        "hp_for_boss(Tough, 0, 0) should be 90.0, got {}",
        config.hp_for_boss(Toughness::Tough, 0, 0)
    );
}

#[test]
fn toughness_config_hp_for_boss_standard_tier3_pos4() {
    let config = ToughnessConfig::default();
    // 20.0 * 2.0736 * 3.0 ≈ 124.416
    assert!(
        (config.hp_for_boss(Toughness::Standard, 3, 4) - 124.416).abs() < 0.01,
        "hp_for_boss(Standard, 3, 4) should be ~124.416, got {}",
        config.hp_for_boss(Toughness::Standard, 3, 4)
    );
}

// Behavior 15 edge case: boss_multiplier of 1.0
#[test]
fn toughness_config_hp_for_boss_multiplier_one_equals_hp_for() {
    let config = ToughnessConfig {
        boss_multiplier: 1.0,
        ..Default::default()
    };
    let hp = config.hp_for(Toughness::Standard, 0, 0);
    let boss_hp = config.hp_for_boss(Toughness::Standard, 0, 0);
    assert!(
        (hp - boss_hp).abs() < f32::EPSILON,
        "boss_multiplier 1.0 should yield same as hp_for"
    );
}

// Behavior 16: ToughnessConfig derives GameConfig correctly
#[test]
fn toughness_defaults_ron_parses() {
    let ron_str = include_str!("../../../../assets/config/defaults.toughness.ron");
    let result: ToughnessDefaults = ron::de::from_str(ron_str).expect("toughness RON should parse");
    assert!((result.weak_base - 10.0).abs() < f32::EPSILON);
    assert!((result.standard_base - 20.0).abs() < f32::EPSILON);
    assert!((result.tough_base - 30.0).abs() < f32::EPSILON);
}
