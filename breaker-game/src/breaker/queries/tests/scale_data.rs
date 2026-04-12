use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Scale2D;

use super::{super::data::*, helpers::*};
use crate::{
    breaker::components::{BaseHeight, BaseWidth, Breaker},
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    shared::{
        NodeScalingFactor,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
    },
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

// ── Part H: SyncBreakerScaleData (mutable) ──────────────────────

// Behavior 16: SyncBreakerScaleData named field access
#[test]
fn sync_breaker_scale_data_named_field_access() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BaseWidth(120.0),
        BaseHeight(20.0),
        Scale2D { x: 1.0, y: 1.0 },
        size_stack(&[1.5]),
        NodeScalingFactor(0.8),
        MinWidth(60.0),
        MaxWidth(200.0),
        MinHeight(10.0),
        MaxHeight(50.0),
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<SyncBreakerScaleDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!((data.base_width.0 - 120.0).abs() < f32::EPSILON);
                assert!((data.base_height.0 - 20.0).abs() < f32::EPSILON);
                assert!(data.size_boosts.is_some());
                assert!((data.node_scale.unwrap().0 - 0.8).abs() < f32::EPSILON);
                assert!((data.min_w.unwrap().0 - 60.0).abs() < f32::EPSILON);
                assert!((data.max_w.unwrap().0 - 200.0).abs() < f32::EPSILON);
                assert!((data.min_h.unwrap().0 - 10.0).abs() < f32::EPSILON);
                assert!((data.max_h.unwrap().0 - 50.0).abs() < f32::EPSILON);
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 16 edge case: all optionals absent
#[test]
fn sync_breaker_scale_data_all_optionals_absent() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        BaseWidth(120.0),
        BaseHeight(20.0),
        Scale2D { x: 1.0, y: 1.0 },
    ));

    app.add_systems(
        FixedUpdate,
        |query: Query<SyncBreakerScaleDataReadOnly, With<Breaker>>,
         mut matched: ResMut<QueryMatched>| {
            for data in &query {
                matched.0 = true;
                assert!(data.size_boosts.is_none());
                assert!(data.node_scale.is_none());
                assert!(data.min_w.is_none());
                assert!(data.max_w.is_none());
                assert!(data.min_h.is_none());
                assert!(data.max_h.is_none());
            }
        },
    );
    tick(&mut app);
    assert_query_matched(&app);
}

// Behavior 17: Scale2D mutation through SyncBreakerScaleData
#[test]
fn sync_breaker_scale_data_scale_mutation() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    app.add_systems(
        FixedUpdate,
        |mut query: Query<SyncBreakerScaleData, With<Breaker>>| {
            for mut data in &mut query {
                data.scale.x = 160.0;
                data.scale.y = 26.666;
            }
        },
    );
    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!((scale.x - 160.0).abs() < f32::EPSILON);
    assert!((scale.y - 26.666).abs() < 1e-3);
}
