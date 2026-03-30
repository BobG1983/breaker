use bevy::prelude::*;

use super::super::ext::*;
use crate::effect::{core::*, effects::damage_boost::ActiveDamageBoosts};

// -- Section II: TransferCommand insert-if-absent bug fix tests ──────────

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
fn transfer_do_children_fire_even_without_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let cmd = TransferCommand {
        entity,
        chip_name: "amp".to_string(),
        children: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        permanent: true,
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
    };
    // Should not panic -- the entity-not-found guard handles this
    cmd.apply(&mut world);
}
