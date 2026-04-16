use bevy::prelude::*;

use super::helpers::{TestImpactMessages, bridge_test_app, impact_occurred_speed_tree};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::EntityKind,
    },
    prelude::*,
};

// -- Behavior 1: ImpactOccurred(Any) fires on all entities for BoltImpactCell --

#[test]
fn impact_occurred_any_fires_on_all_entities_for_bolt_impact_cell() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    let entity_a = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    let entity_b = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_b",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        bolt_cell: vec![BoltImpactCell {
            bolt:               bolt_entity,
            cell:               cell_entity,
            impact_normal:      Vec2::ZERO,
            piercing_remaining: 0,
        }],
        ..default()
    });

    tick(&mut app);

    let stack_a = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_a)
        .expect("entity_a should have EffectStack after ImpactOccurred(Any)");
    assert_eq!(stack_a.len(), 1, "entity_a should have 1 effect entry");

    let stack_b = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_b)
        .expect("entity_b should have EffectStack after ImpactOccurred(Any)");
    assert_eq!(stack_b.len(), 1, "entity_b should have 1 effect entry");
}

// -- Behavior 1 edge case: specific kind also fires alongside Any --

#[test]
fn impact_occurred_specific_kind_fires_alongside_any_for_bolt_impact_cell() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    // Entity with both Any and Cell gates
    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            impact_occurred_speed_tree("chip_any", EntityKind::Any, 1.5),
            impact_occurred_speed_tree("chip_cell", EntityKind::Cell, 2.0),
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        bolt_cell: vec![BoltImpactCell {
            bolt:               bolt_entity,
            cell:               cell_entity,
            impact_normal:      Vec2::ZERO,
            piercing_remaining: 0,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Any and Cell gates should fire, yielding 2 entries"
    );
}

// -- Behavior 2: ImpactOccurred(Any) fires for BoltImpactWall --

#[test]
fn impact_occurred_any_fires_for_bolt_impact_wall() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let wall_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Any,
            2.0,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        bolt_wall: vec![BoltImpactWall {
            bolt: bolt_entity,
            wall: wall_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after BoltImpactWall + Any gate");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 2 edge case: both Any and Wall gates fire --

#[test]
fn impact_occurred_any_and_wall_both_fire_for_bolt_impact_wall() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let wall_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            impact_occurred_speed_tree("chip_any", EntityKind::Any, 2.0),
            impact_occurred_speed_tree("chip_wall", EntityKind::Wall, 3.0),
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        bolt_wall: vec![BoltImpactWall {
            bolt: bolt_entity,
            wall: wall_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(stack.len(), 2, "Both Any and Wall gates should fire");
}

// -- Behavior 3: ImpactOccurred(Any) fires for BoltImpactBreaker --

#[test]
fn impact_occurred_any_fires_for_bolt_impact_breaker() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        bolt_breaker: vec![BoltImpactBreaker {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after BoltImpactBreaker + Any gate");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 3 edge case: no bound effects entities — no panic --

#[test]
fn impact_occurred_any_no_bound_effects_entities_is_no_op() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestImpactMessages {
        bolt_breaker: vec![BoltImpactBreaker {
            bolt:    bolt_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    // No entities with BoundEffects — should not panic
    tick(&mut app);
}
