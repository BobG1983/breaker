use super::{super::data::*, helpers::valid_definition};

// ── Part B: CellTypeDefinition with toughness ───────────────────

// Behavior 5: CellTypeDefinition has toughness field
#[test]
fn definition_has_toughness_field() {
    let def = CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::Standard,
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         None,
        effects:           None,
    };
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 5 edge case: toughness defaults via serde when omitted
#[test]
fn definition_toughness_defaults_when_omitted_from_ron() {
    let ron_str = r#"(
        id: "test",
        alias: "T",
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize without toughness field");
    assert_eq!(
        def.toughness,
        Toughness::Standard,
        "toughness should default to Standard"
    );
}

// Behavior 6: validate() no longer checks hp
#[test]
fn validate_accepts_all_toughness_variants() {
    for toughness in [Toughness::Weak, Toughness::Standard, Toughness::Tough] {
        let mut def = valid_definition();
        def.toughness = toughness;
        assert!(
            def.validate().is_ok(),
            "toughness {toughness:?} should pass validation"
        );
    }
}

// Behavior 7: CellTypeDefinition deserializes from RON with toughness field
#[test]
fn definition_deserializes_with_toughness_weak() {
    let ron_str = r#"(
        id: "test",
        alias: "T",
        toughness: Weak,
        color_rgb: (1.0, 0.5, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.2,
        damage_blue_range: 0.4,
        damage_blue_base: 0.2,
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize with toughness: Weak");
    assert_eq!(def.toughness, Toughness::Weak);
}
