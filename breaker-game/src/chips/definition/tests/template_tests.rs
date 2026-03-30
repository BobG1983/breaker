use super::super::types::*;
use crate::effect::{EffectKind, EffectNode, RootEffect, Target, Trigger};

// =========================================================================
// ChipTemplate with Vec<RootEffect>
// =========================================================================

#[test]
fn chip_template_ron_with_root_effect() {
    let ron_str = r#"(name: "Surge", max_taken: 3, common: Some((prefix: "Basic", effects: [On(target: Bolt, then: [When(trigger: PerfectBump, then: [Do(SpeedBoost(multiplier: 1.2))])])])), uncommon: None, rare: None, legendary: None)"#;
    let template: ChipTemplate =
        ron::de::from_str(ron_str).expect("ChipTemplate with RootEffect RON should parse");
    assert_eq!(template.name, "Surge");
    let common = template.common.unwrap();
    assert!(matches!(
        common.effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

#[test]
fn expand_chip_template_produces_root_effect() {
    let template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.2 })],
                }],
            }],
        }),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Basic Surge");
    assert_eq!(defs[0].rarity, Rarity::Common);
    assert_eq!(defs[0].max_stacks, 3);
    assert!(matches!(
        defs[0].effects[0],
        RootEffect::On {
            target: Target::Bolt,
            ..
        }
    ));
}

#[test]
fn expand_chip_template_preserves_target() {
    let template = ChipTemplate {
        name: "Wide".to_owned(),
        max_taken: 3,
        common: Some(RaritySlot {
            prefix: "Basic".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::SizeBoost(20.0))],
            }],
        }),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert!(
        matches!(
            defs[0].effects[0],
            RootEffect::On {
                target: Target::Breaker,
                ..
            }
        ),
        "expand_chip_template should preserve Target::Breaker from slot"
    );
}

#[test]
fn expanded_defs_have_correct_rarities_with_root_effect() {
    let make_slot = |prefix: &str, val: u32| RaritySlot {
        prefix: prefix.to_owned(),
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(val))],
        }],
    };
    let template = ChipTemplate {
        name: "AllSlots".to_owned(),
        max_taken: 5,
        common: Some(make_slot("C", 1)),
        uncommon: Some(make_slot("U", 2)),
        rare: Some(make_slot("R", 3)),
        legendary: Some(make_slot("L", 4)),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 4);
    assert_eq!(defs[0].rarity, Rarity::Common);
    assert_eq!(defs[1].rarity, Rarity::Uncommon);
    assert_eq!(defs[2].rarity, Rarity::Rare);
    assert_eq!(defs[3].rarity, Rarity::Legendary);
}

// =========================================================================
// Preserved tests: expand_chip_template mechanics
// =========================================================================

#[test]
fn expand_chip_template_all_none_returns_empty() {
    let template = ChipTemplate {
        name: "Empty".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let defs = expand_chip_template(&template);
    assert!(defs.is_empty());
}

#[test]
fn expanded_chip_empty_prefix_uses_template_name() {
    let template = ChipTemplate {
        name: "Glass Cannon".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: Some(RaritySlot {
            prefix: String::new(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "Glass Cannon");
    assert_eq!(defs[0].rarity, Rarity::Legendary);
}

#[test]
fn expanded_chip_whitespace_prefix_uses_template_name() {
    let template = ChipTemplate {
        name: "Glass Cannon".to_owned(),
        max_taken: 1,
        common: None,
        uncommon: None,
        rare: None,
        legendary: Some(RaritySlot {
            prefix: "   ".to_owned(),
            effects: vec![RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 1);
    assert_eq!(
        defs[0].name, "Glass Cannon",
        "whitespace-only prefix should be treated as empty — name should equal template name"
    );
    assert_eq!(defs[0].rarity, Rarity::Legendary);
}

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
// C6: expand_chip_template sets template_name on all variants
// =========================================================================

#[test]
fn expand_chip_template_sets_template_name_on_all_variants() {
    let make_slot = |prefix: &str| RaritySlot {
        prefix: prefix.to_owned(),
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::Piercing(1))],
        }],
    };
    let template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(make_slot("Basic")),
        uncommon: Some(make_slot("Keen")),
        rare: Some(make_slot("Honed")),
        legendary: Some(make_slot("Mythic")),
    };
    let defs = expand_chip_template(&template);
    assert_eq!(defs.len(), 4);

    for (i, def) in defs.iter().enumerate() {
        assert_eq!(
            def.template_name,
            Some("Surge".to_owned()),
            "defs[{i}] ({}) should have template_name == Some(\"Surge\"), got {:?}",
            def.name,
            def.template_name
        );
    }

    // Edge case: single slot template still gets template_name
    let single_template = ChipTemplate {
        name: "Surge".to_owned(),
        max_taken: 3,
        common: Some(make_slot("Basic")),
        uncommon: None,
        rare: None,
        legendary: None,
    };
    let single_defs = expand_chip_template(&single_template);
    assert_eq!(single_defs.len(), 1);
    assert_eq!(
        single_defs[0].template_name,
        Some("Surge".to_owned()),
        "single-slot template should still set template_name"
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
