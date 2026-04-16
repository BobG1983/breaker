use crate::cells::definition::{CellBehavior, CellTypeDefinition, Toughness};

// ── Part M: RON file content after toughness migration ──────────

#[test]
fn regen_cell_ron_has_regen_behavior() {
    let ron_str = include_str!("../../../../assets/cells/regen.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("regen.cell.ron should parse as CellTypeDefinition");
    assert_eq!(def.behaviors, Some(vec![CellBehavior::Regen { rate: 2.0 }]),);
}

#[test]
fn lock_cell_ron_has_no_behaviors() {
    let ron_str = include_str!("../../../../assets/cells/lock.cell.ron");
    let def: CellTypeDefinition =
        ron::de::from_str(ron_str).expect("lock.cell.ron should parse as CellTypeDefinition");
    assert!(def.behaviors.is_none());
}

#[test]
fn standard_and_tough_cell_rons_have_string_alias_and_no_behaviors() {
    let standard_ron = include_str!("../../../../assets/cells/standard.cell.ron");
    let standard: CellTypeDefinition =
        ron::de::from_str(standard_ron).expect("standard.cell.ron should parse");
    assert_eq!(standard.alias, "S");
    assert!(standard.behaviors.is_none());

    let tough_ron = include_str!("../../../../assets/cells/tough.cell.ron");
    let tough: CellTypeDefinition =
        ron::de::from_str(tough_ron).expect("tough.cell.ron should parse");
    assert_eq!(tough.alias, "T");
    assert!(tough.behaviors.is_none());
}

// Behavior 44: standard.cell.ron has toughness Standard
#[test]
fn standard_cell_ron_has_toughness_standard() {
    let ron_str = include_str!("../../../../assets/cells/standard.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 45: tough.cell.ron has toughness Tough
#[test]
fn tough_cell_ron_has_toughness_tough() {
    let ron_str = include_str!("../../../../assets/cells/tough.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Tough);
}

// Behavior 46: guarded.cell.ron has guardian_hp_fraction
#[test]
fn guarded_cell_ron_has_guardian_hp_fraction() {
    let ron_str = include_str!("../../../../assets/cells/guarded.cell.ron");
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
                | CellBehavior::Phantom { .. }
                | CellBehavior::Magnetic { .. }
                | CellBehavior::Survival { .. }
                | CellBehavior::SurvivalPermanent { .. }
                | CellBehavior::Portal { .. } => None,
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
    let ron_str = include_str!("../../../../assets/cells/regen.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Standard);
}

// Behavior 48: guardian.cell.ron has toughness Weak
#[test]
fn guardian_cell_ron_has_toughness_weak() {
    let ron_str = include_str!("../../../../assets/cells/guardian.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Weak);
}

// Behavior 49: lock.cell.ron has toughness Weak
#[test]
fn lock_cell_ron_has_toughness_weak() {
    let ron_str = include_str!("../../../../assets/cells/lock.cell.ron");
    let def: CellTypeDefinition = ron::de::from_str(ron_str).expect("should parse");
    assert_eq!(def.toughness, Toughness::Weak);
}

// ── Volatile RON round-trip (Wave 1) ────────────────────────────

#[test]
fn volatile_cell_ron_parses_with_expected_fields() {
    let ron_str = include_str!("../../../../assets/cells/volatile.cell.ron");
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
    let ron_str = include_str!("../../../../assets/cells/volatile.cell.ron");
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
