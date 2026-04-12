//! Behaviors 11-13: mixed targets, missing definition, component insertion.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::helpers::{TEST_BOLT_NAME, test_app_with_dispatch};
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        definition::BoltDefinition,
        registry::BoltRegistry,
        systems::dispatch_bolt_effects::dispatch_bolt_effects,
    },
    effect_v3::{
        effects::{DamageBoostConfig, SpeedBoostConfig},
        storage::BoundEffects,
        types::{EffectType, RootNode, StampTarget, Tree, Trigger},
    },
};

/// Helper: creates a minimal `BoltDefinition` with the given effects.
fn make_bolt_def(name: &str, effects: Vec<RootNode>) -> BoltDefinition {
    BoltDefinition {
        name: name.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects,
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    }
}

fn speed_boost_tree(multiplier: f32) -> Tree {
    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig {
        multiplier: OrderedFloat(multiplier),
    }))
}

fn damage_boost_tree(multiplier: f32) -> Tree {
    Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(multiplier),
    }))
}

// ---- Behavior 11: Mixed targets dispatched correctly (Aegis-style bolt definition) ----

#[test]
fn dispatch_handles_mixed_targets_aegis_style() {
    use crate::effect_v3::effects::LoseLifeConfig;

    let def = make_bolt_def(
        "AegisBolt",
        vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5))),
            ),
            RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(Trigger::EarlyBumped, Box::new(speed_boost_tree(1.1))),
            ),
            RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(Trigger::LateBumped, Box::new(speed_boost_tree(1.1))),
            ),
        ],
    );
    let mut app = test_app_with_dispatch(def);
    let breaker = crate::breaker::test_utils::spawn_breaker(&mut app, 0.0, 0.0);
    app.world_mut()
        .entity_mut(breaker)
        .insert(BoundEffects::default());
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("AegisBolt".to_owned())))
        .id();
    app.update();

    let breaker_bound = app.world().get::<BoundEffects>(breaker).unwrap();
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "breaker should have exactly 1 effect (BoltLost -> LoseLife)"
    );

    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects inserted");
    assert_eq!(
        bolt_bound.0.len(),
        3,
        "bolt should have exactly 3 effects (PerfectBumped, EarlyBumped, LateBumped)"
    );
}

#[test]
fn dispatch_mixed_targets_no_breaker_entity_only_bolt_effects_dispatched() {
    use crate::effect_v3::effects::LoseLifeConfig;

    let def = make_bolt_def(
        "AegisBolt",
        vec![
            RootNode::Stamp(
                StampTarget::Breaker,
                Tree::When(
                    Trigger::BoltLostOccurred,
                    Box::new(Tree::Fire(EffectType::LoseLife(LoseLifeConfig {}))),
                ),
            ),
            RootNode::Stamp(
                StampTarget::Bolt,
                Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5))),
            ),
        ],
    );
    let mut app = test_app_with_dispatch(def);
    // No breaker entity
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef("AegisBolt".to_owned())))
        .id();
    app.update();

    // Bolt effects should still be dispatched
    let bolt_bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("bolt should have BoundEffects even without breaker entity");
    assert_eq!(
        bolt_bound.0.len(),
        1,
        "bolt should have 1 Bolt-targeted effect even though no breaker exists"
    );
}

// ---- Behavior 12: Missing bolt definition in registry is a no-op (warning logged) ----

#[test]
fn dispatch_with_missing_definition_in_registry_is_noop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(BoltRegistry::default())
        .add_systems(Update, dispatch_bolt_effects);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("NonExistent".to_owned()),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "no effects should be dispatched when definition not in registry"
    );
}

#[test]
fn dispatch_with_registry_missing_specific_name_is_noop() {
    let def = make_bolt_def("OtherBolt", vec![]);
    let mut app = test_app_with_dispatch(def);
    // Spawn bolt referencing a different name than what's in the registry
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef("MissingName".to_owned()),
            BoundEffects::default(),
        ))
        .id();
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        0,
        "no effects should be dispatched when definition name not found"
    );
}

// ---- Behavior 13: BoundEffects inserted on target entities that lack them ----

#[test]
fn dispatch_inserts_bound_effects_when_absent() {
    let tree = Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5)));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );
    let mut app = test_app_with_dispatch(def);
    // Spawn bolt with Bolt marker only -- no BoundEffects
    let bolt = app
        .world_mut()
        .spawn((Bolt, BoltDefinitionRef(TEST_BOLT_NAME.to_owned())))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should be inserted on bolt");
    assert_eq!(bound.0.len(), 1, "bolt should have 1 dispatched entry");
}

#[test]
fn dispatch_appends_to_existing_bound_effects() {
    let tree = Tree::When(Trigger::PerfectBumped, Box::new(speed_boost_tree(1.5)));
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootNode::Stamp(StampTarget::Bolt, tree)],
    );

    let mut app = test_app_with_dispatch(def);
    // Spawn bolt WITH BoundEffects already present
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BoundEffects(vec![(
                "prior_chip".to_owned(),
                Tree::When(Trigger::Bumped, Box::new(damage_boost_tree(1.5))),
            )]),
        ))
        .id();
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(bolt)
        .expect("BoundEffects should still be present on bolt");
    assert_eq!(
        bound.0.len(),
        2,
        "bolt should have 2 entries (1 prior + 1 dispatched)"
    );
    assert_eq!(
        &bound.0[0].0, "prior_chip",
        "prior entry should be preserved at index 0"
    );
}
