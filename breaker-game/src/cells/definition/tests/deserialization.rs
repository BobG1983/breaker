use super::{super::data::*, helpers::valid_definition};

// ── CellTypeDefinition deserialization ──────────────────────────

#[test]
fn definition_with_no_behaviors_field_deserializes_to_none() {
    let ron_str = r#"(
        id: "test",
        alias: "S",
        toughness: Weak,
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize without behaviors field");
    assert!(def.behaviors.is_none());
}

#[test]
fn definition_with_single_regen_behavior_deserializes() {
    let ron_str = r#"(
        id: "regen",
        alias: "R",
        toughness: Standard,
        color_rgb: (0.3, 4.0, 0.3),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.4,
        damage_blue_range: 0.3,
        damage_blue_base: 0.1,
        behaviors: Some([Regen(rate: 2.0)]),
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize with Regen behavior");
    assert_eq!(def.behaviors, Some(vec![CellBehavior::Regen { rate: 2.0 }]),);
}

// ── alias validation ────────────────────────────────────────────

#[test]
fn validate_rejects_empty_alias() {
    let mut def = valid_definition();
    def.alias = String::new();
    assert!(def.validate().is_err());
}

#[test]
fn validate_rejects_dot_alias() {
    let mut def = valid_definition();
    def.alias = ".".to_owned();
    assert!(def.validate().is_err());
}

#[test]
fn validate_accepts_valid_definition_without_behaviors() {
    let def = valid_definition();
    assert!(def.validate().is_ok());
}
