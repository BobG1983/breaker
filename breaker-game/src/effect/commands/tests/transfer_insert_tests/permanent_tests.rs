use bevy::prelude::*;

use super::super::super::ext::*;
use crate::effect::core::*;

#[test]
fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_when_child() {
    let mut world = World::new();
    let entity = world.spawn(Name::new("test")).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "aegis".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should contain exactly 1 entry"
    );
    assert_eq!(bound.0[0].0, "aegis");
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }
    );

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted as default alongside BoundEffects");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be empty (default)"
    );
}

#[test]
fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_on_child() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "redirect".to_string(),
        children: vec![EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should contain exactly 1 On entry"
    );
    assert_eq!(bound.0[0].0, "redirect");
    assert_eq!(
        bound.0[0].1,
        EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }
    );

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted as default");
    assert!(staged.0.is_empty());
}

#[test]
fn transfer_permanent_stores_on_child_with_empty_then() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "redirect".to_string(),
        children: vec![EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "On node with empty then should still be stored"
    );
    assert_eq!(
        bound.0[0].1,
        EffectNode::On {
            target: Target::Bolt,
            permanent: false,
            then: vec![],
        }
    );
}

#[test]
fn transfer_permanent_appends_to_existing_bound_effects() {
    let mut world = World::new();
    let existing_entry = (
        "old_chip".to_string(),
        EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        },
    );
    let entity = world
        .spawn((BoundEffects(vec![existing_entry]), StagedEffects::default()))
        .id();

    let cmd = TransferCommand {
        entity,
        chip_name: "new_chip".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain original + new entry"
    );
    assert_eq!(bound.0[0].0, "old_chip", "original entry at index 0");
    assert_eq!(bound.0[1].0, "new_chip", "new entry appended at index 1");
    assert_eq!(
        bound.0[1].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
        }
    );

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert!(
        staged.0.is_empty(),
        "StagedEffects should remain empty after permanent transfer"
    );
}

#[test]
fn transfer_permanent_inserts_bound_effects_when_absent_and_stores_until_child() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "aegis".to_string(),
        children: vec![EffectNode::Until {
            trigger: Trigger::TimeExpires(5.0),
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        permanent: true,
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "Until child should be stored in BoundEffects"
    );
    assert_eq!(bound.0[0].0, "aegis");
    assert_eq!(
        bound.0[0].1,
        EffectNode::Until {
            trigger: Trigger::TimeExpires(5.0),
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }
    );
}
