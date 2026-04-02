use bevy::prelude::*;

use super::super::super::ext::*;
use crate::effect::{core::*, effects::damage_boost::ActiveDamageBoosts};

#[test]
fn transfer_do_children_fire_even_without_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "amp".to_string(),
        children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do child should fire even when BoundEffects was absent"
    );

    // Insert-if-absent should happen unconditionally
    assert!(
        world.get::<BoundEffects>(entity).is_some(),
        "BoundEffects should be inserted even when only Do children exist"
    );
    assert!(
        world.get::<StagedEffects>(entity).is_some(),
        "StagedEffects should be inserted even when only Do children exist"
    );
}

#[test]
fn transfer_mixed_do_and_when_children_without_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "overclock".to_string(),
        children: vec![
            EffectNode::Do(EffectKind::DamageBoost(2.0)),
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        ],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do child should fire regardless of BoundEffects absence"
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "When child should be stored in BoundEffects"
    );
    assert_eq!(bound.0[0].0, "overclock");
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }
    );
}

#[test]
fn transfer_mixed_when_before_do_children_both_processed() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    // When comes before Do in the children vec -- order should not matter
    let cmd = TransferCommand {
        entity,
        chip_name: "overclock".to_string(),
        children: vec![
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
            EffectNode::Do(EffectKind::DamageBoost(2.0)),
        ],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do child should fire regardless of order in children vec"
    );

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "When child should be stored in BoundEffects"
    );
}

#[test]
fn transfer_permanent_stores_until_child_with_zero_duration() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let cmd = TransferCommand {
        entity,
        chip_name: "aegis".to_string(),
        children: vec![EffectNode::Until {
            trigger: Trigger::TimeExpires(0.0),
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "Until with zero duration should still be stored"
    );
}

#[test]
fn transfer_entity_with_staged_but_not_bound_inserts_bound_and_stores_permanent_child() {
    let mut world = World::new();
    let entity = world.spawn(StagedEffects::default()).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "asymmetric_a".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent (StagedEffects was present)");
    assert_eq!(
        bound.0.len(),
        1,
        "Permanent child should be stored in BoundEffects"
    );
    assert_eq!(bound.0[0].0, "asymmetric_a");
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }
    );
}

#[test]
fn transfer_entity_with_staged_preserves_existing_staged_entries_when_inserting_bound() {
    let mut world = World::new();
    let existing_staged = vec![(
        "pre_existing".to_string(),
        EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
        },
    )];
    let entity = world.spawn(StagedEffects(existing_staged)).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "asymmetric_a".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "Pre-existing StagedEffects entries must not be disturbed"
    );
    assert_eq!(staged.0[0].0, "pre_existing");
}

#[test]
fn transfer_entity_with_bound_but_not_staged_inserts_staged_and_stores_non_permanent_child() {
    let mut world = World::new();
    let entity = world.spawn(BoundEffects::default()).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "asymmetric_b".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted when absent (BoundEffects was present)");
    assert_eq!(
        staged.0.len(),
        1,
        "Non-permanent child should be stored in StagedEffects"
    );
    assert_eq!(staged.0[0].0, "asymmetric_b");
    assert_eq!(
        staged.0[0].1,
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }
    );
}

#[test]
fn transfer_entity_with_bound_preserves_existing_bound_entries_when_inserting_staged() {
    let mut world = World::new();
    let existing_bound = vec![(
        "pre_existing".to_string(),
        EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
        },
    )];
    let entity = world.spawn(BoundEffects(existing_bound)).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "asymmetric_b".to_string(),
        children: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        }],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Pre-existing BoundEffects entries must not be disturbed"
    );
    assert_eq!(bound.0[0].0, "pre_existing");
}

#[test]
fn transfer_on_despawned_entity_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.despawn(entity);

    let cmd = TransferCommand {
        entity,
        chip_name: "ghost".to_string(),
        children: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
        permanent: true,
        context: TriggerContext::default(),
    };
    // Should not panic -- the entity-not-found guard handles this
    cmd.apply(&mut world);
}
