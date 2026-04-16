//! Group B — `CellBehavior::Armored` validation.
//!
//! Validates `CellTypeDefinition::validate()` branches for the Armored
//! variant. Uses a local `armor_definition(value, facing)` helper.

use super::helpers::armor_definition;
use crate::cells::{
    behaviors::armored::components::ArmorDirection,
    definition::{CellBehavior, CellTypeDefinition, Toughness},
};

// ── Behavior 5 ─────────────────────────────────────────────────────────────

#[test]
fn validate_accepts_armored_value_1_bottom() {
    let def = armor_definition(1, ArmorDirection::Bottom);
    assert!(def.validate().is_ok());
}

// ── Behavior 6 ─────────────────────────────────────────────────────────────

#[test]
fn validate_accepts_armored_value_2_top() {
    let def = armor_definition(2, ArmorDirection::Top);
    assert!(def.validate().is_ok());
}

// ── Behavior 7 ─────────────────────────────────────────────────────────────

#[test]
fn validate_accepts_armored_value_3_left() {
    let def = armor_definition(3, ArmorDirection::Left);
    assert!(def.validate().is_ok());
}

// ── Behavior 7 edge: Right also accepted ──────────────────────────────────

#[test]
fn validate_accepts_armored_value_3_right() {
    let def = armor_definition(3, ArmorDirection::Right);
    assert!(def.validate().is_ok());
}

// ── Behavior 8 ─────────────────────────────────────────────────────────────

#[test]
fn validate_rejects_armored_value_0_bottom() {
    let def = armor_definition(0, ArmorDirection::Bottom);
    let err = def.validate().expect_err("value 0 should be rejected");
    let lower = err.to_lowercase();
    assert!(
        lower.contains("armored"),
        "error should mention 'armored', got: {err}"
    );
    assert!(
        lower.contains('0'),
        "error should mention the invalid value '0', got: {err}"
    );
}

// ── Behavior 8 edge: value 0 with Top facing also rejected ────────────────

#[test]
fn validate_rejects_armored_value_0_top() {
    let def = armor_definition(0, ArmorDirection::Top);
    assert!(def.validate().is_err());
}

// ── Behavior 9 ─────────────────────────────────────────────────────────────

#[test]
fn validate_rejects_armored_value_4_bottom() {
    let def = armor_definition(4, ArmorDirection::Bottom);
    let err = def.validate().expect_err("value 4 should be rejected");
    let lower = err.to_lowercase();
    assert!(
        lower.contains("armored"),
        "error should mention 'armored', got: {err}"
    );
}

// ── Behavior 9 edge: u8::MAX also rejected ────────────────────────────────

#[test]
fn validate_rejects_armored_value_max_bottom() {
    let def = armor_definition(u8::MAX, ArmorDirection::Bottom);
    assert!(def.validate().is_err());
}

// ── Behavior 10 ────────────────────────────────────────────────────────────

#[test]
fn validate_rejects_invalid_armored_even_after_valid_regen_sibling() {
    let def = CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::default(),
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         Some(vec![
            CellBehavior::Regen { rate: 1.0 },
            CellBehavior::Armored {
                value:  0,
                facing: ArmorDirection::Bottom,
            },
        ]),
        effects:           None,
    };
    assert!(
        def.validate().is_err(),
        "validation must not short-circuit on the valid Regen entry"
    );
}

// ── Behavior 10 edge: reversed order, still rejected ──────────────────────

#[test]
fn validate_rejects_invalid_armored_before_valid_regen_sibling() {
    let def = CellTypeDefinition {
        id:                "test".to_owned(),
        alias:             "T".to_owned(),
        toughness:         Toughness::default(),
        color_rgb:         [1.0, 0.5, 0.2],
        required_to_clear: true,
        damage_hdr_base:   4.0,
        damage_green_min:  0.2,
        damage_blue_range: 0.4,
        damage_blue_base:  0.2,
        behaviors:         Some(vec![
            CellBehavior::Armored {
                value:  4,
                facing: ArmorDirection::Top,
            },
            CellBehavior::Regen { rate: 1.0 },
        ]),
        effects:           None,
    };
    assert!(
        def.validate().is_err(),
        "validation must not short-circuit — order reversed, still rejected"
    );
}
