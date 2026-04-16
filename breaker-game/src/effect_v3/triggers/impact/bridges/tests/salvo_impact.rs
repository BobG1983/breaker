use bevy::prelude::*;

use super::helpers::{
    TestImpactMessages, bridge_test_app, impact_occurred_speed_tree, impacted_speed_tree,
    impacted_test_app,
};
use crate::{
    effect_v3::{
        effects::SpeedBoostConfig, stacking::EffectStack, storage::BoundEffects, types::EntityKind,
    },
    prelude::*,
};

// ==========================================================================
// Section 5: Impact Bridge — SalvoImpactBreaker
// ==========================================================================

// -- Behavior 18: Impacted(Salvo) fires locally on breaker entity --

#[test]
fn impacted_salvo_fires_on_breaker_entity_for_salvo_impact_breaker() {
    let mut app = impacted_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impacted_speed_tree(
            "chip_a",
            EntityKind::Salvo,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack from Impacted(Salvo)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn impacted_salvo_and_any_both_fire_on_breaker() {
    let mut app = impacted_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            impacted_speed_tree("chip_salvo", EntityKind::Salvo, 1.5),
            impacted_speed_tree("chip_any", EntityKind::Any, 2.0),
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack");
    assert_eq!(
        stack.len(),
        2,
        "Both Impacted(Salvo) and Impacted(Any) should fire on breaker"
    );
}

// -- Behavior 19: Impacted(Breaker) fires locally on salvo entity --

#[test]
fn impacted_breaker_fires_on_salvo_entity_for_salvo_impact_breaker() {
    let mut app = impacted_test_app();

    let breaker_entity = app.world_mut().spawn_empty().id();

    let salvo_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impacted_speed_tree(
            "chip_a",
            EntityKind::Breaker,
            2.0,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity)
        .expect("salvo should have EffectStack from Impacted(Breaker)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn impacted_breaker_no_panic_when_salvo_has_no_bound_effects() {
    let mut app = impacted_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    // Should not panic
    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity);
    assert!(stack.is_none(), "no BoundEffects means no EffectStack");
}

// -- Behavior 20: Impacted(Any) fires on both participants --

#[test]
fn impacted_any_fires_on_both_participants_for_salvo_impact_breaker() {
    let mut app = impacted_test_app();

    let salvo_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impacted_speed_tree(
            "chip_a",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    let breaker_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impacted_speed_tree(
            "chip_b",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack_salvo = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity)
        .expect("salvo should have EffectStack from Impacted(Any)");
    assert_eq!(stack_salvo.len(), 1);

    let stack_breaker = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity)
        .expect("breaker should have EffectStack from Impacted(Any)");
    assert_eq!(stack_breaker.len(), 1);
}

#[test]
fn impacted_any_only_one_participant_has_bound_effects() {
    let mut app = impacted_test_app();

    let salvo_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impacted_speed_tree(
            "chip_a",
            EntityKind::Any,
            1.5,
        )]))
        .id();

    // breaker has no BoundEffects
    let breaker_entity = app.world_mut().spawn_empty().id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack_salvo = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(salvo_entity)
        .expect("salvo should have EffectStack from Impacted(Any)");
    assert_eq!(stack_salvo.len(), 1);

    let stack_breaker = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker_entity);
    assert!(
        stack_breaker.is_none(),
        "breaker without BoundEffects should be skipped"
    );
}

// -- Behavior 21: ImpactOccurred(Salvo) fires globally --

#[test]
fn impact_occurred_salvo_fires_globally_for_salvo_impact_breaker() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let third_entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Salvo,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(third_entity)
        .expect("third entity should have EffectStack from ImpactOccurred(Salvo)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn impact_occurred_bolt_does_not_fire_for_salvo_impact_breaker() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    // Entity with ImpactOccurred(Bolt) — should NOT fire for SalvoImpactBreaker
    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Bolt,
            1.5,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "ImpactOccurred(Bolt) should NOT fire for SalvoImpactBreaker"
    );
}

// -- Behavior 22: ImpactOccurred(Breaker) fires globally --

#[test]
fn impact_occurred_breaker_fires_globally_for_salvo_impact_breaker() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Breaker,
            2.0,
        )]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack from ImpactOccurred(Breaker)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn impact_occurred_breaker_and_any_both_fire_for_salvo_impact_breaker() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
    let breaker_entity = app.world_mut().spawn_empty().id();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![
            impact_occurred_speed_tree("chip_breaker", EntityKind::Breaker, 2.0),
            impact_occurred_speed_tree("chip_any", EntityKind::Any, 1.5),
        ]))
        .id();

    app.insert_resource(TestImpactMessages {
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
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
        "Both ImpactOccurred(Breaker) and ImpactOccurred(Any) should fire"
    );
}

// -- Behavior 23: ImpactOccurred(Any) fires globally --

#[test]
fn impact_occurred_any_fires_globally_for_salvo_impact_breaker() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
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
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
        }],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack from ImpactOccurred(Any)");
    assert_eq!(stack.len(), 1);
}

#[test]
fn impact_occurred_any_fires_twice_for_two_salvo_impact_breaker_messages() {
    let mut app = bridge_test_app();

    let salvo_a = app.world_mut().spawn_empty().id();
    let salvo_b = app.world_mut().spawn_empty().id();
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
        salvo_breaker: vec![
            SalvoImpactBreaker {
                salvo:   salvo_a,
                breaker: breaker_entity,
            },
            SalvoImpactBreaker {
                salvo:   salvo_b,
                breaker: breaker_entity,
            },
        ],
        ..default()
    });

    tick(&mut app);

    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(entity)
        .expect("entity should have EffectStack after two SalvoImpactBreaker messages");
    assert_eq!(
        stack.len(),
        2,
        "Two collision events should fire ImpactOccurred(Any) twice"
    );
}

// -- Behavior 24: ImpactOccurred(Any) fires once per collision, not per participant --

#[test]
fn impact_occurred_any_fires_once_per_salvo_impact_breaker_collision() {
    let mut app = bridge_test_app();

    let salvo_entity = app.world_mut().spawn_empty().id();
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
        salvo_breaker: vec![SalvoImpactBreaker {
            salvo:   salvo_entity,
            breaker: breaker_entity,
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
        1,
        "Any should fire once per collision event, not once per participant"
    );
}

// -- Behavior 25: No SalvoImpactBreaker — impact bridge is a no-op --

#[test]
fn impact_occurred_salvo_noop_without_salvo_impact_messages() {
    let mut app = bridge_test_app();

    let entity = app
        .world_mut()
        .spawn(BoundEffects(vec![impact_occurred_speed_tree(
            "chip_a",
            EntityKind::Salvo,
            1.5,
        )]))
        .id();

    // No salvo_breaker messages, but other messages fire normally
    let bolt_entity = app.world_mut().spawn_empty().id();
    let cell_entity = app.world_mut().spawn_empty().id();

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

    // Salvo gate should NOT fire (no SalvoImpactBreaker message)
    let stack = app.world().get::<EffectStack<SpeedBoostConfig>>(entity);
    assert!(
        stack.is_none(),
        "ImpactOccurred(Salvo) should not fire without SalvoImpactBreaker messages"
    );
}
