//! Tests for `RemoveChainsCommand`.

use bevy::prelude::*;

use crate::effect::{core::*, triggers::evaluate::system::RemoveChainsCommand};

#[test]
fn remove_chains_command_removes_matching_chain_from_bound_effects() {
    let mut world = World::new();

    let chain_a = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };
    let chain_b = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    };
    let chain_c = EffectNode::Until {
        trigger: Trigger::TimeExpires(5.0),
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
    };

    let entity = world
        .spawn(BoundEffects(vec![
            ("chip_a".to_string(), chain_a.clone()),
            ("chip_b".to_string(), chain_b.clone()),
            ("chip_c".to_string(), chain_c.clone()),
        ]))
        .id();

    RemoveChainsCommand {
        entity,
        chains: vec![chain_a],
    }
    .apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should have 2 entries after removing the matching chain"
    );
    assert_eq!(bound.0[0].0, "chip_b");
    assert_eq!(bound.0[0].1, chain_b);
    assert_eq!(bound.0[1].0, "chip_c");
    assert_eq!(bound.0[1].1, chain_c);
}

#[test]
fn remove_chains_command_with_non_matching_chain_retains_all_entries() {
    let mut world = World::new();

    let chain_a = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };
    let chain_b = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    };
    let chain_c = EffectNode::Until {
        trigger: Trigger::TimeExpires(5.0),
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
    };

    let entity = world
        .spawn(BoundEffects(vec![
            ("chip_a".to_string(), chain_a),
            ("chip_b".to_string(), chain_b),
            ("chip_c".to_string(), chain_c),
        ]))
        .id();

    // Try to remove a chain that does not exist in BoundEffects
    let non_existent = EffectNode::When {
        trigger: Trigger::BoltLost,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(99.0))],
    };
    RemoveChainsCommand {
        entity,
        chains: vec![non_existent],
    }
    .apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "All 3 entries should be retained when no chain matches"
    );
}

#[test]
fn remove_chains_command_on_entity_without_bound_effects_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    RemoveChainsCommand {
        entity,
        chains: vec![EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }],
    }
    .apply(&mut world);
    // no panic = pass
}
