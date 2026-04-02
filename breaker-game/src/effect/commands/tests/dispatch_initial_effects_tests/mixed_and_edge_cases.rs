//! Tests for empty effects, empty then, and multiple root effects (behaviors 12-13).

use super::helpers::*;

// ── Behavior 12: Empty effects list is a no-op ──────────────────────────
// An empty effects list means nothing to dispatch -- the stub already does
// nothing. To guarantee RED failure, we dispatch an empty list alongside a
// non-empty list and assert the non-empty one was processed.

#[test]
fn empty_effects_list_alongside_real_effect() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));

    // First call: empty list (should be a no-op)
    DispatchInitialEffects {
        effects: vec![],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert!(
        bound.0.is_empty(),
        "Empty effects list should not add any BoundEffects entries"
    );

    // Second call: real effect (must be processed -- fails with stub)
    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "Non-empty effects list should add 1 BoundEffects entry"
    );
}

#[test]
fn on_with_empty_then_alongside_real_effect() {
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = world
        .spawn(
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build(),
        )
        .id();
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "On(Breaker) with empty then should add 0 entries, but the second On should add 1"
    );
}

// ── Behavior 13: Multiple RootEffects with different targets all processed ──

#[test]
fn multiple_root_effects_different_targets_all_processed() {
    let mut world = World::new();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));
    let primary = world
        .spawn((
            Bolt,
            PrimaryBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Breaker,
                then: vec![
                    EffectNode::Do(EffectKind::DamageBoost(2.0)),
                    EffectNode::When {
                        trigger: Trigger::BoltLost,
                        then: vec![EffectNode::Do(EffectKind::LoseLife)],
                    },
                ],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker: Do fired, When stored
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker's DamageBoost(2.0) should fire immediately"
    );
    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry (When(BoltLost))"
    );

    // PrimaryBolt: When stored
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry (When(PerfectBumped))"
    );

    // ExtraBolt: nothing
    let extra_bound = world
        .get::<BoundEffects>(extra)
        .expect("ExtraBolt should have BoundEffects");
    assert!(
        extra_bound.0.is_empty(),
        "ExtraBolt should have 0 BoundEffects entries"
    );
}

#[test]
fn three_root_effects_breaker_bolt_all_bolts() {
    let mut world = World::new();
    let breaker = world
        .spawn({
            let def = BreakerDefinition::default();
            Breaker::builder()
                .definition(&def)
                .headless()
                .primary()
                .build()
        })
        .id();
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));
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
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
            },
            RootEffect::On {
                target: Target::AllBolts,
                then: vec![EffectNode::When {
                    trigger: Trigger::Bumped,
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
                }],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker: Do fired + AllBolts deferred wrapper
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(boosts.0, vec![2.0], "Breaker's Do should fire immediately");

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry (AllBolts deferred wrapper)"
    );

    // PrimaryBolt: Bolt-targeted When only, NOT the AllBolts deferred one
    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry (Bolt target only, not AllBolts deferred)"
    );
}
