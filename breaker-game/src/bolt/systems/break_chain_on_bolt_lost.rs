//! Chain break detection — despawns constraints when a referenced bolt is removed.

use bevy::prelude::*;
use rantzsoft_physics2d::constraint::DistanceConstraint;

use crate::bolt::components::Bolt;

/// Despawns [`DistanceConstraint`] entities when either referenced bolt entity
/// no longer has the [`Bolt`] component (i.e., it was despawned).
///
/// Uses `RemovedComponents<Bolt>` to detect bolt removal each frame.
/// For each removed entity, scans all constraints and despawns any that
/// reference the removed entity.
pub(crate) fn break_chain_on_bolt_lost(
    mut commands: Commands,
    mut removed: RemovedComponents<Bolt>,
    constraint_query: Query<(Entity, &DistanceConstraint)>,
) {
    for removed_entity in removed.read() {
        for (constraint_entity, constraint) in &constraint_query {
            if constraint.entity_a == removed_entity || constraint.entity_b == removed_entity {
                commands.entity(constraint_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    use rantzsoft_physics2d::constraint::DistanceConstraint;
    use rantzsoft_spatial2d::components::Position2D;

    use crate::bolt::components::{Bolt, BoltVelocity, ExtraBolt};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, break_chain_on_bolt_lost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 15: Constraint despawned when referenced bolt removed ──

    #[test]
    fn constraint_despawned_when_bolt_removed() {
        // Given: bolt A (anchor), bolt B (chain), constraint linking them
        // When: bolt B is despawned
        // Then: constraint entity is also despawned
        let mut app = test_app();

        // Tick once to initialize RemovedComponents tracker
        tick(&mut app);

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Position2D(Vec2::new(100.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let constraint = app
            .world_mut()
            .spawn(DistanceConstraint {
                entity_a: a,
                entity_b: b,
                max_distance: 200.0,
            })
            .id();

        // Tick to register entities
        tick(&mut app);

        // Despawn bolt B
        app.world_mut().despawn(b);

        // Tick so RemovedComponents fires
        tick(&mut app);

        assert!(
            app.world().get_entity(constraint).is_err(),
            "constraint should be despawned when bolt B is removed"
        );
        // Bolt A should still exist
        assert!(
            app.world().get_entity(a).is_ok(),
            "anchor bolt A should still exist"
        );
    }

    #[test]
    fn constraint_despawned_when_anchor_bolt_removed() {
        // Given: bolt A (anchor), bolt B (chain), constraint linking them
        // When: bolt A (the anchor) is despawned
        // Then: constraint entity is also despawned
        let mut app = test_app();

        tick(&mut app);

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Position2D(Vec2::new(100.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let constraint = app
            .world_mut()
            .spawn(DistanceConstraint {
                entity_a: a,
                entity_b: b,
                max_distance: 200.0,
            })
            .id();

        tick(&mut app);

        // Despawn anchor bolt A
        app.world_mut().despawn(a);
        tick(&mut app);

        assert!(
            app.world().get_entity(constraint).is_err(),
            "constraint should be despawned when anchor bolt A is removed"
        );
    }

    // ── Behavior 16: Constraint intact when both bolts alive ──

    #[test]
    fn constraint_intact_when_both_bolts_alive() {
        // Given: bolt A, bolt B, constraint linking them
        // When: neither bolt is removed
        // Then: constraint still exists
        let mut app = test_app();

        tick(&mut app);

        let a = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let b = app
            .world_mut()
            .spawn((
                Bolt,
                ExtraBolt,
                Position2D(Vec2::new(100.0, 0.0)),
                BoltVelocity::new(0.0, 400.0),
            ))
            .id();
        let constraint = app
            .world_mut()
            .spawn(DistanceConstraint {
                entity_a: a,
                entity_b: b,
                max_distance: 200.0,
            })
            .id();

        // Tick several times without removing any bolt
        tick(&mut app);
        tick(&mut app);
        tick(&mut app);

        assert!(
            app.world().get_entity(constraint).is_ok(),
            "constraint should still exist when both bolts are alive"
        );
        let dc = app
            .world()
            .get::<DistanceConstraint>(constraint)
            .expect("DistanceConstraint should still be present");
        assert_eq!(dc.entity_a, a);
        assert_eq!(dc.entity_b, b);
    }
}
