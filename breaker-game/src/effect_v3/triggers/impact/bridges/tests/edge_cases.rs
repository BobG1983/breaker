use bevy::prelude::*;

use super::helpers::{TestImpactMessages, bridge_test_app, impact_occurred_speed_tree};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::EntityKind,
    },
    prelude::*,
};

// -- Behavior 6: does not fire on entities without BoundEffects --

#[test]
fn impact_occurred_any_does_not_fire_on_entities_without_bound_effects() {
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

    // entity_b has no BoundEffects
    let entity_b = app.world_mut().spawn_empty().id();

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
        .expect("entity_a (has BoundEffects) should have EffectStack");
    assert_eq!(stack_a.len(), 1);

    let stack_b = app.world().get::<EffectStack<SpeedBoostConfig>>(entity_b);
    assert!(
        stack_b.is_none(),
        "entity_b (no BoundEffects) should not have EffectStack"
    );
}

// -- Behavior 6 edge case: empty BoundEffects still walked but no match --

#[test]
fn impact_occurred_any_empty_bound_effects_no_match() {
    let mut app = bridge_test_app();

    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    let entity = app.world_mut().spawn(BoundEffects(vec![])).id();

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

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "empty BoundEffects should not produce EffectStack"
    );
}

// -- Behavior 7: Any dispatch does not false-match specific gates --

#[test]
fn impact_occurred_any_dispatch_does_not_false_match_specific_gates() {
    let mut app = bridge_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

    // entity_specific gates on Bolt — BreakerImpactCell has no Bolt
    let entity_specific = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_bolt",
            EntityKind::Bolt,
            1.5,
        )]))
        .id();

    // entity_any gates on Any — should always fire
    let entity_any = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_any",
            EntityKind::Any,
            2.0,
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

    // entity_any should fire
    let stack_any = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_any)
        .expect("entity_any should have EffectStack");
    assert_eq!(stack_any.len(), 1);

    // entity_specific should NOT fire — BreakerImpactCell dispatches
    // Breaker and Cell, not Bolt. ImpactOccurred(Any) should not match Bolt gate.
    let stack_specific = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity_specific);
    assert!(
        stack_specific.is_none(),
        "ImpactOccurred(Any) dispatch must not false-match ImpactOccurred(Bolt) gate"
    );
}

// -- Behavior 8: ImpactOccurred(Any) fires for BreakerImpactWall --

#[test]
fn impact_occurred_any_fires_for_breaker_impact_wall() {
    let mut app = bridge_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();
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
        breaker_wall: vec![BreakerImpactWall {
            breaker: breaker_entity,
            wall:    wall_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after BreakerImpactWall + Any gate");
    assert_eq!(stack.len(), 1);
}

// -- Behavior 9: ImpactOccurred(Any) fires for CellImpactWall --

#[test]
fn impact_occurred_any_fires_for_cell_impact_wall() {
    let mut app = bridge_test_app();

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
        cell_wall: vec![CellImpactWall {
            cell: cell_entity,
            wall: wall_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after CellImpactWall + Any gate");
    assert_eq!(stack.len(), 1);
}
