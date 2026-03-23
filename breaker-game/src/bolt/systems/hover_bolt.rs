//! System to keep a serving bolt hovering above the breaker.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    bolt::{
        components::{Bolt, BoltSpawnOffsetY},
        filters::ServingFilter,
    },
    breaker::components::Breaker,
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
    use rantzsoft_spatial2d::components::{Position2D, Spatial2D};

    use super::*;
    use crate::{
        bolt::{
            components::{BoltServing, BoltVelocity},
            resources::BoltConfig,
        },
        shared::GameDrawLayer,
    };

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

        let config = BoltConfig::default();

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
            BoltSpawnOffsetY(config.spawn_offset_y),
            BoltVelocity::new(0.0, 0.0),
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

        let expected = Vec2::new(100.0, -250.0 + config.spawn_offset_y);
        assert!(
            (position.0.x - expected.x).abs() < f32::EPSILON
                && (position.0.y - expected.y).abs() < f32::EPSILON,
            "hover bolt Position2D should be {expected:?}, got {:?}",
            position.0,
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
            BoltVelocity::new(0.0, 400.0),
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
