use bevy::prelude::*;
use rantzsoft_physics2d::aabb::Aabb2D;
use rantzsoft_spatial2d::components::{
    BaseSpeed, MaxSpeed, MinAngleHorizontal, MinAngleVertical, MinSpeed, Scale2D,
};

use super::helpers::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseDamage, BoltDefinitionRef},
        definition::BoltDefinition,
    },
    effect::{BoundEffects, EffectKind, EffectNode, RootEffect, StagedEffects, Target, Trigger},
    shared::size::BaseRadius,
};

// ---- Behavior 26: Hot reload rebuilds definition-sourced BoundEffects, preserves chip-sourced ----

/// Seeds a 2-effect definition (`PerfectBumped` + `EarlyBumped`), spawns a bolt with matching
/// definition-sourced `BoundEffects` plus a chip-sourced piercing entry. Returns the bolt entity.
fn seed_two_effect_bolt_with_chip(app: &mut App) -> Entity {
    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![
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
        ],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(app, initial);

    app.world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            BoundEffects(vec![
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                ),
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::EarlyBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.1 })],
                    },
                ),
                (
                    "piercing_chip".to_owned(),
                    EffectNode::When {
                        trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
                    },
                ),
            ]),
            StagedEffects::default(),
        ))
        .id()
}

#[test]
fn hot_reload_rebuilds_definition_sourced_bound_effects_preserves_chip_sourced() {
    let mut app = test_app();
    let bolt = seed_two_effect_bolt_with_chip(&mut app);

    // Update definition: remove old 2 effects, add 1 new one
    let updated = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::LateBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "should have 2 entries: 1 chip-sourced (preserved) + 1 definition-sourced (new); got {}",
        bound.0.len()
    );

    // The chip-sourced entry should be preserved
    let chip_entries: Vec<_> = bound
        .0
        .iter()
        .filter(|(name, _)| !name.is_empty())
        .collect();
    assert_eq!(
        chip_entries.len(),
        1,
        "should have exactly 1 chip-sourced entry"
    );
    assert_eq!(chip_entries[0].0, "piercing_chip");

    // The definition-sourced entry should be the new one (LateBumped)
    let def_entries: Vec<_> = bound.0.iter().filter(|(name, _)| name.is_empty()).collect();
    assert_eq!(
        def_entries.len(),
        1,
        "should have exactly 1 definition-sourced entry (new)"
    );
    assert!(matches!(
        &def_entries[0].1,
        EffectNode::When {
            trigger: Trigger::LateBumped,
            ..
        }
    ));
}

#[test]
fn hot_reload_empty_definition_effects_clears_definition_entries_keeps_chip() {
    let mut app = test_app();

    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(&mut app, initial);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            BoundEffects(vec![
                (
                    String::new(),
                    EffectNode::When {
                        trigger: Trigger::PerfectBumped,
                        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                    },
                ),
                (
                    "piercing_chip".to_owned(),
                    EffectNode::When {
                        trigger: Trigger::Impacted(crate::effect::ImpactTarget::Bolt),
                        then: vec![EffectNode::Do(EffectKind::Piercing(1))],
                    },
                ),
            ]),
            StagedEffects::default(),
        ))
        .id();

    // Update definition to empty effects
    let updated = make_bolt_def(TEST_BOLT_NAME); // effects: vec![]
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "should have only the chip-sourced entry remaining"
    );
    assert_eq!(&bound.0[0].0, "piercing_chip");
}

#[test]
fn hot_reload_no_chip_sourced_entries_clears_all_then_adds_new() {
    let mut app = test_app();

    let initial = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    seed_and_flush(&mut app, initial);

    let bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BoltDefinitionRef(TEST_BOLT_NAME.to_owned()),
            BaseSpeed(720.0),
            MinSpeed(360.0),
            MaxSpeed(1440.0),
            BaseRadius(14.0),
            BoltBaseDamage(10.0),
            Scale2D { x: 14.0, y: 14.0 },
            Aabb2D::new(Vec2::ZERO, Vec2::new(14.0, 14.0)),
            MinAngleHorizontal(5.0_f32.to_radians()),
            MinAngleVertical(5.0_f32.to_radians()),
            // Only definition-sourced entries (no chip-sourced)
            BoundEffects(vec![(
                String::new(),
                EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            )]),
            StagedEffects::default(),
        ))
        .id();

    // Update definition with different effect
    let updated = BoltDefinition {
        name: TEST_BOLT_NAME.to_owned(),
        base_speed: 720.0,
        min_speed: 360.0,
        max_speed: 1440.0,
        radius: 14.0,
        base_damage: 10.0,
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::LateBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
            }],
        }],
        color_rgb: [6.0, 5.0, 0.5],
        min_angle_horizontal: 5.0,
        min_angle_vertical: 5.0,
        min_radius: None,
        max_radius: None,
    };
    mutate_registry(&mut app, updated);
    app.update();

    let bound = app.world().get::<BoundEffects>(bolt).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "should have 1 entry (old cleared, new added)"
    );
    assert!(
        bound.0[0].0.is_empty(),
        "entry should be definition-sourced (empty chip name)"
    );
    assert!(matches!(
        &bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::LateBumped,
            ..
        }
    ));
}
