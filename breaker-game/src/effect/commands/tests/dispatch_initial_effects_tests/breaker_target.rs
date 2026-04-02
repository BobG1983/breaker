//! Tests for breaker target dispatch (behaviors 1-2).

use super::helpers::*;

// ── Behavior 1: Breaker target with Do effect fires immediately ──────────

#[test]
fn breaker_target_do_effect_fires_immediately() {
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
    world.entity_mut(breaker).insert(ActiveDamageBoosts(vec![]));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do(DamageBoost(2.0)) should fire immediately on Breaker"
    );
}

#[test]
fn breaker_target_multiple_bare_do_children_all_fire() {
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
    world.entity_mut(breaker).insert(ActiveDamageBoosts(vec![]));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::Do(EffectKind::DamageBoost(3.0)),
            ],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0.len(),
        2,
        "Both Do children should fire immediately on Breaker"
    );
}

// ── Behavior 2: Breaker target with When effect pushes to BoundEffects ───

#[test]
fn breaker_target_when_effect_pushes_to_bound_effects() {
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
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
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
        "BoundEffects should have exactly 1 entry for the When node"
    );
    assert_eq!(
        bound.0[0].0, "",
        "Chip name should be empty string when source_chip is None"
    );
    assert_eq!(
        bound.0[0].1,
        EffectNode::When {
            trigger: Trigger::PerfectBump,
            then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
        },
        "Stored effect node should match the When node"
    );
}

#[test]
fn breaker_target_mixed_do_and_when_fires_do_stores_when() {
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
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![
                EffectNode::Do(EffectKind::DamageBoost(2.0)),
                EffectNode::When {
                    trigger: Trigger::PerfectBump,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                },
            ],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("ActiveDamageBoosts should be present");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Do(DamageBoost(2.0)) should fire immediately"
    );

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have exactly 1 entry (the When node)"
    );
}
