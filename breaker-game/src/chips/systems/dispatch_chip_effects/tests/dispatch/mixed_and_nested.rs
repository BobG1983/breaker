//! Mixed dispatch and multiple Stamp tests — behaviors 13-14.
//!
//! Tests for chips with mixed `Fire`+`When` stamps, multiple `RootNode::Stamp`
//! entries, and multi-target dispatch.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, ShieldConfig, ShockwaveConfig},
        stacking::EffectStack,
        storage::BoundEffects,
        types::{EffectType, EntityKind, RootNode, StampTarget, Tree, Trigger},
    },
};

// ── Behavior 13: Mixed Fire and When stamps on Breaker ──

#[test]
fn mixed_fire_and_when_fire_fires_when_stamps() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Mixed".to_owned(),
        description: String::new(),
        rarity: crate::chips::definition::Rarity::Common,
        max_stacks: 5,
        effects: vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.2),
                })),
            ),
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::DeathOccurred(EntityKind::Cell),
                    Box::new(Tree::Fire(EffectType::Shockwave(ShockwaveConfig {
                        base_range: OrderedFloat(24.0),
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
    select_chip(&mut app, "Mixed");

    app.update();

    let damage = app
        .world()
        .get::<EffectStack<DamageBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(
        damage.len(),
        1,
        "DamageBoost should have been fired immediately"
    );

    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry for the When node"
    );
    assert_eq!(bound.0[0].0, "Mixed");
}

// ── Behavior 14: Chip with multiple `RootNode::Stamp` entries dispatches all ──

#[test]
fn multiple_root_stamps_all_dispatched() {
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

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        2,
        "Breaker should have 2 BoundEffects entries: 1 direct + 1 deferred"
    );
    assert_eq!(breaker_bound.0[0].0, "Parry Multi");
    assert_eq!(breaker_bound.0[1].0, "Parry Multi");
}

// ── Behavior 14 edge case: Three `Stamp` entries (Breaker + Bolt + ActiveCells) ──

#[test]
fn three_root_stamps_all_dispatched_to_breaker() {
    let mut app = test_app();

    let def = ChipDefinition {
        name: "Triple".to_owned(),
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
                StampTarget::Bolt,
                Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                    multiplier: OrderedFloat(1.3),
                })),
            ),
            RootNode::Stamp(
                StampTarget::ActiveCells,
                Tree::When(
                    Trigger::Impacted(EntityKind::Bolt),
                    Box::new(Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
                        multiplier: OrderedFloat(1.0),
                    }))),
                ),
            ),
        ],
        ingredients: None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    select_chip(&mut app, "Triple");

    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        3,
        "Breaker should have 3 BoundEffects entries: 1 direct + 2 deferred"
    );
}
