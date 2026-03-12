//! System to keep a serving bolt hovering above the breaker.

use bevy::prelude::*;

use crate::{
    bolt::{
        components::{Bolt, BoltSpawnOffsetY},
        filters::ServingBoltFilter,
    },
    breaker::components::Breaker,
};

/// Keeps the bolt positioned above the breaker while serving.
///
/// Only affects bolts with the [`BoltServing`] marker. The bolt tracks
/// the breaker's X position so the player can choose their opening angle.
pub fn hover_bolt(
    breaker_query: Query<&Transform, (With<Breaker>, Without<Bolt>)>,
    mut bolt_query: Query<(&mut Transform, &BoltSpawnOffsetY), ServingBoltFilter>,
) {
    let Ok(breaker_tf) = breaker_query.single() else {
        return;
    };

    for (mut bolt_tf, spawn_offset) in &mut bolt_query {
        bolt_tf.translation.x = breaker_tf.translation.x;
        bolt_tf.translation.y = breaker_tf.translation.y + spawn_offset.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bolt::{
        components::{BoltServing, BoltVelocity},
        resources::BoltConfig,
    };

    #[test]
    fn hover_bolt_tracks_breaker_x() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let config = BoltConfig::default();

        // Spawn breaker at x=100
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(100.0, -250.0, 0.0)));

        // Spawn serving bolt at origin
        app.world_mut().spawn((
            Bolt,
            BoltServing,
            BoltSpawnOffsetY(config.spawn_offset_y),
            BoltVelocity::new(0.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        app.add_systems(Update, hover_bolt);
        app.update();

        let bolt_tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            (bolt_tf.translation.x - 100.0).abs() < f32::EPSILON,
            "bolt X should track breaker X"
        );
        assert!(
            (bolt_tf.translation.y - (-250.0 + config.spawn_offset_y)).abs() < f32::EPSILON,
            "bolt Y should be above breaker"
        );
    }

    #[test]
    fn hover_bolt_ignores_non_serving_bolt() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn breaker
        app.world_mut()
            .spawn((Breaker, Transform::from_xyz(100.0, -250.0, 0.0)));

        // Spawn non-serving bolt (no BoltServing marker)
        app.world_mut().spawn((
            Bolt,
            BoltVelocity::new(0.0, 400.0),
            Transform::from_xyz(50.0, 50.0, 0.0),
        ));

        app.add_systems(Update, hover_bolt);
        app.update();

        let bolt_tf = app
            .world_mut()
            .query_filtered::<&Transform, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt should exist");

        assert!(
            (bolt_tf.translation.x - 50.0).abs() < f32::EPSILON,
            "non-serving bolt X should be unchanged"
        );
    }
}
