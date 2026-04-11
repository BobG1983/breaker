//! Tests for core `Until` desugaring behavior: removal from `BoundEffects`/`StagedEffects`,
//! `Reverse` node creation, `Do` children recorded in `Reverse.effects`, non-`Do` children
//! pushed to `BoundEffects`.

use bevy::prelude::*;

use super::helpers::*;

#[test]
fn until_in_bound_removed_and_desugared() {
    // Until(TimeExpires(2.0), [non-Do child]) in BoundEffects.
    // After desugaring: BoundEffects loses the Until but gains the non-Do
    // child. StagedEffects gains When(TimeExpires(2.0), [Reverse(...)]).
    let mut app = test_app();

    let inner_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![inner_when.clone()],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("chip_a".into(), until_node)]),
            StagedEffects::default(),
        ))
        .id();

    tick(&mut app);

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    // Until removed, non-Do child pushed to BoundEffects
    assert_eq!(
        bound.0.len(),
        1,
        "Non-Do child should be pushed to BoundEffects"
    );
    assert_eq!(
        bound.0[0].1, inner_when,
        "BoundEffects entry should be the inner When(Bump, Do(DamageBoost))"
    );

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged.0.len(),
        1,
        "Desugared When+Reverse should appear in StagedEffects"
    );
    if let EffectNode::When {
        trigger: Trigger::TimeExpires(secs),
        ref then,
    } = staged.0[0].1
    {
        assert!(
            (secs - 2.0).abs() < f32::EPSILON,
            "Trigger should preserve TimeExpires(2.0), got {secs}"
        );
        assert_eq!(then.len(), 1, "Should have exactly one Reverse child");
        if let EffectNode::Reverse { effects, chains } = &then[0] {
            assert!(effects.is_empty(), "No Do children → no fired effects");
            assert_eq!(chains.len(), 1, "The non-Do child should appear in chains");
            assert_eq!(
                chains[0], inner_when,
                "Chain should be the inner When(Bump)"
            );
        } else {
            panic!("Expected Reverse node inside the desugared When");
        }
    } else {
        panic!("Expected When(TimeExpires(2.0), [Reverse(...)]) in StagedEffects");
    }
}

#[test]
fn until_in_staged_removed_and_desugared() {
    // Until(TimeExpires(2.0), [non-Do child]) starting in StagedEffects.
    // Same desugaring: Until removed, non-Do child → BoundEffects,
    // When(TimeExpires, Reverse) → StagedEffects.
    let mut app = test_app();

    let inner_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(3.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![inner_when.clone()],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects::default(),
            StagedEffects(vec![("chip_b".into(), until_node)]),
        ))
        .id();

    tick(&mut app);

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(bound.0.len(), 1, "Non-Do child pushed to BoundEffects");
    assert_eq!(bound.0[0].1, inner_when, "Should be the inner When(Bump)");

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1, "Desugared When+Reverse in StagedEffects");
    assert!(
        matches!(
            &staged.0[0].1,
            EffectNode::When {
                trigger: Trigger::TimeExpires(_),
                ..
            }
        ),
        "StagedEffects should contain the desugared When(TimeExpires, Reverse)"
    );
}

#[test]
fn do_children_recorded_in_reverse_effects() {
    // Until(TimeExpires(2.0), [Do(DamageBoost(5.0))]) — the Do child
    // should fire immediately AND appear in the Reverse node's effects list.
    // NOTE: fire_effect command will be applied during update and may panic
    // due to missing world state — this is expected RED phase behavior.
    let mut app = test_app();

    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![EffectNode::Do(EffectKind::DamageBoost(5.0))],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("chip_a".into(), until_node)]),
            StagedEffects::default(),
        ))
        .id();

    tick(&mut app);

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged.0.len(), 1);
    if let EffectNode::When { ref then, .. } = staged.0[0].1 {
        if let EffectNode::Reverse { effects, chains } = &then[0] {
            assert_eq!(effects.len(), 1, "Do child should appear in fired effects");
            assert_eq!(
                effects[0],
                EffectKind::DamageBoost(5.0),
                "Fired effect should be DamageBoost(5.0)"
            );
            assert!(chains.is_empty(), "No non-Do children → empty chains");
        } else {
            panic!("Expected Reverse node");
        }
    } else {
        panic!("Expected When node");
    }
}

#[test]
fn non_do_children_pushed_to_bound_effects() {
    // Until with When(Impact(Cell), Do(Y)) child — the When is non-Do,
    // so it should appear in BoundEffects and in Reverse.chains.
    let mut app = test_app();

    let inner = EffectNode::When {
        trigger: Trigger::Impact(ImpactTarget::Cell),
        then: vec![EffectNode::Do(EffectKind::DamageBoost(4.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(3.0),
        then: vec![inner.clone()],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("chip_a".into(), until_node)]),
            StagedEffects::default(),
        ))
        .id();

    tick(&mut app);

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound.0.len(),
        1,
        "Non-Do child should be pushed to BoundEffects"
    );
    assert_eq!(
        bound.0[0].1, inner,
        "Should be the inner When(Impact(Cell))"
    );

    let staged = app.world().get::<StagedEffects>(entity).unwrap();
    if let EffectNode::When { ref then, .. } = staged.0[0].1 {
        if let EffectNode::Reverse { effects, chains } = &then[0] {
            assert!(effects.is_empty(), "No Do children → no fired effects");
            assert_eq!(chains.len(), 1, "Non-Do child in chains");
            assert_eq!(chains[0], inner, "Chain entry should be the inner When");
        } else {
            panic!("Expected Reverse node");
        }
    } else {
        panic!("Expected When node in staged");
    }
}
