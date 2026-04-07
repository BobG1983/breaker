use bevy::prelude::*;

use crate::effect::{commands::ext::*, core::*};

#[test]
fn transfer_non_permanent_inserts_staged_effects_when_absent_and_stores_when_child() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "flux".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when absent");
    assert_eq!(
        staged.0.len(),
        1,
        "StagedEffects should contain exactly 1 entry"
    );
    assert_eq!(staged.0[0].0, "flux");
    assert_eq!(
        staged.0[0].1,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted as default alongside StagedEffects");
    assert!(bound.0.is_empty(), "BoundEffects should be empty (default)");
}

#[test]
fn transfer_non_permanent_stores_multiple_non_do_children() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "flux".to_string(),
        children: vec![
            EffectNode::When {
                trigger: Trigger::Bump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
            EffectNode::Until {
                trigger: Trigger::TimeExpires(3.0),
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            },
        ],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when absent");
    assert_eq!(
        staged.0.len(),
        2,
        "StagedEffects should contain both When and Until entries"
    );
    assert_eq!(staged.0[0].0, "flux");
    assert_eq!(staged.0[1].0, "flux");
}

#[test]
fn transfer_non_permanent_inserts_staged_effects_when_absent_and_stores_once_child() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "second_wind".to_string(),
        children: vec![EffectNode::Once(vec![EffectNode::Do(
            EffectKind::SecondWind,
        )])],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when absent");
    assert_eq!(
        staged.0.len(),
        1,
        "Once child should be stored in StagedEffects"
    );
    assert_eq!(staged.0[0].0, "second_wind");
    assert_eq!(
        staged.0[0].1,
        EffectNode::Once(vec![EffectNode::Do(EffectKind::SecondWind)])
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted as default alongside StagedEffects");
    assert!(bound.0.is_empty());
}

#[test]
fn transfer_non_permanent_stores_once_child_with_empty_children() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "second_wind".to_string(),
        children: vec![EffectNode::Once(vec![])],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when absent");
    assert_eq!(
        staged.0.len(),
        1,
        "Once with empty children should still be stored"
    );
}
