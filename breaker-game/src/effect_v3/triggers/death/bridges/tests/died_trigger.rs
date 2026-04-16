use bevy::prelude::*;

use super::helpers::{
    TestCellDestroyedMessages, cell_death_test_app, death_speed_tree, destroyed_cell,
};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig,
        stacking::EffectStack,
        storage::{BoundEffects, StagedEffects},
        types::{EntityKind, Trigger},
    },
    prelude::*,
};

// -- Behavior 19: Died trigger unaffected by EntityKind::Any changes --

#[test]
fn died_trigger_fires_on_victim_regardless_of_any_changes() {
    let mut app = cell_death_test_app();

    let victim_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![death_speed_tree(
            "chip_a",
            Trigger::Died,
            1.5,
        )]))
        .id();

    let bolt_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        victim_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(victim_entity)
        .expect("victim should have EffectStack from Died trigger");
    assert_eq!(
        stack.len(),
        1,
        "Died (local, no EntityKind) must still fire on the victim"
    );
}

// -- Behavior 20: StagedEffects path — Died (local dispatch) walks staged first --

#[test]
fn died_trigger_fires_staged_entry_and_consumes_it_entry_specifically() {
    // Verifies the death bridge's local-dispatch staged path:
    // `walk_staged_effects` runs before `walk_bound_effects`, fires the
    // staged entry via `commands.remove_staged_effect` (entry-specific,
    // BoundEffects untouched). Regression guard: a rewire of the bridge
    // that skipped `walk_staged_effects` would silently drop staged
    // entries on death events.
    let mut app = cell_death_test_app();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let staged_fire_tree = death_speed_tree("chip_a", Trigger::Died, 1.5).1;

    let victim_entity = app
        .world_mut()
        .spawn((
            BoundEffects(vec![]),
            StagedEffects(vec![("chip_a".to_string(), staged_fire_tree)]),
        ))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        victim_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(victim_entity)
        .expect("staged When(Died, Fire) must have fired on the victim");
    assert_eq!(stack.len(), 1);

    // Entry-specific consume removed the staged entry from StagedEffects
    // but left BoundEffects untouched.
    let staged = app.world().get::<StagedEffects>(victim_entity).unwrap();
    assert!(staged.0.is_empty(), "staged entry should be consumed");
    let bound = app.world().get::<BoundEffects>(victim_entity).unwrap();
    assert!(bound.0.is_empty(), "BoundEffects must not be touched");
}

// -- Behavior 21: StagedEffects path — DeathOccurred(Any) global dispatch walks staged first --

#[test]
fn death_occurred_any_fires_staged_entries_on_all_entities() {
    let mut app = cell_death_test_app();
    let cell_entity = app.world_mut().spawn_empty().id();
    let bolt_entity = app.world_mut().spawn_empty().id();
    let staged_tree = death_speed_tree("chip_a", Trigger::DeathOccurred(EntityKind::Any), 1.5).1;

    let listener = app
        .world_mut()
        .spawn((
            BoundEffects(vec![]),
            StagedEffects(vec![("chip_a".to_string(), staged_tree)]),
        ))
        .id();

    app.insert_resource(TestCellDestroyedMessages(vec![destroyed_cell(
        cell_entity,
        Some(bolt_entity),
    )]));

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(listener)
        .expect("global staged entry should fire on DeathOccurred(Any)");
    assert_eq!(stack.len(), 1);

    let staged = app.world().get::<StagedEffects>(listener).unwrap();
    assert!(staged.0.is_empty(), "staged entry should be consumed");
}
