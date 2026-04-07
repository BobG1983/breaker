use bevy::prelude::*;

use crate::effect::{commands::ext::*, core::*, effects::damage_boost::ActiveDamageBoosts};

// -- Section III: A1-A3 coverage tests ────────────────────────────────

#[test]
fn transfer_permanent_stores_multiple_mixed_non_do_children_in_bound_effects() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let when_node = EffectNode::When {
        trigger: Trigger::PerfectBump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(5.0),
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    };
    let once_node = EffectNode::Once(vec![EffectNode::Do(EffectKind::DamageBoost(1.0))]);

    let cmd = TransferCommand {
        entity,
        chip_name: "aegis".to_string(),
        children: vec![when_node.clone(), until_node.clone(), once_node.clone()],
        permanent: true,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted");
    assert_eq!(
        bound.0.len(),
        3,
        "BoundEffects should contain When, Until, and Once entries"
    );
    assert_eq!(bound.0[0].0, "aegis");
    assert_eq!(bound.0[0].1, when_node);
    assert_eq!(bound.0[1].0, "aegis");
    assert_eq!(bound.0[1].1, until_node);
    assert_eq!(bound.0[2].0, "aegis");
    assert_eq!(bound.0[2].1, once_node);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted as default");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should have 0 entries for permanent transfer"
    );
}

#[test]
fn transfer_non_permanent_stores_mixed_non_do_children_in_staged_and_fires_do() {
    let mut world = World::new();
    let entity = world.spawn(ActiveDamageBoosts(vec![])).id();

    let when_node = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(3.0),
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
    };
    let do_node = EffectNode::Do(EffectKind::DamageBoost(3.0));

    let cmd = TransferCommand {
        entity,
        chip_name: "flux".to_string(),
        children: vec![when_node.clone(), until_node.clone(), do_node],
        permanent: false,
        context: TriggerContext::default(),
    };
    cmd.apply(&mut world);

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted");
    assert_eq!(
        staged.0.len(),
        2,
        "StagedEffects should contain When and Until entries"
    );
    assert_eq!(staged.0[0].0, "flux");
    assert_eq!(staged.0[0].1, when_node);
    assert_eq!(staged.0[1].0, "flux");
    assert_eq!(staged.0[1].1, until_node);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted as default");
    assert!(
        bound.0.is_empty(),
        "BoundEffects should have 0 entries for non-permanent transfer"
    );

    // Do child should have been fired immediately
    let boosts = world.get::<ActiveDamageBoosts>(entity).unwrap();
    assert_eq!(
        boosts.0,
        vec![3.0],
        "Do(DamageBoost(3.0)) should have fired, populating ActiveDamageBoosts"
    );
}

#[test]
fn push_bound_effects_inserts_components_when_absent_then_appends() {
    let mut world = World::new();
    let entity = world.spawn(Name::new("bare")).id();

    let effects = vec![
        (
            "chip_a".to_string(),
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ),
        (
            "chip_b".to_string(),
            EffectNode::Until {
                trigger: Trigger::TimeExpires(3.0),
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
        ),
    ];

    PushBoundEffects { entity, effects }.apply(&mut world);

    let bound = world
        .get::<BoundEffects>(entity)
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        2,
        "BoundEffects should contain exactly 2 entries"
    );
    assert_eq!(bound.0[0].0, "chip_a");
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }
    );
    assert_eq!(bound.0[1].0, "chip_b");
    assert_eq!(
        bound.0[1].1,
        EffectNode::Until {
            trigger: Trigger::TimeExpires(3.0),
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        }
    );

    let staged = world
        .get::<StagedEffects>(entity)
        .expect("StagedEffects should be inserted as default by ensure_effect_components");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be empty (default)"
    );
}

#[test]
fn push_bound_effects_on_despawned_entity_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.despawn(entity);

    PushBoundEffects {
        entity,
        effects: vec![(
            "chip_a".to_string(),
            EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        )],
    }
    .apply(&mut world);
    // no panic = pass
}

#[test]
fn push_bound_effects_appends_to_existing_bound_effects() {
    let mut world = World::new();
    let existing = vec![(
        "existing".to_string(),
        EffectNode::When {
            trigger: Trigger::BoltLost,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
        },
    )];
    let entity = world
        .spawn((BoundEffects(existing), StagedEffects::default()))
        .id();

    PushBoundEffects {
        entity,
        effects: vec![
            (
                "chip_a".to_string(),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
                },
            ),
            (
                "chip_b".to_string(),
                EffectNode::Until {
                    trigger: Trigger::TimeExpires(3.0),
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ),
        ],
    }
    .apply(&mut world);

    let bound = world.get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        3,
        "BoundEffects should contain 1 existing + 2 appended = 3 entries"
    );
    assert_eq!(bound.0[0].0, "existing", "original entry at index 0");
    assert_eq!(bound.0[1].0, "chip_a", "first appended entry at index 1");
    assert_eq!(bound.0[2].0, "chip_b", "second appended entry at index 2");

    let staged = world.get::<StagedEffects>(entity).unwrap();
    assert!(
        staged.0.is_empty(),
        "StagedEffects should remain empty after push_bound_effects"
    );
}
