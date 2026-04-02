//! Syncs bolt `Scale2D` to effective radius each frame.
//!
//! Replaces `bolt_scale_visual` -- applies `ActiveSizeBoosts`, `NodeScalingFactor`,
//! and optional min/max radius constraints via `effective_radius`.

use bevy::prelude::*;

use crate::bolt::{components::Bolt, queries::SyncBoltScaleData};

/// Sets bolt [`Scale2D`] based on [`BaseRadius`], optional [`ActiveSizeBoosts`],
/// optional [`NodeScalingFactor`], and optional min/max radius constraints.
pub(crate) fn sync_bolt_scale(mut query: Query<SyncBoltScaleData, With<Bolt>>) {
    use crate::{
        effect::effects::size_boost::ActiveSizeBoosts,
        shared::size::{ClampRange, effective_radius},
    };

    for mut data in &mut query {
        let boost = data.size_boosts.map_or(1.0, ActiveSizeBoosts::multiplier);
        let node = data.node_scale.map_or(1.0, |s| s.0);
        let range = ClampRange {
            min: data.min_radius.map(|m| m.0),
            max: data.max_radius.map(|m| m.0),
        };
        let r = effective_radius(data.base_radius.0, boost, node, range);
        data.scale.x = r;
        data.scale.y = r;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::Scale2D;

    use super::*;
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
            .add_systems(FixedUpdate, sync_bolt_scale);
        app
    }

    /// Accumulates one fixed timestep then runs one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
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
}
