use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{BaseHeight, BaseWidth, BreakerReflectionSpread, BreakerTilt},
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    prelude::*,
};

fn size_stack(values: &[f32]) -> EffectStack<SizeBoostConfig> {
    let mut stack = EffectStack::default();
    for &v in values {
        stack.push(
            "test".into(),
            SizeBoostConfig {
                multiplier: OrderedFloat(v),
            },
        );
    }
    stack
}

// ── Part A: BreakerCollisionData (read-only) ────────────────────

// Behavior 1: BreakerCollisionData can be queried with all required components
#[test]
fn breaker_collision_data_query_matches_with_required_components() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -200.0)),
        BreakerTilt::default(),
        BaseWidth(120.0),
        BaseHeight(20.0),
        BreakerReflectionSpread(0.5),
    ));

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerCollisionData, With<Breaker>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly 1 result from BreakerCollisionData query"
    );
    let data = &results[0];
    assert_eq!(data.position.0, Vec2::new(0.0, -200.0));
    assert!((data.tilt.angle - 0.0).abs() < f32::EPSILON);
    assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
    assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
    assert!((data.reflection_spread.0 - 0.5).abs() < f32::EPSILON);
    assert!(data.size_boosts.is_none());
    assert!(data.node_scale.is_none());
}

// Behavior 1 edge case: entity missing BreakerReflectionSpread -- query returns 0 results
#[test]
fn breaker_collision_data_query_skips_entity_missing_required_component() {
    let mut app = test_app();
    // Spawn entity WITHOUT BreakerReflectionSpread -- should not match
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -200.0)),
        BreakerTilt::default(),
        BaseWidth(120.0),
        BaseHeight(20.0),
    ));

    let count = app
        .world_mut()
        .query_filtered::<BreakerCollisionData, With<Breaker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "entity missing BreakerReflectionSpread should not match BreakerCollisionData"
    );
}

// Behavior 2: BreakerCollisionData with optional components present
#[test]
fn breaker_collision_data_query_includes_optional_components() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -200.0)),
        BreakerTilt::default(),
        BaseWidth(120.0),
        BaseHeight(20.0),
        BreakerReflectionSpread(0.5),
        size_stack(&[1.5]),
        NodeScalingFactor(0.8),
    ));

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerCollisionData, With<Breaker>>();
    let data = query.single(app.world()).unwrap();
    assert!(
        data.size_boosts.is_some(),
        "ActiveSizeBoosts should be Some"
    );
    assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
}

// Behavior 2 edge case: only one optional component present
#[test]
fn breaker_collision_data_query_partial_optionals() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(0.0, -200.0)),
        BreakerTilt::default(),
        BaseWidth(120.0),
        BaseHeight(20.0),
        BreakerReflectionSpread(0.5),
        NodeScalingFactor(0.8),
        // No ActiveSizeBoosts
    ));

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerCollisionData, With<Breaker>>();
    let data = query.single(app.world()).unwrap();
    assert!(
        data.node_scale.is_some(),
        "NodeScalingFactor should be Some"
    );
    assert!(
        data.size_boosts.is_none(),
        "ActiveSizeBoosts should be None"
    );
}
