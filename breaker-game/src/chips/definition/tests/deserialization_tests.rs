use crate::{
    chips::definition::types::*,
    effect::{ImpactTarget, Target},
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

#[test]
fn rarity_deserializes_legendary() {
    let r: Rarity = ron::de::from_str("Legendary").expect("should parse Legendary");
    assert_eq!(r, Rarity::Legendary);
}

// =========================================================================
// Preserved tests: ImpactTarget deserialization
// =========================================================================

#[test]
fn impact_target_deserializes_cell() {
    let t: ImpactTarget = ron::de::from_str("Cell").expect("should parse Cell");
    assert_eq!(t, ImpactTarget::Cell);
}

#[test]
fn impact_target_deserializes_breaker() {
    let t: ImpactTarget = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, ImpactTarget::Breaker);
}

#[test]
fn impact_target_deserializes_wall() {
    let t: ImpactTarget = ron::de::from_str("Wall").expect("should parse Wall");
    assert_eq!(t, ImpactTarget::Wall);
}

// =========================================================================
// Preserved tests: Target deserialization
// =========================================================================

#[test]
fn target_deserializes_bolt() {
    let t: Target = ron::de::from_str("Bolt").expect("should parse Bolt");
    assert_eq!(t, Target::Bolt);
}

#[test]
fn target_deserializes_breaker() {
    let t: Target = ron::de::from_str("Breaker").expect("should parse Breaker");
    assert_eq!(t, Target::Breaker);
}

#[test]
fn target_deserializes_all_bolts() {
    let t: Target = ron::de::from_str("AllBolts").expect("should parse AllBolts");
    assert_eq!(t, Target::AllBolts);
}

#[test]
fn target_cell_is_valid_variant() {
    let result = ron::de::from_str::<Target>("Cell");
    assert!(result.is_ok(), "Target::Cell should be a valid variant");
    assert_eq!(result.unwrap(), Target::Cell);
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
