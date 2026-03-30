//! Integration test for the overclock pattern: `When(Bumped, [Until(TimeExpires, [Do(SpeedBoost)])])`.

use bevy::prelude::*;

use super::*;

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
