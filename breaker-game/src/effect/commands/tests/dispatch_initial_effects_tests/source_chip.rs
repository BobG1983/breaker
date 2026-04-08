use bevy::{ecs::world::CommandQueue, prelude::*};

// Tests for `source_chip` passthrough behavior (behaviors 10-11).
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

// ── Behavior 10: Source chip Some passthrough to BoundEffects entries ─────

#[test]
fn source_chip_some_passes_through_to_bound_effects() {
    let mut world = World::new();
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
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::SpeedBoost { multiplier: 1.5 })],
            }],
        }],
        source_chip: Some("overclock".to_string()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "overclock",
        "Chip name should be 'overclock' from source_chip Some('overclock')"
    );
}

#[test]
fn source_chip_special_characters_stored_as_is() {
    let mut world = World::new();
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
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::PerfectBump,
                then: vec![EffectNode::Do(EffectKind::DamageBoost(1.0))],
            }],
        }],
        source_chip: Some("flux-v2.1".to_string()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "flux-v2.1",
        "Special characters in chip name should be stored as-is"
    );
}

// ── Behavior 11: Source chip None maps to empty string ───────────────────

#[test]
fn source_chip_none_maps_to_empty_string() {
    let mut world = World::new();
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
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
        source_chip: None,
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "",
        "Source chip None should produce empty string chip name"
    );
}

#[test]
fn source_chip_some_empty_string_same_as_none() {
    let mut world = World::new();
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
        effects: vec![RootEffect::On {
            target: Target::Breaker,
            then: vec![EffectNode::When {
                trigger: Trigger::BoltLost,
                then: vec![EffectNode::Do(EffectKind::LoseLife)],
            }],
        }],
        source_chip: Some(String::new()),
    }
    .apply(&mut world);

    let bound = world
        .get::<BoundEffects>(breaker)
        .expect("BoundEffects should be present");
    assert_eq!(bound.0.len(), 1, "BoundEffects should have exactly 1 entry");
    assert_eq!(
        bound.0[0].0, "",
        "Source chip Some('') should produce empty string chip name (same as None)"
    );
}
