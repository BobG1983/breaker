use bevy::prelude::*;
use rantzsoft_spatial2d::components::Scale2D;

use super::system::*;
use crate::{
    breaker::components::{BaseHeight, BaseWidth, Breaker},
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::{
        NodeScalingFactor,
        size::{MaxHeight, MaxWidth, MinHeight, MinWidth},
        test_utils::TestAppBuilder,
    },
};

fn test_app() -> App {
    TestAppBuilder::new()
        .with_system(FixedUpdate, sync_breaker_scale)
        .build()
}

use crate::shared::test_utils::tick;

// ── Behavior 16: Base dimensions with no boosts ─────────────────

#[test]
fn sync_breaker_scale_sets_base_dimensions_with_no_boosts() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), Scale2D { x: 1.0, y: 1.0 }, no ActiveSizeBoosts, no NodeScalingFactor
    // When: sync_breaker_scale runs
    // Then: Scale2D { x: 120.0, y: 20.0 }
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

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 120.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "expected Scale2D (120.0, 20.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 17: ActiveSizeBoosts applies to BOTH width and height ──

#[test]
fn sync_breaker_scale_applies_boost_to_both_axes() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![4/3])
    // When: sync_breaker_scale runs
    // Then: Scale2D { x: 160.0, y: ~26.666 } (boost on BOTH axes)
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 160.0).abs() < 1e-3,
        "expected Scale2D.x = 160.0, got {}",
        scale.x,
    );
    // Key behavioral change: height is ALSO boosted (unlike old width_boost_visual)
    let expected_y = 20.0 * 4.0 / 3.0;
    assert!(
        (scale.y - expected_y).abs() < 1e-3,
        "expected Scale2D.y = {expected_y} (boost applies to height), got {}",
        scale.y,
    );
}

#[test]
fn sync_breaker_scale_with_1_5_boost_on_both_axes() {
    // Edge case: ActiveSizeBoosts(vec![1.5])
    // Scale2D { x: 180.0, y: 30.0 }
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![1.5]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 180.0).abs() < 1e-3,
        "expected Scale2D.x = 180.0, got {}",
        scale.x,
    );
    assert!(
        (scale.y - 30.0).abs() < 1e-3,
        "expected Scale2D.y = 30.0 (boost on height), got {}",
        scale.y,
    );
}

// ── Behavior 18: NodeScalingFactor applies to both axes ─────────

#[test]
fn sync_breaker_scale_applies_node_scale_to_both_axes() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), NodeScalingFactor(0.7), no ActiveSizeBoosts
    // Then: Scale2D { x: 84.0, y: 14.0 }
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            NodeScalingFactor(0.7),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 84.0).abs() < 1e-3,
        "expected Scale2D.x = 84.0, got {}",
        scale.x,
    );
    assert!(
        (scale.y - 14.0).abs() < 1e-3,
        "expected Scale2D.y = 14.0, got {}",
        scale.y,
    );
}

#[test]
fn sync_breaker_scale_identity_node_scale() {
    // Edge case: NodeScalingFactor(1.0) should produce same as no scale
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            NodeScalingFactor(1.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 120.0).abs() < f32::EPSILON && (scale.y - 20.0).abs() < f32::EPSILON,
        "NodeScalingFactor(1.0) should produce (120.0, 20.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 19: Both boosts and node scale ─────────────────────

#[test]
fn sync_breaker_scale_with_boost_and_node_scale() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![4/3]), NodeScalingFactor(0.7)
    // Then: Scale2D { x: 112.0, y: ~18.666 }
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
            NodeScalingFactor(0.7),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 112.0).abs() < 1e-3,
        "expected Scale2D.x = 112.0, got {}",
        scale.x,
    );
    let expected_y = 20.0 * 4.0 / 3.0 * 0.7;
    assert!(
        (scale.y - expected_y).abs() < 1e-2,
        "expected Scale2D.y = {expected_y}, got {}",
        scale.y,
    );
}

#[test]
fn sync_breaker_scale_boost_with_identity_node_scale() {
    // Edge case: NodeScalingFactor(1.0) with boost -> same as boost-only
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![4.0_f32 / 3.0]),
            NodeScalingFactor(1.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 160.0).abs() < 1e-3,
        "expected Scale2D.x = 160.0, got {}",
        scale.x,
    );
    let expected_y = 20.0 * 4.0 / 3.0;
    assert!(
        (scale.y - expected_y).abs() < 1e-3,
        "expected Scale2D.y = {expected_y}, got {}",
        scale.y,
    );
}

// ── Behavior 20: Large boost with no constraint components ──────

#[test]
fn sync_breaker_scale_large_boost_no_constraints_unclamped() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![10.0]), no min/max
    // Then: Scale2D { x: 1200.0, y: 200.0 } (unclamped)
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![10.0]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 1200.0).abs() < 1e-3,
        "expected Scale2D.x = 1200.0 (unclamped), got {}",
        scale.x,
    );
    assert!(
        (scale.y - 200.0).abs() < 1e-3,
        "expected Scale2D.y = 200.0 (unclamped), got {}",
        scale.y,
    );
}

#[test]
fn sync_breaker_scale_small_node_scale_no_constraints_unclamped() {
    // Edge case: NodeScalingFactor(0.01) with no constraints
    // Scale2D { x: 1.2, y: 0.2 }
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            NodeScalingFactor(0.01),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 1.2).abs() < 1e-3,
        "expected Scale2D.x = 1.2 (unclamped), got {}",
        scale.x,
    );
    assert!(
        (scale.y - 0.2).abs() < 1e-3,
        "expected Scale2D.y = 0.2 (unclamped), got {}",
        scale.y,
    );
}

// ── Behavior 21: Clamps to min/max when constraint components present ──

#[test]
fn sync_breaker_scale_clamps_to_constraints() {
    // Given: BaseWidth(120.0), BaseHeight(20.0), ActiveSizeBoosts(vec![3.0]),
    //        MinWidth(60.0), MaxWidth(200.0), MinHeight(10.0), MaxHeight(50.0)
    // width: 120.0 * 3.0 = 360.0 -> clamped to 200.0
    // height: 20.0 * 3.0 = 60.0 -> clamped to 50.0
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            ActiveSizeBoosts(vec![3.0]),
            MinWidth(60.0),
            MaxWidth(200.0),
            MinHeight(10.0),
            MaxHeight(50.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 200.0).abs() < 1e-3,
        "expected Scale2D.x = 200.0 (clamped to max), got {}",
        scale.x,
    );
    assert!(
        (scale.y - 50.0).abs() < 1e-3,
        "expected Scale2D.y = 50.0 (clamped to max), got {}",
        scale.y,
    );
}

#[test]
fn sync_breaker_scale_clamps_to_min_when_scaled_small() {
    // Edge case: NodeScalingFactor(0.1), no boosts, same min/max
    // width: 120.0 * 0.1 = 12.0 -> clamped to 60.0
    // height: 20.0 * 0.1 = 2.0 -> clamped to 10.0
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            NodeScalingFactor(0.1),
            MinWidth(60.0),
            MaxWidth(200.0),
            MinHeight(10.0),
            MaxHeight(50.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 60.0).abs() < 1e-3,
        "expected Scale2D.x = 60.0 (clamped to min), got {}",
        scale.x,
    );
    assert!(
        (scale.y - 10.0).abs() < 1e-3,
        "expected Scale2D.y = 10.0 (clamped to min), got {}",
        scale.y,
    );
}
