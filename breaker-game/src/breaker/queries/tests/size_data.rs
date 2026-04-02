use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{BaseHeight, BaseWidth, Breaker},
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::NodeScalingFactor,
};

// ── Part B: BreakerSizeData (read-only) ─────────────────────────

// Behavior 3: BreakerSizeData can be queried
#[test]
fn breaker_size_data_query_matches_with_all_components() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            Position2D(Vec2::new(50.0, -200.0)),
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![1.5]),
            NodeScalingFactor(0.8),
        ))
        .id();

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerSizeData, With<Breaker>>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(
        results.len(),
        1,
        "expected exactly 1 result from BreakerSizeData query"
    );
    let data = &results[0];
    assert_eq!(data.entity, entity);
    assert_eq!(data.position.0, Vec2::new(50.0, -200.0));
    assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
    assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
    assert!(data.size_boosts.is_some());
    assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
}

// Behavior 3 edge case: optionals absent, query still matches
#[test]
fn breaker_size_data_query_matches_without_optionals() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(50.0, -200.0)),
        BaseWidth(120.0),
        BaseHeight(20.0),
    ));

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerSizeData, With<Breaker>>();
    let data = query.single(app.world()).unwrap();
    assert!(data.size_boosts.is_none());
    assert!(data.node_scale.is_none());
}
