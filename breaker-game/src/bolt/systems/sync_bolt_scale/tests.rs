use bevy::prelude::*;
use rantzsoft_spatial2d::components::Scale2D;

use super::system::*;
use crate::{
    bolt::components::Bolt,
    effect::effects::size_boost::ActiveSizeBoosts,
    shared::{
        NodeScalingFactor,
        size::{BaseRadius, MaxRadius, MinRadius},
    },
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, sync_bolt_scale);
    app
}

/// Runs one update tick.
fn tick(app: &mut App) {
    app.update();
}

// ── Behavior 9: Base radius with no boosts sets Scale2D to base radius ──

#[test]
fn sync_bolt_scale_sets_base_radius_with_no_boosts() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((Bolt, BaseRadius(8.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "expected Scale2D (8.0, 8.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_sets_base_radius_14() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((Bolt, BaseRadius(14.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON && (scale.y - 14.0).abs() < f32::EPSILON,
        "expected Scale2D (14.0, 14.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 10: ActiveSizeBoosts applies to bolt radius ──

#[test]
fn sync_bolt_scale_applies_boost() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            ActiveSizeBoosts(vec![2.0]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 16.0).abs() < 1e-3 && (scale.y - 16.0).abs() < 1e-3,
        "expected Scale2D (16.0, 16.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_identity_boost() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            ActiveSizeBoosts(vec![1.0]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "expected Scale2D (8.0, 8.0) with identity boost, got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 11: NodeScalingFactor applies to bolt radius ──

#[test]
fn sync_bolt_scale_applies_node_scale() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            NodeScalingFactor(0.5),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 4.0).abs() < 1e-3 && (scale.y - 4.0).abs() < 1e-3,
        "expected Scale2D (4.0, 4.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_identity_node_scale() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            NodeScalingFactor(1.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < f32::EPSILON && (scale.y - 8.0).abs() < f32::EPSILON,
        "expected Scale2D (8.0, 8.0) with identity node scale, got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 12: Both boosts and node scale multiply together ──

#[test]
fn sync_bolt_scale_boost_and_node_scale_multiply() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            ActiveSizeBoosts(vec![2.0]),
            NodeScalingFactor(0.5),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 8.0).abs() < 1e-3 && (scale.y - 8.0).abs() < 1e-3,
        "expected Scale2D (8.0, 8.0) (8.0 * 2.0 * 0.5), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_large_boost_with_fractional_scale() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(14.0),
            ActiveSizeBoosts(vec![3.0]),
            NodeScalingFactor(0.7),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 29.4).abs() < 1e-2 && (scale.y - 29.4).abs() < 1e-2,
        "expected Scale2D (29.4, 29.4) (14.0 * 3.0 * 0.7), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 13: Large boost with no constraint components is unclamped ──

#[test]
fn sync_bolt_scale_large_boost_no_constraints_unclamped() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            ActiveSizeBoosts(vec![10.0]),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 80.0).abs() < 1e-3 && (scale.y - 80.0).abs() < 1e-3,
        "expected Scale2D (80.0, 80.0) (unclamped), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_small_node_scale_no_constraints_unclamped() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            NodeScalingFactor(0.01),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 0.08).abs() < 1e-5 && (scale.y - 0.08).abs() < 1e-5,
        "expected Scale2D (0.08, 0.08) (unclamped), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 14: Clamps to min/max when constraint components present ──

#[test]
fn sync_bolt_scale_clamps_to_max() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            ActiveSizeBoosts(vec![10.0]),
            MinRadius(4.0),
            MaxRadius(20.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 20.0).abs() < 1e-3 && (scale.y - 20.0).abs() < 1e-3,
        "expected Scale2D (20.0, 20.0) (80.0 clamped to max), got ({}, {})",
        scale.x,
        scale.y,
    );
}

#[test]
fn sync_bolt_scale_clamps_to_min() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            NodeScalingFactor(0.01),
            MinRadius(4.0),
            MaxRadius(20.0),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 4.0).abs() < 1e-3 && (scale.y - 4.0).abs() < 1e-3,
        "expected Scale2D (4.0, 4.0) (0.08 clamped to min), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// ── Behavior 15: Only queries entities with Bolt marker ──

#[test]
fn sync_bolt_scale_only_affects_bolt_entities() {
    let mut app = test_app();

    let bolt_entity = app
        .world_mut()
        .spawn((Bolt, BaseRadius(8.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    let non_bolt_entity = app
        .world_mut()
        .spawn((BaseRadius(20.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    tick(&mut app);

    let bolt_scale = app.world().get::<Scale2D>(bolt_entity).unwrap();
    assert!(
        (bolt_scale.x - 8.0).abs() < f32::EPSILON && (bolt_scale.y - 8.0).abs() < f32::EPSILON,
        "Bolt entity should have Scale2D (8.0, 8.0), got ({}, {})",
        bolt_scale.x,
        bolt_scale.y,
    );

    let non_bolt_scale = app.world().get::<Scale2D>(non_bolt_entity).unwrap();
    assert!(
        (non_bolt_scale.x - 1.0).abs() < f32::EPSILON
            && (non_bolt_scale.y - 1.0).abs() < f32::EPSILON,
        "Non-bolt entity should remain at Scale2D (1.0, 1.0), got ({}, {})",
        non_bolt_scale.x,
        non_bolt_scale.y,
    );
}

#[test]
fn sync_bolt_scale_empty_world_no_panic() {
    let mut app = test_app();
    // No entities — system should run without panicking
    tick(&mut app);
}

// ── Behaviors 8-9: sync_bolt_scale skips bolts with Birthing ──

/// Helper to create a `Birthing` component for tests.
fn test_birthing() -> crate::shared::birthing::Birthing {
    use rantzsoft_physics2d::collision_layers::CollisionLayers;

    use crate::shared::birthing::BIRTHING_DURATION;

    crate::shared::birthing::Birthing {
        timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
        target_scale: Scale2D { x: 8.0, y: 8.0 },
        stashed_layers: CollisionLayers::default(),
    }
}

// Behavior 8: sync_bolt_scale skips bolts with Birthing
#[test]
fn sync_bolt_scale_skips_bolts_with_birthing() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            Scale2D { x: 1.0, y: 1.0 },
            test_birthing(),
        ))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 1.0).abs() < f32::EPSILON && (scale.y - 1.0).abs() < f32::EPSILON,
        "sync_bolt_scale should NOT modify Scale2D on bolt with Birthing, expected (1.0, 1.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}

// Behavior 8 edge case: non-birthing bolt DOES get Scale2D updated
#[test]
fn sync_bolt_scale_skips_birthing_but_processes_non_birthing() {
    let mut app = test_app();

    let birthing_bolt = app
        .world_mut()
        .spawn((
            Bolt,
            BaseRadius(8.0),
            Scale2D { x: 1.0, y: 1.0 },
            test_birthing(),
        ))
        .id();

    let normal_bolt = app
        .world_mut()
        .spawn((Bolt, BaseRadius(8.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    tick(&mut app);

    // Birthing bolt: Scale2D unchanged
    let birthing_scale = app.world().get::<Scale2D>(birthing_bolt).unwrap();
    assert!(
        (birthing_scale.x - 1.0).abs() < f32::EPSILON
            && (birthing_scale.y - 1.0).abs() < f32::EPSILON,
        "birthing bolt Scale2D should remain (1.0, 1.0), got ({}, {})",
        birthing_scale.x,
        birthing_scale.y,
    );

    // Non-birthing bolt: Scale2D set to (8.0, 8.0)
    let normal_scale = app.world().get::<Scale2D>(normal_bolt).unwrap();
    assert!(
        (normal_scale.x - 8.0).abs() < f32::EPSILON && (normal_scale.y - 8.0).abs() < f32::EPSILON,
        "non-birthing bolt Scale2D should be (8.0, 8.0), got ({}, {})",
        normal_scale.x,
        normal_scale.y,
    );
}

// Behavior 9: sync_bolt_scale processes bolts without Birthing normally
#[test]
fn sync_bolt_scale_processes_non_birthing_bolt_with_radius_14() {
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((Bolt, BaseRadius(14.0), Scale2D { x: 1.0, y: 1.0 }))
        .id();

    tick(&mut app);

    let scale = app.world().get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 14.0).abs() < f32::EPSILON && (scale.y - 14.0).abs() < f32::EPSILON,
        "non-birthing bolt Scale2D should be (14.0, 14.0), got ({}, {})",
        scale.x,
        scale.y,
    );
}
