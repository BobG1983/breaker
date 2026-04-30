use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

use super::super::helpers::*;
use crate::{
    bolt::components::{Bolt, ExtraBolt, PrimaryBolt},
    effect_v3::{
        effects::{DamageBoostConfig, circuit_breaker::components::CircuitBreakerCounter},
        storage::BoundEffects,
        types::{EffectType, Tree},
    },
    shared::test_utils::tick,
};

// ── C11-11: inherit: true attaches BoundEffects to spawned ExtraBolt ──

#[test]
fn inherit_true_attaches_bound_effects_to_spawned_extra_bolts() {
    let mut app = circuit_breaker_app();

    let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    }));
    app.world_mut().spawn((
        PrimaryBolt,
        Bolt,
        Position2D(Vec2::ZERO),
        Velocity2D(Vec2::new(0.0, 400.0)),
        BaseSpeed(400.0),
        BoundEffects(vec![("chip_a".to_string(), tree_a)]),
    ));

    app.world_mut().spawn(CircuitBreakerCounter {
        remaining:       1,
        bumps_required:  2,
        spawn_count:     2,
        inherit:         true,
        shockwave_range: 64.0,
        shockwave_speed: 200.0,
    });
    queue_bump(&mut app);

    tick(&mut app);

    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        extra_count, 2,
        "spawn_count=2 should spawn exactly 2 ExtraBolt entities, got {extra_count}",
    );

    let inherited: Vec<&BoundEffects> = app
        .world_mut()
        .query_filtered::<&BoundEffects, With<ExtraBolt>>()
        .iter(app.world())
        .collect();
    assert_eq!(
        inherited.len(),
        2,
        "both ExtraBolt entities should carry BoundEffects, got {}",
        inherited.len(),
    );
    for effects in &inherited {
        assert!(
            effects.0.iter().any(|(name, _)| name == "chip_a"),
            "inherited BoundEffects should contain chip_a entry",
        );
    }
}

#[test]
fn inherit_true_with_no_primary_bolt_still_spawns_extra_bolts_without_bound_effects() {
    let mut app = circuit_breaker_app();

    app.world_mut().spawn(CircuitBreakerCounter {
        remaining:       1,
        bumps_required:  2,
        spawn_count:     2,
        inherit:         true,
        shockwave_range: 64.0,
        shockwave_speed: 200.0,
    });
    queue_bump(&mut app);

    tick(&mut app);

    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        extra_count, 2,
        "spawn_count=2 should spawn exactly 2 ExtraBolt entities even with no PrimaryBolt, got {extra_count}",
    );

    let inherited_count = app
        .world_mut()
        .query_filtered::<&BoundEffects, With<ExtraBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        inherited_count, 0,
        "no PrimaryBolt means no BoundEffects to inherit, got {inherited_count}",
    );
}

// ── C11-12: inherit: false does NOT attach BoundEffects even with PrimaryBolt ──

#[test]
fn inherit_false_does_not_attach_bound_effects_even_with_primary_bolt() {
    let mut app = circuit_breaker_app();

    let tree_a = Tree::Fire(EffectType::DamageBoost(DamageBoostConfig {
        multiplier: OrderedFloat(2.0),
    }));
    app.world_mut().spawn((
        PrimaryBolt,
        Bolt,
        Position2D(Vec2::ZERO),
        Velocity2D(Vec2::new(0.0, 400.0)),
        BaseSpeed(400.0),
        BoundEffects(vec![("chip_a".to_string(), tree_a)]),
    ));

    app.world_mut().spawn(CircuitBreakerCounter {
        remaining:       1,
        bumps_required:  2,
        spawn_count:     3,
        inherit:         false,
        shockwave_range: 64.0,
        shockwave_speed: 200.0,
    });
    queue_bump(&mut app);

    tick(&mut app);

    let extra_count = app
        .world_mut()
        .query_filtered::<Entity, With<ExtraBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        extra_count, 3,
        "spawn_count=3 should spawn exactly 3 ExtraBolt entities, got {extra_count}",
    );

    let inherited_count = app
        .world_mut()
        .query_filtered::<&BoundEffects, With<ExtraBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        inherited_count, 0,
        "inherit=false should NOT attach BoundEffects, got {inherited_count}",
    );
}
