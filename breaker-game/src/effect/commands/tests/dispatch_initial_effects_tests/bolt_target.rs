use bevy::{ecs::world::CommandQueue, prelude::*};

// Tests for bolt target dispatch (behaviors 3, 15).
use super::helpers::*;

fn spawn_in_world(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        f(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Behavior 3: Bolt target dispatches to PrimaryBolt only ───────────────

#[test]
fn bolt_target_dispatches_to_primary_bolt_only() {
    let mut world = World::new();
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
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBumped,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_bound = world
        .get::<BoundEffects>(primary)
        .expect("PrimaryBolt should have BoundEffects");
    assert_eq!(
        primary_bound.0.len(),
        1,
        "PrimaryBolt should have 1 BoundEffects entry"
    );
    assert_eq!(
        primary_bound.0[0].0, "",
        "Chip name should be empty string for source_chip None"
    );

    let extra_bound = world
        .get::<BoundEffects>(extra)
        .expect("ExtraBolt should have BoundEffects");
    assert!(
        extra_bound.0.is_empty(),
        "ExtraBolt should have 0 BoundEffects entries (Bolt target -> PrimaryBolt only)"
    );
}

#[test]
fn bolt_target_no_primary_bolt_but_breaker_still_processed() {
    // Bolt target with no PrimaryBolt is a no-op for bolt, but a co-dispatched
    // Breaker target must still work. Tests both no-op bolt + positive breaker.
    let mut world = World::new();
    let _extra = world
        .spawn((
            Bolt,
            ExtraBolt,
            BoundEffects::default(),
            StagedEffects::default(),
        ))
        .id();
    let def = BreakerDefinition::default();
    let breaker = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });
    world
        .entity_mut(breaker)
        .insert((BoundEffects::default(), StagedEffects::default()));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::When {
                    trigger: Trigger::PerfectBumped,
                    then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
                }],
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

    // Breaker must still be processed even though Bolt target was a no-op
    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker should have 1 BoundEffects entry even though Bolt target had no PrimaryBolt"
    );
}

// ── Behavior 15: Bolt target with bare Do fires on PrimaryBolt immediately ──

#[test]
fn bolt_target_bare_do_fires_on_primary_bolt() {
    let mut world = World::new();
    let primary = world
        .spawn((Bolt, PrimaryBolt, ActiveDamageBoosts(vec![])))
        .id();
    let extra = world
        .spawn((Bolt, ExtraBolt, ActiveDamageBoosts(vec![])))
        .id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let primary_boosts = world
        .get::<ActiveDamageBoosts>(primary)
        .expect("PrimaryBolt should have ActiveDamageBoosts");
    assert_eq!(
        primary_boosts.0,
        vec![1.5],
        "Do(DamageBoost(1.5)) should fire on PrimaryBolt"
    );

    let extra_boosts = world
        .get::<ActiveDamageBoosts>(extra)
        .expect("ExtraBolt should have ActiveDamageBoosts");
    assert!(
        extra_boosts.0.is_empty(),
        "ExtraBolt should NOT have DamageBoost fired (Bolt target -> PrimaryBolt only)"
    );
}

#[test]
fn bolt_target_do_fires_on_only_bolt_when_it_is_primary() {
    let mut world = World::new();
    let primary = world
        .spawn((Bolt, PrimaryBolt, ActiveDamageBoosts(vec![])))
        .id();

    DispatchInitialEffects {
        effects: vec![RootEffect::On {
            target: Target::Bolt,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let boosts = world
        .get::<ActiveDamageBoosts>(primary)
        .expect("PrimaryBolt should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![1.5],
        "Single PrimaryBolt should receive the Do effect"
    );
}

#[test]
fn bolt_target_do_no_bolts_alongside_breaker() {
    // Zero bolts -> Bolt target is no-op. Breaker target must still work.
    let mut world = World::new();
    let def = BreakerDefinition::default();
    let breaker = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });
    world.entity_mut(breaker).insert((
        BoundEffects::default(),
        StagedEffects::default(),
        ActiveDamageBoosts(vec![]),
    ));

    DispatchInitialEffects {
        effects: vec![
            RootEffect::On {
                target: Target::Bolt,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.5))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Breaker must still be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do should fire even with zero bolts for Bolt target"
    );
}
