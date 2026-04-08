//! System to keep a serving bolt hovering above the breaker.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltSpawnOffsetY},
        filters::ServingFilter,
    },
    prelude::*,
};

/// Keeps the bolt positioned above the breaker while serving.
///
/// Only affects bolts with the [`BoltServing`] marker. The bolt tracks
/// the breaker's X position so the player can choose their opening angle.
pub fn hover_bolt(
    breaker_query: Query<&Position2D, (With<Breaker>, Without<Bolt>)>,
    mut bolt_query: Query<(&mut Position2D, &BoltSpawnOffsetY), ServingFilter>,
) {
    let Ok(breaker_pos) = breaker_query.single() else {
        return;
    };

    for (mut bolt_position, spawn_offset) in &mut bolt_query {
        bolt_position.0.x = breaker_pos.0.x;
        bolt_position.0.y = breaker_pos.0.y + spawn_offset.0;
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D, Velocity2D};

    use super::*;
    use crate::{bolt::components::BoltServing, shared::GameDrawLayer};

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn hover_bolt_writes_position2d_tracking_breaker() {
        // Given: serving bolt at Position2D(0.0, 0.0), breaker at Position2D(100.0, -250.0),
        //        spawn_offset_y = 30.0
        // When: hover_bolt runs
        // Then: bolt Position2D(Vec2::new(100.0, -220.0))
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let spawn_offset_y = 30.0;

        // Breaker uses Position2D as canonical position
        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        // Serving bolt with Position2D
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            BoltSpawnOffsetY(spawn_offset_y),
            Velocity2D(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(0.0, 0.0)),
        ));

        app.add_systems(FixedUpdate, hover_bolt);
        tick(&mut app);

        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have Position2D");

        let expected = Vec2::new(100.0, -250.0 + spawn_offset_y);
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "hover bolt Position2D should be {expected:?}, got {:?}",
            position.0,
        );
    }

    // ── Behavior 7: hover_bolt still positions birthing+serving bolts ──

    /// Helper to create a `Birthing` component for tests.
    fn test_birthing() -> crate::shared::birthing::Birthing {
        use rantzsoft_physics2d::collision_layers::CollisionLayers;
        use rantzsoft_spatial2d::components::Scale2D;

        use crate::shared::birthing::BIRTHING_DURATION;

        crate::shared::birthing::Birthing {
            timer: Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
            target_scale: Scale2D { x: 8.0, y: 8.0 },
            stashed_layers: CollisionLayers::default(),
        }
    }

    #[test]
    fn hover_bolt_positions_birthing_and_serving_bolt() {
        // Given: bolt with Bolt + BoltServing + Birthing at Position2D(0.0, 0.0),
        //        breaker at Position2D(100.0, -250.0), BoltSpawnOffsetY(30.0)
        // When: hover_bolt runs
        // Then: bolt Position2D is updated to (100.0, -220.0) — proving
        //       ServingFilter is intentionally unchanged (no Without<Birthing>)
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let spawn_offset_y = 30.0;

        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        // Birthing + serving bolt
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            BoltSpawnOffsetY(spawn_offset_y),
            Velocity2D(Vec2::new(0.0, 0.0)),
            Position2D(Vec2::new(0.0, 0.0)),
            test_birthing(),
        ));

        app.add_systems(FixedUpdate, hover_bolt);
        tick(&mut app);

        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have Position2D");

        let expected = Vec2::new(100.0, -250.0 + spawn_offset_y);
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "hover_bolt should position birthing+serving bolt at {:?}, got {:?}",
            expected,
            position.0,
        );
    }

    // Behavior 7 edge case: non-birthing serving bolt is also positioned
    #[test]
    fn hover_bolt_positions_both_birthing_and_non_birthing_serving_bolts() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let spawn_offset_y = 30.0;

        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        // Birthing + serving bolt
        let birthing_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltSpawnOffsetY(spawn_offset_y),
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::ZERO),
                test_birthing(),
            ))
            .id();

        // Non-birthing serving bolt
        let normal_bolt = app
            .world_mut()
            .spawn((
                Bolt,
                BoltServing,
                BoltSpawnOffsetY(spawn_offset_y),
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.add_systems(FixedUpdate, hover_bolt);
        tick(&mut app);

        let expected = Vec2::new(100.0, -250.0 + spawn_offset_y);

        let birthing_pos = app.world().get::<Position2D>(birthing_bolt).unwrap();
        assert!(
            (birthing_pos.0.x - expected.x).abs() < f32::EPSILON
                && (birthing_pos.0.y - expected.y).abs() < f32::EPSILON,
            "birthing+serving bolt should be positioned at {:?}, got {:?}",
            expected,
            birthing_pos.0,
        );

        let normal_pos = app.world().get::<Position2D>(normal_bolt).unwrap();
        assert!(
            (normal_pos.0.x - expected.x).abs() < f32::EPSILON
                && (normal_pos.0.y - expected.y).abs() < f32::EPSILON,
            "non-birthing serving bolt should also be positioned at {:?}, got {:?}",
            expected,
            normal_pos.0,
        );
    }

    #[test]
    fn hover_bolt_ignores_non_serving_bolt() {
        // Given: non-serving bolt at Position2D(50.0, 50.0)
        // When: hover_bolt runs
        // Then: Position2D unchanged
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn((
            Breaker,
            Position2D(Vec2::new(100.0, -250.0)),
            Spatial2D,
            GameDrawLayer::Breaker,
        ));

        // Non-serving bolt (no BoltServing marker) with Position2D
        app.world_mut().spawn((
            Bolt,
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(50.0, 50.0)),
        ));

        app.add_systems(FixedUpdate, hover_bolt);
        tick(&mut app);

        let position = app
            .world_mut()
            .query_filtered::<&Position2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should have Position2D");

        assert!(
            (position.0.x - 50.0).abs() < f32::EPSILON
                && (position.0.y - 50.0).abs() < f32::EPSILON,
            "non-serving bolt Position2D should be unchanged at (50.0, 50.0), got {:?}",
            position.0,
        );
    }
}
