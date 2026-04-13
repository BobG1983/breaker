//! `SpawnPhantomConfig` — spawn phantom bolt with limited lifetime.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_physics2d::collision_layers::CollisionLayers;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
use serde::{Deserialize, Serialize};

use super::components::{PhantomBolt, PhantomLifetime, PhantomOwner};
use crate::{
    bolt::components::{Bolt, ExtraBolt},
    effect_v3::traits::Fireable,
    shared::{BOLT_LAYER, BREAKER_LAYER, WALL_LAYER},
};

/// Configuration for spawning a temporary phantom bolt.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpawnPhantomConfig {
    /// How long the phantom bolt exists before despawning.
    pub duration:   OrderedFloat<f32>,
    /// Maximum phantom bolts from this source that can exist at once.
    pub max_active: u32,
}

impl Fireable for SpawnPhantomConfig {
    fn fire(&self, entity: Entity, _source: &str, world: &mut World) {
        // Count existing phantom bolts owned by this source
        let existing_count = world
            .query::<(&PhantomBolt, &PhantomOwner)>()
            .iter(world)
            .filter(|(_, owner)| owner.0 == entity)
            .count();

        if existing_count >= self.max_active as usize {
            return;
        }

        // Read source state
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let vel = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

        // Spawn phantom bolt directly with collision layers excluding CELL_LAYER
        world.spawn((
            Bolt,
            ExtraBolt,
            PhantomBolt,
            PhantomLifetime(self.duration.0),
            PhantomOwner(entity),
            Position2D(pos),
            Velocity2D(vel),
            CollisionLayers::new(BOLT_LAYER, WALL_LAYER | BREAKER_LAYER),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::tick_phantom_lifetime;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_phantom_lifetime.in_set(EffectV3Systems::Tick),
        );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_physics2d::collision_layers::CollisionLayers;
    use rantzsoft_spatial2d::components::{BaseSpeed, Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::components::{Bolt, ExtraBolt},
        effect_v3::{
            effects::phantom_bolt::components::{PhantomBolt, PhantomLifetime, PhantomOwner},
            traits::Fireable,
        },
        shared::{BREAKER_LAYER, CELL_LAYER, WALL_LAYER, rng::GameRng},
    };

    fn spawn_source(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
        world
            .spawn((Bolt, Position2D(pos), Velocity2D(vel), BaseSpeed(400.0)))
            .id()
    }

    #[test]
    fn fire_spawns_one_phantom_bolt() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let phantom_count = world
            .query_filtered::<Entity, With<PhantomBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            phantom_count, 1,
            "should spawn exactly 1 PhantomBolt entity"
        );
    }

    #[test]
    fn spawned_phantom_is_at_source_position() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(75.0, 150.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let positions: Vec<Vec2> = world
            .query_filtered::<&Position2D, With<PhantomBolt>>()
            .iter(&world)
            .map(|p| p.0)
            .collect();
        assert_eq!(positions.len(), 1);
        assert!(
            (positions[0] - Vec2::new(75.0, 150.0)).length() < 1e-3,
            "phantom bolt should be at source position, got {:?}",
            positions[0],
        );
    }

    #[test]
    fn spawned_phantom_has_configured_lifetime() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(5.0),
            max_active: 1,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let lifetimes: Vec<f32> = world
            .query_filtered::<&PhantomLifetime, With<PhantomBolt>>()
            .iter(&world)
            .map(|l| l.0)
            .collect();
        assert_eq!(lifetimes.len(), 1);
        assert!(
            (lifetimes[0] - 5.0).abs() < 1e-3,
            "PhantomLifetime should be ~5.0, got {}",
            lifetimes[0],
        );
    }

    #[test]
    fn spawned_phantom_has_owner_pointing_to_source() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let owners: Vec<Entity> = world
            .query_filtered::<&PhantomOwner, With<PhantomBolt>>()
            .iter(&world)
            .map(|o| o.0)
            .collect();
        assert_eq!(owners.len(), 1);
        assert_eq!(
            owners[0], source,
            "PhantomOwner should point to source entity"
        );
    }

    #[test]
    fn max_active_prevents_spawn_when_limit_reached() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        // Pre-spawn 2 phantom bolts owned by source
        world.spawn((PhantomBolt, PhantomOwner(source)));
        world.spawn((PhantomBolt, PhantomOwner(source)));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 2,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let phantom_count = world
            .query_filtered::<Entity, With<PhantomBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            phantom_count, 2,
            "should NOT spawn when max_active is reached"
        );
    }

    #[test]
    fn max_active_one_allows_spawn_when_no_existing_phantoms() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 1,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let phantom_count = world
            .query_filtered::<Entity, With<PhantomBolt>>()
            .iter(&world)
            .count();
        assert_eq!(
            phantom_count, 1,
            "max_active: 1 should allow spawn when no phantoms exist"
        );
    }

    #[test]
    fn max_active_counts_only_phantoms_owned_by_source() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source_a = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));
        let source_b = spawn_source(&mut world, Vec2::new(200.0, 200.0), Vec2::new(0.0, 400.0));

        // Pre-spawn 1 phantom owned by source_b
        world.spawn((PhantomBolt, PhantomOwner(source_b)));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 1,
        };
        config.fire(source_a, "phantom", &mut world);
        world.flush();

        let phantoms_for_a_count = world
            .query::<(&PhantomBolt, &PhantomOwner)>()
            .iter(&world)
            .filter(|(_, owner)| owner.0 == source_a)
            .count();
        assert_eq!(
            phantoms_for_a_count, 1,
            "phantom owned by B should not count against A's limit",
        );
    }

    #[test]
    fn spawned_phantom_has_extra_bolt_marker() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let extra_phantom_count = world
            .query_filtered::<Entity, (With<PhantomBolt>, With<ExtraBolt>)>()
            .iter(&world)
            .count();
        assert_eq!(
            extra_phantom_count, 1,
            "phantom bolt should have ExtraBolt marker"
        );
    }

    #[test]
    fn spawned_phantom_has_bolt_marker() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let bolt_phantom_count = world
            .query_filtered::<Entity, (With<PhantomBolt>, With<Bolt>)>()
            .iter(&world)
            .count();
        assert_eq!(
            bolt_phantom_count, 1,
            "phantom bolt should have Bolt marker"
        );
    }

    #[test]
    fn spawned_phantom_collision_layers_exclude_cell_layer() {
        let mut world = World::new();
        world.insert_resource(GameRng::from_seed(42));
        let source = spawn_source(&mut world, Vec2::new(100.0, 200.0), Vec2::new(0.0, 400.0));

        let config = SpawnPhantomConfig {
            duration:   OrderedFloat(2.0),
            max_active: 3,
        };
        config.fire(source, "phantom", &mut world);
        world.flush();

        let layers: Vec<&CollisionLayers> = world
            .query_filtered::<&CollisionLayers, With<PhantomBolt>>()
            .iter(&world)
            .collect();
        assert_eq!(layers.len(), 1, "phantom bolt should have CollisionLayers");
        assert_eq!(
            layers[0].mask & CELL_LAYER,
            0,
            "phantom bolt mask should NOT include CELL_LAYER",
        );
        assert_ne!(
            layers[0].mask & WALL_LAYER,
            0,
            "phantom bolt mask should include WALL_LAYER",
        );
        assert_ne!(
            layers[0].mask & BREAKER_LAYER,
            0,
            "phantom bolt mask should include BREAKER_LAYER",
        );
    }
}
