//! System to spawn tethered chain bolt entities.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, constraint::DistanceConstraint,
};
use rantzsoft_spatial2d::components::{
    Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{components::*, messages::SpawnChainBolt, resources::BoltConfig},
    shared::{BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameRng, WALL_LAYER},
};

/// Reads [`SpawnChainBolt`] messages and spawns tethered chain bolt entities.
///
/// Each chain bolt spawns at the anchor bolt's position plus a small upward
/// offset, with a randomized upward velocity at base speed. A standalone
/// `DistanceConstraint` entity is created to tether the new bolt to the
/// anchor.
///
/// The chain bolt is marked `ExtraBolt` so it despawns on loss rather than
/// respawning.
pub(crate) fn spawn_chain_bolt(
    mut commands: Commands,
    mut reader: MessageReader<SpawnChainBolt>,
    bolt_config: Res<BoltConfig>,
    mut rng: ResMut<GameRng>,
    bolt_query: Query<&Position2D, With<Bolt>>,
) {
    for msg in reader.read() {
        // Skip if anchor bolt no longer exists
        let Ok(anchor_pos) = bolt_query.get(msg.anchor) else {
            continue;
        };

        let angle = rng
            .0
            .random_range(-bolt_config.respawn_angle_spread..=bolt_config.respawn_angle_spread);
        let velocity = Velocity2D(Vec2::new(
            bolt_config.base_speed * angle.sin(),
            bolt_config.base_speed * angle.cos(),
        ));

        let spawn_pos = Vec2::new(anchor_pos.0.x, anchor_pos.0.y + bolt_config.spawn_offset_y);

        let new_bolt = commands
            .spawn((
                Bolt,
                ExtraBolt,
                velocity,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: bolt_config.radius,
                    y: bolt_config.radius,
                },
                PreviousScale {
                    x: bolt_config.radius,
                    y: bolt_config.radius,
                },
                Aabb2D::new(
                    Vec2::ZERO,
                    Vec2::new(bolt_config.radius, bolt_config.radius),
                ),
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                (
                    BoltBaseSpeed(bolt_config.base_speed),
                    BoltMinSpeed(bolt_config.min_speed),
                    BoltMaxSpeed(bolt_config.max_speed),
                    BoltRadius(bolt_config.radius),
                    BoltSpawnOffsetY(bolt_config.spawn_offset_y),
                    BoltRespawnOffsetY(bolt_config.respawn_offset_y),
                    BoltRespawnAngleSpread(bolt_config.respawn_angle_spread),
                    BoltInitialAngle(bolt_config.initial_angle),
                ),
                CleanupOnNodeExit,
            ))
            .id();

        // Standalone constraint entity linking anchor to new bolt
        commands.spawn((
            DistanceConstraint {
                entity_a: msg.anchor,
                entity_b: new_bolt,
                max_distance: msg.tether_distance,
            },
            CleanupOnNodeExit,
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, constraint::DistanceConstraint,
    };
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::{
            components::{Bolt, ExtraBolt},
            messages::SpawnChainBolt,
            resources::BoltConfig,
        },
        shared::{CleanupOnNodeExit, GameRng},
    };

    #[derive(Resource)]
    struct SendSpawnChain(Vec<SpawnChainBolt>);

    fn send_spawn_chain(
        mut flag: ResMut<SendSpawnChain>,
        mut writer: MessageWriter<SpawnChainBolt>,
    ) {
        for msg in flag.0.drain(..) {
            writer.write(msg);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<BoltConfig>()
            .init_resource::<GameRng>()
            .add_message::<SpawnChainBolt>()
            .insert_resource(SendSpawnChain(vec![]))
            .add_systems(FixedUpdate, (send_spawn_chain, spawn_chain_bolt).chain());
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // ── Behavior 9: Spawns tethered bolt at anchor position + offset ──

    #[test]
    fn chain_bolt_spawns_at_anchor_position_with_offset() {
        // Given: anchor bolt at (100, 50)
        // When: SpawnChainBolt message with tether_distance=200
        // Then: new bolt spawns near anchor position (at anchor + small offset)
        let mut app = test_app();

        let anchor = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(100.0, 50.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<SendSpawnChain>()
            .0
            .push(SpawnChainBolt {
                anchor,
                tether_distance: 200.0,
                source_chip: None,
            });
        tick(&mut app);

        // Should have 2 bolts total
        let bolt_count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(bolt_count, 2, "should have anchor + 1 chain bolt");

        // Chain bolt should be near the anchor position
        let chain_pos = app
            .world_mut()
            .query_filtered::<&Position2D, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .expect("chain bolt should have Position2D");
        let dist_from_anchor = (chain_pos.0 - Vec2::new(100.0, 50.0)).length();
        assert!(
            dist_from_anchor < 100.0,
            "chain bolt should spawn near anchor, distance={dist_from_anchor:.1}",
        );
    }

    // ── Behavior 10: Creates standalone DistanceConstraint entity ──

    #[test]
    fn chain_bolt_creates_standalone_distance_constraint() {
        // Given: anchor bolt entity
        // When: SpawnChainBolt message with tether_distance=200
        // Then: a DistanceConstraint entity is created with entity_a=anchor,
        //       entity_b=new bolt, max_distance=200
        let mut app = test_app();

        let anchor = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<SendSpawnChain>()
            .0
            .push(SpawnChainBolt {
                anchor,
                tether_distance: 200.0,
                source_chip: None,
            });
        tick(&mut app);

        // Find the constraint entity — extract owned data to avoid borrow conflicts
        let constraints: Vec<(Entity, DistanceConstraint)> = app
            .world_mut()
            .query::<(Entity, &DistanceConstraint)>()
            .iter(app.world())
            .map(|(e, dc)| (e, dc.clone()))
            .collect();
        assert_eq!(
            constraints.len(),
            1,
            "should have exactly one DistanceConstraint entity"
        );

        let (constraint_entity, constraint) = &constraints[0];
        assert_eq!(
            constraint.entity_a, anchor,
            "constraint entity_a should be the anchor"
        );
        assert!(
            (constraint.max_distance - 200.0).abs() < f32::EPSILON,
            "constraint max_distance should be 200.0, got {}",
            constraint.max_distance,
        );

        // Constraint entity should NOT have Bolt component
        assert!(
            app.world().get::<Bolt>(*constraint_entity).is_none(),
            "constraint entity should not have Bolt component"
        );

        // Constraint entity should have CleanupOnNodeExit
        assert!(
            app.world()
                .get::<CleanupOnNodeExit>(*constraint_entity)
                .is_some(),
            "constraint entity should have CleanupOnNodeExit"
        );

        // entity_b should be the new chain bolt
        let chain_bolt_entity = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .expect("chain bolt should exist");
        assert_eq!(
            constraint.entity_b, chain_bolt_entity,
            "constraint entity_b should be the chain bolt"
        );
    }

    // ── Behavior 11: Chain bolt has ExtraBolt + CleanupOnNodeExit + collision layers ──

    #[test]
    fn chain_bolt_has_extra_bolt_and_cleanup_and_collision_layers() {
        let mut app = test_app();

        let anchor = app
            .world_mut()
            .spawn((
                Bolt,
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .resource_mut::<SendSpawnChain>()
            .0
            .push(SpawnChainBolt {
                anchor,
                tether_distance: 200.0,
                source_chip: None,
            });
        tick(&mut app);

        let chain_bolt = app
            .world_mut()
            .query_filtered::<Entity, With<ExtraBolt>>()
            .iter(app.world())
            .next()
            .expect("chain bolt should exist");

        assert!(
            app.world().get::<Bolt>(chain_bolt).is_some(),
            "chain bolt should have Bolt marker"
        );
        assert!(
            app.world().get::<ExtraBolt>(chain_bolt).is_some(),
            "chain bolt should have ExtraBolt marker"
        );
        assert!(
            app.world().get::<CleanupOnNodeExit>(chain_bolt).is_some(),
            "chain bolt should have CleanupOnNodeExit"
        );
        assert!(
            app.world().get::<CollisionLayers>(chain_bolt).is_some(),
            "chain bolt should have CollisionLayers"
        );
        assert!(
            app.world().get::<Velocity2D>(chain_bolt).is_some(),
            "chain bolt should have Velocity2D"
        );
    }

    // ── Behavior 12: bolt=None (missing anchor) — no spawn ──

    #[test]
    fn no_spawn_when_anchor_bolt_missing() {
        // Given: SpawnChainBolt message referencing a despawned entity
        // When: spawn_chain_bolt runs
        // Then: no new bolt or constraint is created
        let mut app = test_app();

        let stale = app.world_mut().spawn_empty().id();
        app.world_mut().despawn(stale);
        app.world_mut()
            .resource_mut::<SendSpawnChain>()
            .0
            .push(SpawnChainBolt {
                anchor: stale,
                tether_distance: 200.0,
                source_chip: None,
            });
        tick(&mut app);

        let bolt_count = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .count();
        assert_eq!(bolt_count, 0, "no bolt should spawn when anchor is missing");

        let constraint_count = app
            .world_mut()
            .query::<&DistanceConstraint>()
            .iter(app.world())
            .count();
        assert_eq!(
            constraint_count, 0,
            "no constraint should spawn when anchor is missing"
        );
    }
}
