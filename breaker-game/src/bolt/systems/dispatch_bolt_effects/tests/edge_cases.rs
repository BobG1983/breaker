//! Behaviors 11-13: mixed targets, missing definition, component insertion.

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::helpers::{TEST_BOLT_NAME, test_app_with_dispatch};
use crate::{
    bolt::{
        components::{Bolt, BoltDefinitionRef},
        definition::BoltDefinition,
        registry::BoltRegistry,
        systems::dispatch_bolt_effects::dispatch_bolt_effects,
    },
    breaker::components::Breaker,
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
};

fn spawn_in_world(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        f(&mut commands)
    };
    queue.apply(world);
    entity
}

/// Helper: creates a minimal `BoltDefinition` with the given effects.
fn make_bolt_def(name: &str, effects: Vec<RootEffect>) -> BoltDefinition {
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

// ---- Behavior 11: Mixed targets dispatched correctly (Aegis-style bolt definition) ----

#[test]
fn dispatch_handles_mixed_targets_aegis_style() {
    let def = make_bolt_def(
        "AegisBolt",
        vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::EarlyBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::LateBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                }],
            },
        ],
    );
    let mut app = test_app_with_dispatch(def);
    let breaker_def = crate::breaker::definition::BreakerDefinition::default();
    let breaker = spawn_in_world(app.world_mut(), |commands| {
        Breaker::builder()
            .definition(&breaker_def)
            .headless()
            .primary()
            .spawn(commands)
    });
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

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "bolt should have StagedEffects inserted"
    );
}

#[test]
fn dispatch_mixed_targets_no_breaker_entity_only_bolt_effects_dispatched() {
    let def = make_bolt_def(
        "AegisBolt",
        vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::BoltLost,
                    then: vec![EffectNode::Do(EffectKind::LoseLife)],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
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

// ---- Behavior 13: BoundEffects and StagedEffects inserted on target entities that lack them ----

#[test]
fn dispatch_inserts_bound_effects_and_staged_effects_when_absent() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );
    let mut app = test_app_with_dispatch(def);
    // Spawn bolt with Bolt marker only -- no BoundEffects, no StagedEffects
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

    let staged = app
        .world()
        .get::<StagedEffects>(bolt)
        .expect("StagedEffects should be inserted on bolt");
    assert_eq!(
        staged.0.len(),
        0,
        "newly inserted StagedEffects should be empty"
    );
}

#[test]
fn dispatch_inserts_staged_effects_when_bound_effects_present_but_staged_absent() {
    let def = make_bolt_def(
        TEST_BOLT_NAME,
        vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
    );

    let mut app = test_app_with_dispatch(def);
    // Spawn bolt WITH BoundEffects but WITHOUT StagedEffects
    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BoundEffects(vec![(
                "prior_chip".to_owned(),
                EffectNode::When {
                    trigger: Trigger::Bump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
                },
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

    assert!(
        app.world().get::<StagedEffects>(bolt).is_some(),
        "StagedEffects should be inserted even though only BoundEffects was present initially"
    );
}
