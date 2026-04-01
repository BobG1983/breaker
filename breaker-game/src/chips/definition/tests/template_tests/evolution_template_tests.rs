use super::super::super::types::*;
use crate::effect::{EffectKind, EffectNode, RootEffect, Target};

// =========================================================================
// C5: expand_evolution_template produces Rarity::Evolution
// =========================================================================

#[test]
fn expand_evolution_template_produces_evolution_rarity() {
    let template = EvolutionTemplate {
        name: "Voltaic Piercer".to_owned(),
        description: "Evolved piercing".to_owned(),
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(3))],
        }],
        ingredients: vec![EvolutionIngredient {
            chip_name: "Piercing Shot".to_owned(),
            stacks_required: 3,
        }],
    };
    let def = expand_evolution_template(&template);

    assert_eq!(
        def.rarity,
        Rarity::Evolution,
        "evolution template should produce Rarity::Evolution, not {:?}",
        def.rarity
    );
    assert_eq!(def.name, "Voltaic Piercer");
    assert_eq!(def.description, "Evolved piercing");

    // Edge case: must be Evolution, not Legendary
    assert_ne!(
        def.rarity,
        Rarity::Legendary,
        "evolution rarity must be distinct from Legendary"
    );
}

// =========================================================================
// C7: expand_evolution_template gets max_stacks from template
// =========================================================================

#[test]
fn expand_evolution_template_max_stacks_from_template() {
    let template = EvolutionTemplate {
        name: "Multi Stack Evo".to_owned(),
        description: "Stackable evolution".to_owned(),
        max_stacks: 5,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: vec![EvolutionIngredient {
            chip_name: "Splinter".to_owned(),
            stacks_required: 2,
        }],
    };
    let def = expand_evolution_template(&template);
    assert_eq!(
        def.max_stacks, 5,
        "max_stacks should be 5 from the template, got {}",
        def.max_stacks
    );

    // Edge case: max_stacks = 1 (the serde default)
    let template_one = EvolutionTemplate {
        name: "Single Stack Evo".to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: vec![EvolutionIngredient {
            chip_name: "X".to_owned(),
            stacks_required: 1,
        }],
    };
    let def_one = expand_evolution_template(&template_one);
    assert_eq!(
        def_one.max_stacks, 1,
        "max_stacks should be 1 when template specifies 1, got {}",
        def_one.max_stacks
    );
}

// =========================================================================
// C8: expand_evolution_template copies effects Vec unchanged
// =========================================================================

#[test]
fn expand_evolution_template_copies_effects_unchanged() {
    let template = EvolutionTemplate {
        name: "Combo Evo".to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::Piercing(2))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::SizeBoost(15.0))],
            },
        ],
        ingredients: vec![EvolutionIngredient {
            chip_name: "A".to_owned(),
            stacks_required: 1,
        }],
    };
    let def = expand_evolution_template(&template);

    assert_eq!(
        def.effects.len(),
        2,
        "should have 2 effects, got {}",
        def.effects.len()
    );
    assert!(
        matches!(
            &def.effects[0],
            RootEffect::On {
                target: Target::Bolt,
                ..
            }
        ),
        "first effect should target Bolt"
    );
    assert!(
        matches!(
            &def.effects[1],
            RootEffect::On {
                target: Target::Breaker,
                ..
            }
        ),
        "second effect should target Breaker"
    );

    // Edge case: empty effects
    let empty_template = EvolutionTemplate {
        name: "Empty Evo".to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![],
        ingredients: vec![EvolutionIngredient {
            chip_name: "Z".to_owned(),
            stacks_required: 1,
        }],
    };
    let empty_def = expand_evolution_template(&empty_template);
    assert!(
        empty_def.effects.is_empty(),
        "evolution with empty effects should produce ChipDefinition with empty effects"
    );
}

// =========================================================================
// C9: expand_evolution_template sets template_name to None
// =========================================================================

#[test]
fn expand_evolution_template_sets_template_name_to_none() {
    let template = EvolutionTemplate {
        name: "Storm Piercer".to_owned(),
        description: "Storm desc".to_owned(),
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: vec![EvolutionIngredient {
            chip_name: "X".to_owned(),
            stacks_required: 1,
        }],
    };
    let def = expand_evolution_template(&template);

    assert_eq!(
        def.template_name, None,
        "evolution template_name should be None, got {:?}",
        def.template_name
    );
}

// =========================================================================
// C10: expand_evolution_template copies ingredients as Some(vec)
// =========================================================================

#[test]
fn expand_evolution_template_copies_ingredients_as_some_vec() {
    let template = EvolutionTemplate {
        name: "Fusion Chip".to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: vec![
            EvolutionIngredient {
                chip_name: "Alpha".to_owned(),
                stacks_required: 2,
            },
            EvolutionIngredient {
                chip_name: "Beta".to_owned(),
                stacks_required: 3,
            },
        ],
    };
    let def = expand_evolution_template(&template);

    assert!(
        def.ingredients.is_some(),
        "ingredients should be Some, got None"
    );
    let ingredients = def.ingredients.as_ref().unwrap();
    assert_eq!(
        ingredients.len(),
        2,
        "should have 2 ingredients, got {}",
        ingredients.len()
    );
    assert_eq!(ingredients[0].chip_name, "Alpha");
    assert_eq!(ingredients[0].stacks_required, 2);
    assert_eq!(ingredients[1].chip_name, "Beta");
    assert_eq!(ingredients[1].stacks_required, 3);

    // Edge case: empty ingredients list -> Some(vec![]), not None
    let empty_template = EvolutionTemplate {
        name: "Empty Ingredients Evo".to_owned(),
        description: String::new(),
        max_stacks: 1,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
        ingredients: vec![],
    };
    let empty_def = expand_evolution_template(&empty_template);
    assert_eq!(
        empty_def.ingredients,
        Some(vec![]),
        "empty ingredients should produce Some(vec![]), not None"
    );
}
