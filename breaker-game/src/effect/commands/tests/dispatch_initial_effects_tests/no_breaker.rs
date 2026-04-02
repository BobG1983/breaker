//! Tests for graceful no-op when no breaker exists (behavior 9).

use super::helpers::*;

// ── Behavior 9: No breaker exists -- AllBolts/AllCells/AllWalls graceful no-op ──
// These are true no-ops: no crash, no effects dispatched. To ensure the test
// fails with the stub, we also dispatch a Bolt-targeted effect in the same call
// and verify the PrimaryBolt was processed.

#[test]
fn no_breaker_all_bolts_graceful_noop_bolt_still_processed() {
    let mut world = World::new();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // PrimaryBolt should get the Bolt-targeted When (fails with stub)
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllBolts deferred is silently dropped, but Bolt target should still work"
    );
}

#[test]
fn no_breaker_all_cells_graceful_noop_breaker_absent_no_panic() {
    // Edge case: no breaker, AllCells target. Must not panic.
    // Combined with a Bolt target for a positive assertion.
    let mut world = World::new();
    let _cell = world.spawn((Cell, BoundEffects::default())).id();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllCells,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllCells deferred dropped, but Bolt target should still work"
    );
}

#[test]
fn no_breaker_all_walls_graceful_noop_bolt_still_processed() {
    let mut world = World::new();
    let _wall = world.spawn((Wall, BoundEffects::default())).id();
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::AllWalls,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "No breaker -> AllWalls deferred dropped, but Bolt target should still work"
    );
}
