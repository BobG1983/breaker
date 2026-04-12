//! Mixed target, `chip_name` preservation, and missing breaker
//! tests — behaviors 10-13.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{
            DamageBoostConfig, PiercingConfig, ShieldConfig, ShockwaveConfig, SpeedBoostConfig,
        },
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 10: chip_name is preserved in stamped BoundEffects entries ──

#[test]
fn chip_name_preserved_in_bound_effects() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Surge Bolt",
        StampTarget::Breaker,
        Tree::When(
            Trigger::PerfectBumped,
            Box::new(Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                multiplier: OrderedFloat(1.3),
            }))),
        ),
        5,
    );
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Surge Bolt");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 1);
    assert_eq!(
        bound.0[0].0, "Surge Bolt",
        "chip_name should be preserved in BoundEffects entries"
    );
}

// ── Behavior 11: Multiple Stamp entries preserve chip_name on each ──

#[test]
fn multiple_stamps_preserve_chip_name() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Parry Multi".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::PerfectBumped,
                    Box::new(Tree::Fire(EffectType::Shield(ShieldConfig {
                        duration: OrderedFloat(5.0),
                        reflection_cost: OrderedFloat(0.0),
                    }))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::ActiveBolts,
                Tree::When(
                    Trigger::PerfectBumped,
                    Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                        base_range: OrderedFloat(64.0),
                        range_per_level: OrderedFloat(0.0),
                        stacks: 1,
                        speed: OrderedFloat(500.0),
                    }))),
                ),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Parry Multi");

    app.update();

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(bound.0.len(), 2, "Should have 2 entries");
    assert_eq!(bound.0[0].0, "Parry Multi");
    assert_eq!(bound.0[1].0, "Parry Multi");
}

// ── Behavior 12: Missing breaker for non-Breaker target — no panic ──

#[test]
fn no_breaker_no_panic_for_bolt_target() {
    let mut app = test_app();

    let def = ChipDefinition::test(
        "Empty",
        Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
            multiplier: OrderedFloat(1.0),
        })),
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Empty");
    app.update();
}

#[test]
fn no_breaker_no_panic_for_active_cells_target() {
    let mut app = test_app();

    let def = ChipDefinition::test_on(
        "Cell Target",
        StampTarget::ActiveCells,
        Tree::Fire(EffectType::Piercing(PiercingConfig { charges: 1 })),
        5,
    );
    insert_chip(&mut app, def);

    select_chip(&mut app, "Cell Target");
    app.update();
}

// ── Behavior 13: Mixed Breaker + non-Breaker stamps ──

#[test]
fn mixed_breaker_and_non_breaker_stamps() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Hybrid".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
                    multiplier: OrderedFloat(1.2),
                })),
            ),
            RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(
                    Trigger::Bumped,
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.3),
                    }))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Impacted(EntityKind::Bolt),
                    Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                        base_range: OrderedFloat(32.0),
                        range_per_level: OrderedFloat(0.0),
                        stacks: 1,
                        speed: OrderedFloat(400.0),
                    }))),
                ),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Hybrid");

    app.update();

    // Breaker Fire fires immediately
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "SpeedBoost should fire immediately");

    // Non-Breaker targets stamp to breaker
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "Bolt and ActiveCells trees should stamp to breaker's BoundEffects"
    );
}
