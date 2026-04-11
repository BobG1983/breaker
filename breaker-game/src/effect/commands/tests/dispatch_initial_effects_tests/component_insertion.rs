use bevy::{ecs::world::CommandQueue, prelude::*};

// Tests for BoundEffects/StagedEffects auto-insertion (behavior 14).
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

// ── Behavior 14: BoundEffects and StagedEffects inserted if absent ───────

#[test]
fn bound_effects_and_staged_effects_inserted_if_absent() {
    let mut world = World::new();
    // Spawn breaker with ONLY the marker -- no BoundEffects/StagedEffects
    let def = BreakerDefinition::default();
    let breaker = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });

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
        .expect("BoundEffects should be inserted when absent");
    assert_eq!(
        bound.0.len(),
        1,
        "BoundEffects should have 1 entry after dispatch"
    );

    let staged = world
        .get::<StagedEffects>(breaker)
        .expect("StagedEffects should be inserted when absent");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be inserted empty"
    );
}

#[test]
fn prior_bound_effects_preserved_new_entry_appended() {
    let mut world = World::new();
    let prior_entry = (
        "prior_chip".to_string(),
        EffectNode::When {
            trigger: Trigger::Bump,
            then: vec![EffectNode::Do(EffectKind::DamageBoost(0.5))],
        },
    );

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
        .insert(BoundEffects(vec![prior_entry]));

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
        2,
        "BoundEffects should have 2 entries: 1 prior + 1 new"
    );
    assert_eq!(
        bound.0[0].0, "prior_chip",
        "Prior entry should be preserved at index 0"
    );
    assert_eq!(
        bound.0[1].0, "",
        "New entry should be appended at index 1 with empty chip name"
    );

    // StagedEffects should be inserted (was absent)
    let staged = world
        .get::<StagedEffects>(breaker)
        .expect("StagedEffects should be inserted when absent");
    assert!(
        staged.0.is_empty(),
        "StagedEffects should be inserted empty"
    );
}
