use super::super::data::*;

// ── Part A: Toughness Enum ──────────────────────────────────────

// Behavior 1: Toughness enum has three variants
#[test]
fn toughness_has_three_distinct_variants() {
    // Exhaustive match proves all three variants exist.
    let label = |t: Toughness| match t {
        Toughness::Weak => "weak",
        Toughness::Standard => "standard",
        Toughness::Tough => "tough",
    };
    assert_eq!(label(Toughness::Weak), "weak");
    assert_eq!(label(Toughness::Standard), "standard");
    assert_eq!(label(Toughness::Tough), "tough");
}

// Behavior 1 edge case: Default variant is Standard
#[test]
fn toughness_default_is_standard() {
    assert_eq!(Toughness::default(), Toughness::Standard);
}

// Behavior 2: Toughness::default_base_hp() returns hardcoded fallback per variant
#[test]
fn toughness_weak_default_base_hp_returns_10() {
    assert!(
        (Toughness::Weak.default_base_hp() - 10.0).abs() < f32::EPSILON,
        "Weak.default_base_hp() should return 10.0, got {}",
        Toughness::Weak.default_base_hp()
    );
}

#[test]
fn toughness_standard_default_base_hp_returns_20() {
    assert!(
        (Toughness::Standard.default_base_hp() - 20.0).abs() < f32::EPSILON,
        "Standard.default_base_hp() should return 20.0, got {}",
        Toughness::Standard.default_base_hp()
    );
}

#[test]
fn toughness_tough_default_base_hp_returns_30() {
    assert!(
        (Toughness::Tough.default_base_hp() - 30.0).abs() < f32::EPSILON,
        "Tough.default_base_hp() should return 30.0, got {}",
        Toughness::Tough.default_base_hp()
    );
}

// Behavior 3: Toughness deserializes from RON
#[test]
fn toughness_weak_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Weak").expect("should deserialize Weak");
    assert_eq!(result, Toughness::Weak);
}

#[test]
fn toughness_standard_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Standard").expect("should deserialize Standard");
    assert_eq!(result, Toughness::Standard);
}

#[test]
fn toughness_tough_deserializes_from_ron() {
    let result: Toughness = ron::de::from_str("Tough").expect("should deserialize Tough");
    assert_eq!(result, Toughness::Tough);
}

// Behavior 3 edge case: invalid variant
#[test]
fn toughness_invalid_variant_fails_deserialization() {
    let result: Result<Toughness, _> = ron::de::from_str("Legendary");
    assert!(
        result.is_err(),
        "\"Legendary\" should not deserialize as Toughness"
    );
}

// Behavior 4: Toughness traits
#[test]
fn toughness_is_clone_copy_debug_eq() {
    let t = Toughness::Weak;
    let cloned = t;
    assert_eq!(t, cloned, "copy should equal original");
    let debug_str = format!("{t:?}");
    assert!(
        debug_str.contains("Weak"),
        "debug should contain 'Weak', got: {debug_str}"
    );
    assert_eq!(Toughness::Weak, Toughness::Weak);
    assert_ne!(Toughness::Weak, Toughness::Standard);
}

// Behavior 4 edge case: serialize round-trip
#[test]
fn toughness_serialize_round_trip() {
    let original = Toughness::Tough;
    let serialized = ron::ser::to_string(&original).expect("should serialize");
    let deserialized: Toughness = ron::de::from_str(&serialized).expect("should deserialize");
    assert_eq!(
        original, deserialized,
        "round-trip should produce same value"
    );
}
