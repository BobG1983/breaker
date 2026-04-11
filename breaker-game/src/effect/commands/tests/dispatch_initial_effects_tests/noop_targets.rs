use bevy::{ecs::world::CommandQueue, prelude::*};

// Tests for Cell and Wall target no-op behavior (behaviors 4-5).
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

// ── Behavior 4: Cell target is a no-op (no effects dispatched) ───────────

#[test]
fn cell_target_is_noop_but_breaker_target_processes() {
    // Cell target should be skipped, while a Breaker target in the same call processes.
    let mut world = World::new();
    let cell_a = world.spawn((Cell, BoundEffects::default())).id();
    let cell_b = world.spawn((Cell, BoundEffects::default())).id();
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
                target: Target::Cell,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    // Cells must remain untouched
    let bound_a = world.get::<BoundEffects>(cell_a).unwrap();
    let bound_b = world.get::<BoundEffects>(cell_b).unwrap();
    assert!(
        bound_a.0.is_empty(),
        "Cell target should be noop -- cell_a BoundEffects should be empty"
    );
    assert!(
        bound_b.0.is_empty(),
        "Cell target should be noop -- cell_b BoundEffects should be empty"
    );

    // Breaker must be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do(DamageBoost(2.0)) should fire even though Cell target was noop"
    );
}

#[test]
fn cell_target_when_children_noop_with_breaker_processed() {
    let mut world = World::new();
    let cell = world.spawn((Cell, BoundEffects::default())).id();
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
                target: Target::Cell,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::DamageBoost(1.1))],
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

    let cell_bound = world.get::<BoundEffects>(cell).unwrap();
    assert!(
        cell_bound.0.is_empty(),
        "Cell target with When children should still be noop"
    );

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker When effect should be stored even though Cell target was noop"
    );
}

// ── Behavior 5: Wall target is a no-op (no effects dispatched) ───────────

#[test]
fn wall_target_is_noop_but_breaker_target_processes() {
    let mut world = World::new();
    let wall_a = world.spawn((Wall, BoundEffects::default())).id();
    let wall_b = world.spawn((Wall, BoundEffects::default())).id();
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
                target: Target::Wall,
                then: vec![EffectNode::When {
                    trigger: Trigger::Impacted(ImpactTarget::Bolt),
                    then: vec![EffectNode::Do(EffectKind::Shockwave {
                        base_range: 32.0,
                        range_per_level: 8.0,
                        stacks: 1,
                        speed: 400.0,
                    })],
                }],
            },
            RootEffect::On {
                target: Target::Breaker,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(2.0))],
            },
        ],
        source_chip: None,
    }
    .apply(&mut world);

    let bound_a = world.get::<BoundEffects>(wall_a).unwrap();
    let bound_b = world.get::<BoundEffects>(wall_b).unwrap();
    assert!(
        bound_a.0.is_empty(),
        "Wall target should be noop -- wall_a BoundEffects should be empty"
    );
    assert!(
        bound_b.0.is_empty(),
        "Wall target should be noop -- wall_b BoundEffects should be empty"
    );

    // Breaker must be processed (fails with stub)
    let boosts = world
        .get::<ActiveDamageBoosts>(breaker)
        .expect("Breaker should have ActiveDamageBoosts");
    assert_eq!(
        boosts.0,
        vec![2.0],
        "Breaker Do should fire even though Wall target was noop"
    );
}

#[test]
fn wall_target_do_children_noop_with_breaker_processed() {
    let mut world = World::new();
    let wall = world.spawn((Wall, BoundEffects::default())).id();
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
                target: Target::Wall,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
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

    let wall_bound = world.get::<BoundEffects>(wall).unwrap();
    assert!(
        wall_bound.0.is_empty(),
        "Wall target with bare Do children should still be noop"
    );

    let breaker_bound = world
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        breaker_bound.0.len(),
        1,
        "Breaker When effect should be stored even though Wall target was noop"
    );
}
