use bevy::prelude::*;

use super::system::*;
use crate::effect::core::*;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    register(&mut app);
    app
}

/// Accumulates one fixed timestep then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

// -----------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------

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

#[test]
fn overclock_pattern_when_bumped_until_do_speedboost() {
    // When(Bumped, [Until(TimeExpires(2.0), [Do(SpeedBoost(1.3))])]) in BoundEffects.
    // Step 1: Bumped trigger fires → evaluate pushes Until to StagedEffects
    // Step 2: desugar_until processes the Until from StagedEffects
    // Result: BoundEffects still has When(Bumped, [Until(...)]) (permanent).
    //         StagedEffects has When(TimeExpires(2.0), [Reverse { effects: [SpeedBoost(1.3)], chains: [] }])
    //         SpeedBoost was fired.
    //         No duplication — When(TimeExpires) only in StagedEffects, not BoundEffects.
    let mut app = test_app();

    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
    };
    let overclock = EffectNode::When {
        trigger: Trigger::Bumped,
        then: vec![until_node],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("overclock".into(), overclock.clone())]),
            StagedEffects::default(),
        ))
        .id();

    // Step 1: simulate Bumped trigger evaluation (pushes Until to StagedEffects)
    // We do this manually since we don't have a real bump message
    {
        let world = app.world_mut();
        let trigger = Trigger::Bumped;
        let bound = world.get::<BoundEffects>(entity).unwrap().clone();
        let mut staged = world.get_mut::<StagedEffects>(entity).unwrap();
        // Walk BoundEffects — When(Bumped) matches, Until child is non-Do → pushed to Staged
        for (chip_name, node) in &bound.0 {
            if let EffectNode::When { trigger: t, then } = node
                && *t == trigger
            {
                for child in then {
                    match child {
                        EffectNode::Do(_effect) => {
                            // Would fire — skip in test (no Commands available here)
                        }
                        other => {
                            staged.0.push((chip_name.clone(), other.clone()));
                        }
                    }
                }
            }
        }
    }

    // Verify: Until is now in StagedEffects, BoundEffects unchanged
    let staged_before = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(staged_before.0.len(), 1, "Until should be in StagedEffects");
    assert!(
        matches!(&staged_before.0[0].1, EffectNode::Until { .. }),
        "StagedEffects should contain the Until node"
    );
    let bound_before = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound_before.0.len(),
        1,
        "BoundEffects When(Bumped) still present (permanent)"
    );
    assert_eq!(
        bound_before.0[0].1, overclock,
        "BoundEffects entry unchanged"
    );

    // Step 2: desugar_until runs (processes Until from StagedEffects)
    tick(&mut app);

    // Verify final state
    let bound_after = app.world().get::<BoundEffects>(entity).unwrap();
    assert_eq!(
        bound_after.0.len(),
        1,
        "BoundEffects still just the When(Bumped) — no duplication"
    );
    assert_eq!(
        bound_after.0[0].1, overclock,
        "Original When(Bumped, [Until(...)]) untouched"
    );

    let staged_after = app.world().get::<StagedEffects>(entity).unwrap();
    assert_eq!(
        staged_after.0.len(),
        1,
        "StagedEffects has desugared When(TimeExpires, Reverse)"
    );
    if let EffectNode::When {
        trigger: Trigger::TimeExpires(secs),
        ref then,
    } = staged_after.0[0].1
    {
        assert!((secs - 2.0).abs() < f32::EPSILON);
        assert_eq!(then.len(), 1);
        if let EffectNode::Reverse { effects, chains } = &then[0] {
            assert_eq!(effects.len(), 1, "SpeedBoost should be in fired effects");
            assert_eq!(effects[0], EffectKind::SpeedBoost { multiplier: 1.3 });
            assert!(chains.is_empty(), "No non-Do children → no chains");
        } else {
            panic!("Expected Reverse node");
        }
    } else {
        panic!("Expected When(TimeExpires(2.0), [Reverse(...)]) in StagedEffects");
    }
}

#[test]
fn non_until_entries_unaffected() {
    // When(Bump, Do(X)) in BoundEffects alongside an Until.
    // After desugaring, the When(Bump) should still be in BoundEffects.
    let mut app = test_app();

    let regular_when = EffectNode::When {
        trigger: Trigger::Bump,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
    };
    let inner = EffectNode::When {
        trigger: Trigger::Death,
        then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
    };
    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![inner.clone()],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![
                ("chip_a".into(), regular_when.clone()),
                ("chip_b".into(), until_node),
            ]),
            StagedEffects::default(),
        ))
        .id();

    tick(&mut app);

    let bound = app.world().get::<BoundEffects>(entity).unwrap();
    // regular_when retained + non-Do child from Until pushed = 2 entries
    assert_eq!(bound.0.len(), 2, "Regular When + pushed non-Do child");

    let has_regular = bound
        .0
        .iter()
        .any(|(name, node)| name == "chip_a" && *node == regular_when);
    assert!(
        has_regular,
        "Non-Until entry When(Bump) should be retained in BoundEffects"
    );

    let has_inner = bound
        .0
        .iter()
        .any(|(name, node)| name == "chip_b" && *node == inner);
    assert!(
        has_inner,
        "Inner non-Do child from Until should be pushed to BoundEffects"
    );
}

// -- Section K: EffectSourceChip threading through desugar_until ───────────────────

use crate::effect::effects::speed_boost::ActiveSpeedBoosts;

#[test]
fn desugar_until_threads_chip_name_as_source_chip_to_fire_effect() {
    // Until(TimeExpires(2.0), [Do(SpeedBoost(1.3))]) with chip_name "overclock"
    // SpeedBoost ignores source_chip, but verifying plumbing via ActiveSpeedBoosts
    let mut app = test_app();

    let until_node = EffectNode::Until {
        trigger: Trigger::TimeExpires(2.0),
        then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.3 })],
    };

    let entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![("overclock".into(), until_node)]),
            StagedEffects::default(),
            ActiveSpeedBoosts(vec![]),
        ))
        .id();

    tick(&mut app);

    let boosts = app.world().get::<ActiveSpeedBoosts>(entity).unwrap();
    assert!(
        boosts.0.contains(&1.3),
        "SpeedBoost(1.3) should have been fired — ActiveSpeedBoosts should contain 1.3, got {:?}",
        boosts.0
    );
}
