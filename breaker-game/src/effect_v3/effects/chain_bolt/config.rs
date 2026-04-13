//! `ChainBoltConfig` — fire-and-forget chain bolt redirect.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_physics2d::{collision_layers::CollisionLayers, constraint::DistanceConstraint};
use rantzsoft_spatial2d::components::{Position2D, Scale2D, Velocity2D};
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use crate::{
    bolt::components::{Bolt, ExtraBolt},
    effect_v3::traits::Fireable,
    shared::birthing::Birthing,
    state::types::NodeState,
};

/// Tethers two bolts with a distance constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainBoltConfig {
    /// Maximum distance between the two tethered bolts before the constraint pulls them back.
    pub tether_distance: OrderedFloat<f32>,
}

impl Fireable for ChainBoltConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        // Read source state
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let vel = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

        // Negate source velocity for chain bolt direction
        let chain_vel = Vec2::new(-vel.x, -vel.y);

        let birthing = Birthing::new(Scale2D { x: 8.0, y: 8.0 }, CollisionLayers::default());

        // Spawn the chain bolt
        let new_bolt = world
            .spawn((
                Bolt,
                ExtraBolt,
                Position2D(pos),
                Velocity2D(chain_vel),
                birthing,
            ))
            .id();

        // Spawn the distance constraint linking source and new bolt
        world.spawn((
            DistanceConstraint {
                entity_a:     entity,
                entity_b:     new_bolt,
                max_distance: self.tether_distance.0,
            },
            CleanupOnExit::<NodeState>::default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_physics2d::constraint::DistanceConstraint;
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::{Bolt, ExtraBolt},
        effect_v3::traits::Fireable,
        shared::birthing::Birthing,
    };

    fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
        world
            .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
            .id()
    }

    #[test]
    fn fire_spawns_one_extra_bolt() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let extra_count = world
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(extra_count, 1, "should spawn exactly 1 ExtraBolt entity");
    }

    #[test]
    fn spawned_bolt_is_at_source_position() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(80.0, 160.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let positions: Vec<Vec2> = world
            .query_filtered::<&Position2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|p| p.0)
            .collect();
        assert_eq!(positions.len(), 1);
        assert!(
            (positions[0] - Vec2::new(80.0, 160.0)).length() < 1e-3,
            "spawned bolt should be at source position, got {:?}",
            positions[0],
        );
    }

    #[test]
    fn spawned_bolt_has_negated_velocity() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        let expected = Vec2::new(-200.0, -300.0);
        assert!(
            (velocities[0] - expected).length() < 1e-3,
            "chain bolt velocity should be negated ({expected:?}), got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn spawned_bolt_zero_velocity_source_gets_zero_velocity() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::ZERO);

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let velocities: Vec<Vec2> = world
            .query_filtered::<&Velocity2D, With<ExtraBolt>>()
            .iter(&world)
            .map(|v| v.0)
            .collect();
        assert_eq!(velocities.len(), 1);
        assert!(
            velocities[0].length() < 1e-3,
            "negated zero velocity should be zero, got {:?}",
            velocities[0],
        );
    }

    #[test]
    fn distance_constraint_links_source_and_spawned_bolt() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let spawned_bolt = world
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(&world)
            .next()
            .expect("should have spawned an ExtraBolt");

        let constraints: Vec<&DistanceConstraint> =
            world.query::<&DistanceConstraint>().iter(&world).collect();
        assert_eq!(
            constraints.len(),
            1,
            "should spawn exactly 1 DistanceConstraint"
        );
        assert_eq!(constraints[0].entity_a, source, "entity_a should be source");
        assert_eq!(
            constraints[0].entity_b, spawned_bolt,
            "entity_b should be spawned bolt"
        );
        assert!(
            (constraints[0].max_distance - 120.0).abs() < 1e-3,
            "max_distance should be 120.0, got {}",
            constraints[0].max_distance,
        );
    }

    #[test]
    fn spawned_bolt_has_bolt_and_extra_bolt_markers() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let both_count = world
            .query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()
            .iter(&world)
            .count();
        assert_eq!(
            both_count, 1,
            "spawned bolt should have both Bolt and ExtraBolt"
        );
    }

    #[test]
    fn spawned_bolt_has_birthing_component() {
        let mut world = World::new();
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(200.0, 300.0));

        let config = ChainBoltConfig {
            tether_distance: OrderedFloat(120.0),
        };
        config.fire(source, "chain_bolt", &mut world);
        world.flush();

        let birthing_count = world
            .query_filtered::<&Birthing, With<ExtraBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            birthing_count, 1,
            "spawned bolt should have Birthing component"
        );
    }
}
