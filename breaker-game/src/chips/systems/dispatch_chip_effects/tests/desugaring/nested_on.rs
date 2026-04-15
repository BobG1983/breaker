//! Nested target tests — behavior 9.
//!
//! In the new system, nested On nodes don't exist as a concept in chip dispatch.
//! All targeting is handled at the `RootNode` level via Stamp/Spawn.
//! These tests verify that multi-target chips with multiple Stamp entries work.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::{
    chips::{definition::ChipDefinition, systems::dispatch_chip_effects::tests::helpers::*},
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        stacking::EffectStack,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
    prelude::*,
};

// ── Multi-target chip with Breaker + Bolt stamps ──

#[test]
fn multi_target_stamps_breaker_fires_bolt_defers() {
    let mut app = test_app();

    let def = ChipDefinition {
        name:          "Multi Target".to_owned(),
        description:   String::new(),
        rarity:        crate::chips::definition::Rarity::Common,
        max_stacks:    5,
        effects:       vec![
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
        ],
        ingredients:   None,
        template_name: None,
    };
    insert_chip(&mut app, def);

    let breaker = spawn_breaker(&mut app);
    let _bolt = spawn_bolt(&mut app);
    select_chip(&mut app, "Multi Target");

    app.update();

    // Breaker Fire should fire immediately
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .unwrap();
    assert_eq!(stack.len(), 1, "Breaker SpeedBoost should fire immediately");

    // Bolt target should stamp to breaker
    let bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Bolt target should stamp to breaker's BoundEffects"
    );
}
