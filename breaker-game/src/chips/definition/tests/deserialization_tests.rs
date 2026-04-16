use crate::{
    chips::definition::types::*,
    effect_v3::types::{EntityKind, StampTarget},
};

// =========================================================================
// Preserved tests: Rarity deserialization
// =========================================================================

#[test]
fn rarity_deserializes_common() {
    let r: Rarity = ron::de::from_str("Common").expect("should parse Common");
    assert_eq!(r, Rarity::Common);
}

#[test]
fn rarity_deserializes_uncommon() {
    let r: Rarity = ron::de::from_str("Uncommon").expect("should parse Uncommon");
    assert_eq!(r, Rarity::Uncommon);
}

#[test]
fn rarity_deserializes_rare() {
    let r: Rarity = ron::de::from_str("Rare").expect("should parse Rare");
    assert_eq!(r, Rarity::Rare);
}

// =========================================================================
// Preserved tests: EntityKind deserialization (was ImpactTarget)
// =========================================================================

#[test]
fn entity_kind_deserializes_cell() {
    let t: EntityKind = ron::de::from_str("Cell").expect("should parse Cell");
    assert_eq!(t, EntityKind::Cell);
}

#[test]
fn entity_kind_deserializes_breaker() {
    let t: EntityKind = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, EntityKind::Breaker);
}

#[test]
fn entity_kind_deserializes_wall() {
    let t: EntityKind = ron::de::from_str("Wall").expect("should parse Wall");
    assert_eq!(t, EntityKind::Wall);
}

// =========================================================================
// Preserved tests: StampTarget deserialization (was Target)
// =========================================================================

#[test]
fn stamp_target_deserializes_bolt() {
    let t: StampTarget = ron::de::from_str("Bolt").expect("should parse Bolt");
    assert_eq!(t, StampTarget::Bolt);
}

#[test]
fn stamp_target_deserializes_breaker() {
    let t: StampTarget = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, StampTarget::Breaker);
}

#[test]
fn stamp_target_deserializes_active_bolts() {
    let t: StampTarget = ron::de::from_str("ActiveBolts").expect("should parse ActiveBolts");
    assert_eq!(t, StampTarget::ActiveBolts);
}

#[test]
fn stamp_target_active_cells_is_valid_variant() {
    let result = ron::de::from_str::<StampTarget>("ActiveCells");
    assert!(
        result.is_ok(),
        "StampTarget::ActiveCells should be a valid variant"
    );
    assert_eq!(result.unwrap(), StampTarget::ActiveCells);
}

// =========================================================================
// Preserved tests: EvolutionIngredient
// =========================================================================

#[test]
fn evolution_ingredient_deserializes_from_ron() {
    let ron_str = r#"(chip_name: "Piercing Shot", stacks_required: 2)"#;
    let ingredient: EvolutionIngredient =
        ron::de::from_str(ron_str).expect("should parse EvolutionIngredient");
    assert_eq!(ingredient.chip_name, "Piercing Shot");
    assert_eq!(ingredient.stacks_required, 2);
}
