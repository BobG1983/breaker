use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;

use super::data::*;
use crate::cells::definition::{CellBehavior, CellTypeDefinition, Toughness};

#[test]
fn cell_defaults_width_height_positive() {
    let config = CellConfig::default();
    assert!(config.width > 0.0);
    assert!(config.height > 0.0);
}

#[test]
fn cell_defaults_ron_parses() {
    let ron_str = include_str!("../../../assets/config/defaults.cells.ron");
    let result: CellDefaults = ron::de::from_str(ron_str).expect("cells RON should parse");
    assert!(result.width > 0.0);
}

#[test]
fn all_cell_type_rons_parse() {
    use std::fs;
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/cells");
    for entry in fs::read_dir(dir).expect("assets/cells/ should exist") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            let content = fs::read_to_string(&path).unwrap();
            let def: CellTypeDefinition = ron::de::from_str(&content).unwrap_or_else(|e| {
                panic!("{}: {e}", path.display());
            });
            assert!(
                !def.id.is_empty(),
                "{}: id must not be empty",
                path.display()
            );
            assert!(
                !def.alias.is_empty(),
                "{}: alias must not be empty",
                path.display()
            );
            assert!(
                def.alias != ".",
                "{}: alias must not be '.'",
                path.display()
            );
            // Toughness is an enum — always valid
            let _ = def.toughness;
            assert!(
                def.validate().is_ok(),
                "{}: validate() should pass: {:?}",
                path.display(),
                def.validate(),
            );
        }
    }
}

fn make_cell_type(id: &str, alias: &str, toughness: Toughness) -> CellTypeDefinition {
    CellTypeDefinition {
        id: id.to_owned(),
        alias: alias.to_owned(),
        toughness,
        color_rgb: [1.0, 1.0, 1.0],
        required_to_clear: true,
        damage_hdr_base: 1.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.5,
        damage_blue_base: 0.2,
        behaviors: None,
        effects: None,
    }
}

/// Creates `AssetId` values by adding assets to an `Assets<CellTypeDefinition>` store.
fn asset_pairs(
    defs: Vec<CellTypeDefinition>,
) -> Vec<(AssetId<CellTypeDefinition>, CellTypeDefinition)> {
    let mut assets = Assets::<CellTypeDefinition>::default();
    defs.into_iter()
        .map(|def| {
            let handle = assets.add(def.clone());
            (handle.id(), def)
        })
        .collect()
}

#[test]
fn registry_detects_duplicate_aliases() {
    let def_a = make_cell_type("a", "S", Toughness::Standard);
    let def_b = make_cell_type("b", "S", Toughness::Tough);
    let mut registry = CellTypeRegistry::default();
    registry.insert(def_a.alias.clone(), def_a);
    let had_existing = registry.insert(def_b.alias.clone(), def_b);
    assert!(
        had_existing.is_some(),
        "inserting duplicate alias should replace"
    );
}

// ── SeedableRegistry tests ─────────────────────────────────────

#[test]
fn registry_stores_and_retrieves_by_string_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("standard", "S", Toughness::Standard);
    registry.insert("S".to_owned(), def);
    let retrieved = registry.get("S").expect("registry should contain 'S'");
    assert_eq!(
        retrieved.toughness,
        Toughness::Standard,
        "retrieved toughness should be Standard"
    );
}

#[test]
fn registry_stores_and_retrieves_multi_char_key() {
    let mut registry = CellTypeRegistry::default();
    let def = make_cell_type("guard", "Gu", Toughness::Weak);
    registry.insert("Gu".to_owned(), def);
    let retrieved = registry.get("Gu").expect("registry should contain 'Gu'");
    assert_eq!(retrieved.toughness, Toughness::Weak);
}

#[test]
fn registry_contains_string_key() {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "S".to_owned(),
        make_cell_type("standard", "S", Toughness::Standard),
    );
    assert!(registry.contains("S"), "'S' should be present");
    assert!(!registry.contains("X"), "'X' should not be present");
    assert!(
        !registry.contains("."),
        "'.' (reserved) should not be present"
    );
}

#[test]
fn seed_populates_registry_from_cell_type_definitions() {
    let pairs = asset_pairs(vec![
        make_cell_type("standard", "S", Toughness::Standard),
        make_cell_type("tough", "T", Toughness::Tough),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 2, "registry should contain 2 cell types");
    let standard = registry.get("S").expect("registry should contain 'S'");
    assert_eq!(standard.toughness, Toughness::Standard);
}

#[test]
fn seed_populates_with_multi_char_alias() {
    let pairs = asset_pairs(vec![make_cell_type("guard", "Gu", Toughness::Weak)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(
        registry.get("Gu").is_some(),
        "multi-char alias 'Gu' should be in registry"
    );
}

#[test]
#[should_panic(expected = "reserved alias '.'")]
fn seed_panics_on_dot_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", ".", Toughness::Standard)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

#[test]
#[should_panic(expected = "duplicate cell type alias 'S'")]
fn seed_panics_on_duplicate_alias() {
    let pairs = asset_pairs(vec![
        make_cell_type("first", "S", Toughness::Standard),
        make_cell_type("second", "S", Toughness::Tough),
    ]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);
}

#[test]
fn seed_skips_definition_with_empty_alias() {
    let pairs = asset_pairs(vec![make_cell_type("bad", "", Toughness::Standard)]);

    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    assert_eq!(
        registry.len(),
        0,
        "empty alias should be filtered out by validate()"
    );
}

#[test]
fn seed_clears_existing_entries_before_populating() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", Toughness::Weak));
    assert_eq!(registry.len(), 1);

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    registry.seed(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(registry.get("X").is_none());
    assert!(registry.get("S").is_some());
}

#[test]
fn update_single_upserts_existing_cell_type_by_alias() {
    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    let mut registry = CellTypeRegistry::default();
    registry.seed(&pairs);

    let updated_def = make_cell_type("standard", "S", Toughness::Tough);
    let updated_pairs = asset_pairs(vec![updated_def.clone()]);
    let (updated_id, _) = &updated_pairs[0];
    registry.update_single(*updated_id, &updated_def);

    let standard = registry.get("S").expect("'S' should still exist");
    assert_eq!(
        standard.toughness,
        Toughness::Tough,
        "toughness should be updated to Tough"
    );
}

#[test]
fn update_all_resets_and_reseeds_registry() {
    let mut registry = CellTypeRegistry::default();
    registry.insert("X".to_owned(), make_cell_type("old", "X", Toughness::Weak));

    let pairs = asset_pairs(vec![make_cell_type("standard", "S", Toughness::Standard)]);
    registry.update_all(&pairs);

    assert_eq!(registry.len(), 1);
    assert!(registry.get("X").is_none());
    assert!(registry.get("S").is_some());
}

#[test]
fn asset_dir_returns_cells() {
    assert_eq!(CellTypeRegistry::asset_dir(), "cells");
}

#[test]
fn extensions_returns_cell_ron() {
    assert_eq!(CellTypeRegistry::extensions(), &["cell.ron"]);
}

// ── Part M: RON file content after toughness migration ──────────

#[test]
fn regen_cell_ron_has_regen_behavior() {
    let ron_str = include_str!("../../../assets/cells/regen.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("regen.cell.ron should parse as CellTypeDefinition");
    assert_eq!(def.behaviors, Some(vec![CellBehavior::Regen { rate: 2.0 }]),);
}

#[test]
fn lock_cell_ron_has_no_behaviors() {
    let ron_str = include_str!("../../../assets/cells/lock.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("lock.cell.ron should parse as CellTypeDefinition");
    assert!(def.behaviors.is_none());
}

#[test]
fn standard_and_tough_cell_rons_have_string_alias_and_no_behaviors() {
    let standard_ron = include_str!("../../../assets/cells/standard.cell.ron");
    let standard: CellTypeDefinition =
        ron::de::from_str(standard_ron).expect("standard.cell.ron should parse");
    assert_eq!(standard.alias, "S");
    assert!(standard.behaviors.is_none());

    let tough_ron = include_str!("../../../assets/cells/tough.cell.ron");
    let tough: CellTypeDefinition =
        ron::de::from_str(tough_ron).expect("tough.cell.ron should parse");
    assert_eq!(tough.alias, "T");
    assert!(tough.behaviors.is_none());
}

// Behavior 44: standard.cell.ron has toughness Standard
#[test]
fn standard_cell_ron_has_toughness_standard() {
    let ron_str = include_str!("../../../assets/cells/standard.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 45: tough.cell.ron has toughness Tough
#[test]
fn tough_cell_ron_has_toughness_tough() {
    let ron_str = include_str!("../../../assets/cells/tough.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Tough);
}

// Behavior 46: guarded.cell.ron has guardian_hp_fraction
#[test]
fn guarded_cell_ron_has_guardian_hp_fraction() {
    let ron_str = include_str!("../../../assets/cells/guarded.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    let guarded = def
        .behaviors
        .as_ref()
        .and_then(|b| {
            b.iter().find_map(|beh| match beh {
                CellBehavior::Guarded(g) => Some(g),
                CellBehavior::Regen { .. }
                | CellBehavior::Volatile { .. }
                | CellBehavior::Sequence { .. }
                | CellBehavior::Armored { .. }
                | CellBehavior::Phantom { .. } => None,
            })
        })
        .expect("guarded.cell.ron should have Guarded behavior");
    assert!(
        (guarded.guardian_hp_fraction - 0.5).abs() < f32::EPSILON,
        "guardian_hp_fraction should be 0.5, got {}",
        guarded.guardian_hp_fraction
    );
    assert!(
        guarded.validate().is_ok(),
        "guardian_hp_fraction should be valid"
    );
}

// Behavior 47: regen.cell.ron has toughness Standard
#[test]
fn regen_cell_ron_has_toughness_standard() {
    let ron_str = include_str!("../../../assets/cells/regen.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 48: guardian.cell.ron has toughness Weak
#[test]
fn guardian_cell_ron_has_toughness_weak() {
    let ron_str = include_str!("../../../assets/cells/guardian.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Weak);
}

// Behavior 49: lock.cell.ron has toughness Weak
#[test]
fn lock_cell_ron_has_toughness_weak() {
    let ron_str = include_str!("../../../assets/cells/lock.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Weak);
}

// ── Volatile RON round-trip (Wave 1) ────────────────────────────

#[test]
fn volatile_cell_ron_parses_with_expected_fields() {
    let ron_str = include_str!("../../../assets/cells/volatile.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("volatile.cell.ron should parse as CellTypeDefinition");
    assert_eq!(def.id, "volatile");
    assert_eq!(def.alias, "V");
    assert_eq!(def.toughness, Toughness::Standard);
    assert_eq!(
        def.behaviors,
        Some(vec![CellBehavior::Volatile {
            damage: 25.0,
            radius: 40.0,
        }]),
    );
}

#[test]
fn volatile_cell_ron_passes_validation() {
    let ron_str = include_str!("../../../assets/cells/volatile.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("volatile.cell.ron should parse");
    assert!(
        def.validate().is_ok(),
        "parsed volatile.cell.ron should pass validate()"
    );
}

#[test]
fn inline_ron_with_volatile_deserializes_with_default_toughness() {
    let ron_str = r#"(
        id: "volatile",
        alias: "V",
        color_rgb: (4.0, 0.8, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.3,
        damage_blue_base: 0.1,
        behaviors: Some([Volatile(damage: 25.0, radius: 40.0)]),
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize inline volatile RON");
    assert_eq!(def.toughness, Toughness::Standard);
    assert_eq!(
        def.behaviors,
        Some(vec![CellBehavior::Volatile {
            damage: 25.0,
            radius: 40.0,
        }]),
    );
}

#[test]
fn inline_ron_with_volatile_and_regen_deserializes_in_order() {
    let ron_str = r#"(
        id: "volatile",
        alias: "V",
        color_rgb: (4.0, 0.8, 0.2),
        required_to_clear: true,
        damage_hdr_base: 4.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.3,
        damage_blue_base: 0.1,
        behaviors: Some([Volatile(damage: 0.5, radius: 1.0), Regen(rate: 0.1)]),
    )"#;
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("should deserialize two-behavior volatile+regen RON");
    let behaviors = def.behaviors.expect("behaviors should be Some");
    assert_eq!(behaviors.len(), 2);
    assert_eq!(
        behaviors[0],
        CellBehavior::Volatile {
            damage: 0.5,
            radius: 1.0,
        }
    );
    assert_eq!(behaviors[1], CellBehavior::Regen { rate: 0.1 });
}

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
    let ron_str = include_str!("../../../assets/config/defaults.toughness.ron");
    let result: ToughnessDefaults = ron::de::from_str(ron_str).expect("toughness RON should parse");
    assert!((result.weak_base - 10.0).abs() < f32::EPSILON);
    assert!((result.standard_base - 20.0).abs() < f32::EPSILON);
    assert!((result.tough_base - 30.0).abs() < f32::EPSILON);
}
