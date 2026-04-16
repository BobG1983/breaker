use bevy::prelude::*;

use super::helpers::{TestImpactMessages, bridge_test_app, impact_occurred_speed_tree};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::EntityKind,
    },
    prelude::*,
};

// -- Behavior 4: ImpactOccurred(Any) fires for BreakerImpactCell --

#[test]
fn impact_occurred_any_fires_for_breaker_impact_cell() {
    let mut app = bridge_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        breaker_cell: vec![BreakerImpactCell {
            breaker: breaker_entity,
            cell:    cell_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after BreakerImpactCell + Any gate");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 4 edge case: Breaker-specific also fires alongside Any --

#[test]
fn impact_occurred_breaker_specific_fires_alongside_any_for_breaker_impact_cell() {
    let mut app = bridge_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            impact_occurred_speed_tree("chip_any", EntityKind::Any, 1.5),
            impact_occurred_speed_tree("chip_breaker", EntityKind::Breaker, 2.0),
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        breaker_cell: vec![BreakerImpactCell {
            breaker: breaker_entity,
            cell:    cell_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack");
    assert_eq!(stack.len(), 2, "Both Any and Breaker gates should fire");
}

// -- Behavior 5: ImpactOccurred(Any) fires once per collision, not per participant --

#[test]
fn impact_occurred_any_fires_once_per_collision_event() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
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

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after ImpactOccurred(Any)");
    assert_eq!(
        stack.len(),
        1,
        "EntityKind::Any should fire once per collision, not once per participant"
    );
}

// -- Behavior 5 edge case: two collisions in one frame fires twice --

#[test]
fn impact_occurred_any_fires_once_per_collision_two_events() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();
    let wall_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
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
        .expect("entity should have EffectStack after two collision events");
    assert_eq!(
        stack.len(),
        2,
        "Two collision events should fire ImpactOccurred(Any) twice"
    );
}
