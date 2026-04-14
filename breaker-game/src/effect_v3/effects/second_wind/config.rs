//! `SecondWindConfig` — one-shot bottom wall.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    effect_v3::{
        components::EffectSourceChip,
        traits::{Fireable, Reversible},
    },
    shared::PlayfieldConfig,
    walls::components::Wall,
};

/// Spawns an invisible one-shot bottom wall. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SecondWindConfig {}

impl Fireable for SecondWindConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });
        let playfield = world.resource::<PlayfieldConfig>().clone();

        let mut commands = world.commands();
        let wall_entity = Wall::builder().floor(&playfield).spawn(&mut commands);
        commands
            .entity(wall_entity)
            .insert((SecondWindWall, SecondWindOwner(entity), chip));
    }

    fn register(app: &mut App) {
        use super::systems::despawn_on_first_reflection;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            despawn_on_first_reflection.in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for SecondWindConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        // Despawn all second-wind walls owned by this entity.
        let to_despawn: Vec<Entity> = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(world)
            .filter(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e)
            .collect();
        for e in to_despawn {
            world.despawn(e);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
    use rantzsoft_spatial2d::components::{Position2D, Scale2D};
    use rantzsoft_stateflow::CleanupOnExit;

    use super::*;
    use crate::{
        effect_v3::{
            components::EffectSourceChip,
            traits::{Fireable, Reversible},
        },
        shared::{BOLT_LAYER, GameDrawLayer, PlayfieldConfig, WALL_LAYER},
        state::types::NodeState,
        walls::components::Wall,
    };

    // ── Behavior 8: SecondWind fire() spawns a wall-layer entity with the full wall bundle ──

    #[test]
    fn second_wind_fire_spawns_wall_marker_and_bundle() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .single(&world)
            .expect("should have exactly one SecondWindWall entity");

        assert!(
            world.get::<Wall>(entity).is_some(),
            "spawned entity must have Wall marker",
        );
        assert!(
            world.get::<Position2D>(entity).is_some(),
            "spawned entity must have Position2D",
        );
        assert!(
            world.get::<Aabb2D>(entity).is_some(),
            "spawned entity must have Aabb2D",
        );

        let layers = world
            .get::<CollisionLayers>(entity)
            .expect("spawned entity must have CollisionLayers");
        assert_eq!(layers.membership, WALL_LAYER);
        assert_eq!(layers.mask, BOLT_LAYER);

        assert!(
            matches!(
                world.get::<GameDrawLayer>(entity),
                Some(GameDrawLayer::Wall)
            ),
            "spawned entity must have GameDrawLayer::Wall",
        );
    }

    #[test]
    fn second_wind_fire_places_markers_on_single_entity() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        // Exactly one entity should simultaneously carry Wall AND SecondWindWall.
        let count: usize = world
            .query_filtered::<Entity, (With<Wall>, With<SecondWindWall>)>()
            .iter(&world)
            .count();
        assert_eq!(
            count, 1,
            "exactly 1 entity must carry both Wall and SecondWindWall markers, got {count}",
        );
    }

    // ── Behavior 9: spawned entity carries all existing second-wind-specific markers ──

    #[test]
    fn second_wind_fire_carries_second_wind_markers() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .single(&world)
            .expect("should have exactly one SecondWindWall entity");

        let owner_marker = world
            .get::<SecondWindOwner>(entity)
            .expect("must have SecondWindOwner");
        assert_eq!(owner_marker.0, owner);

        let chip = world
            .get::<EffectSourceChip>(entity)
            .expect("must have EffectSourceChip");
        assert_eq!(chip.0, Some("last_stand".to_owned()));

        let cleanup_count: usize = world
            .query_filtered::<Entity, With<CleanupOnExit<NodeState>>>()
            .iter(&world)
            .filter(|e| *e == entity)
            .count();
        assert_eq!(
            cleanup_count, 1,
            "spawned entity must have CleanupOnExit::<NodeState>",
        );
    }

    #[test]
    fn second_wind_fire_with_empty_source_sets_chip_none() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .single(&world)
            .expect("should have exactly one SecondWindWall entity");

        let chip = world
            .get::<EffectSourceChip>(entity)
            .expect("must have EffectSourceChip");
        assert_eq!(
            chip.0, None,
            "empty source string must produce EffectSourceChip(None)",
        );
    }

    // ── Behavior 10: spawned entity sits at the playfield floor (default) ──

    #[test]
    fn second_wind_fire_positions_entity_at_default_floor() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .single(&world)
            .expect("should have exactly one SecondWindWall entity");

        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            pos.0.x.abs() < f32::EPSILON,
            "Position2D.x should be 0.0, got {}",
            pos.0.x,
        );
        assert!(
            (pos.0.y - PlayfieldConfig::default().bottom()).abs() < f32::EPSILON,
            "Position2D.y should be playfield.bottom() (-300.0), got {}",
            pos.0.y,
        );

        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 400.0).abs() < f32::EPSILON,
            "Aabb2D.half_extents.x should be 400.0, got {}",
            aabb.half_extents.x,
        );
        assert!(
            (aabb.half_extents.y - 90.0).abs() < f32::EPSILON,
            "Aabb2D.half_extents.y should be 90.0, got {}",
            aabb.half_extents.y,
        );

        let scale = world
            .get::<Scale2D>(entity)
            .expect("spawned entity must have Scale2D");
        assert!(
            (scale.x - 400.0).abs() < f32::EPSILON,
            "Scale2D.x should be 400.0, got {}",
            scale.x,
        );
        assert!(
            (scale.y - 90.0).abs() < f32::EPSILON,
            "Scale2D.y should be 90.0, got {}",
            scale.y,
        );
    }

    #[test]
    fn second_wind_fire_positions_entity_at_custom_floor() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig {
            width: 1600.0,
            height: 900.0,
            ..PlayfieldConfig::default()
        });
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .single(&world)
            .expect("should have exactly one SecondWindWall entity");

        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            (pos.0.y - (-450.0)).abs() < f32::EPSILON,
            "Position2D.y should be -450.0 for height 900.0, got {}",
            pos.0.y,
        );

        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 800.0).abs() < f32::EPSILON,
            "Aabb2D.half_extents.x should be 800.0 for width 1600.0, got {}",
            aabb.half_extents.x,
        );

        let scale = world
            .get::<Scale2D>(entity)
            .expect("spawned entity must have Scale2D");
        assert!(
            (scale.x - 800.0).abs() < f32::EPSILON,
            "Scale2D.x should be 800.0 for width 1600.0, got {}",
            scale.x,
        );
        assert!(
            (scale.y - 90.0).abs() < f32::EPSILON,
            "Scale2D.y should be 90.0, got {}",
            scale.y,
        );
    }

    // ── Behavior 11: repeated fire() spawns additional walls (no dedupe) ──

    #[test]
    fn second_wind_fire_twice_spawns_two_walls() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();
        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let walls: Vec<(Entity, &SecondWindOwner)> = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            walls.len(),
            2,
            "two fire() calls should produce two SecondWindWall entities, got {}",
            walls.len(),
        );
        assert!(
            walls.iter().all(|(_, o)| o.0 == owner),
            "all walls should be owned by the same owner",
        );
        // Pre-condition: each fire() must produce a full wall-bundle entity
        // (behavior 8). Today's marker-only spawn does not, so this fails at RED.
        for (entity, _) in &walls {
            assert!(
                world.get::<Wall>(*entity).is_some(),
                "each spawned second-wind wall must have the Wall marker",
            );
        }
    }

    #[test]
    fn second_wind_fire_twice_both_have_full_wall_bundle() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();
        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let entities: Vec<Entity> = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(entities.len(), 2, "should have two second-wind walls");

        for entity in entities {
            assert!(
                world.get::<Wall>(entity).is_some(),
                "entity {entity:?} must have Wall marker",
            );
            assert!(
                world.get::<Aabb2D>(entity).is_some(),
                "entity {entity:?} must have Aabb2D",
            );
            let layers = world
                .get::<CollisionLayers>(entity)
                .expect("entity must have CollisionLayers");
            assert_eq!(layers.membership, WALL_LAYER);
            assert_eq!(layers.mask, BOLT_LAYER);
        }
    }

    // ── Behavior 17: reverse() despawns all walls owned by the given entity ──

    #[test]
    fn second_wind_reverse_despawns_owners_walls_only() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner_a, "last_stand", &mut world);
        world.flush();
        SecondWindConfig {}.fire(owner_b, "last_stand", &mut world);
        world.flush();

        // Capture owner_a's wall entity ID so we can assert it is fully despawned
        // (not merely tombstoned) after reverse().
        let owner_a_wall: Entity = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(&world)
            .find(|(_, owner)| owner.0 == owner_a)
            .map(|(e, _)| e)
            .expect("owner_a should have a second-wind wall before reverse()");

        SecondWindConfig {}.reverse(owner_a, "", &mut world);
        world.flush();

        let walls: Vec<(Entity, &SecondWindOwner)> = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            walls.len(),
            1,
            "exactly 1 second-wind wall should remain, got {}",
            walls.len(),
        );
        assert_eq!(
            walls[0].1.0, owner_b,
            "the surviving wall must be owner_b's",
        );
        // Explicit check: owner_a's wall entity must be fully gone from the World,
        // not just filtered out of the query. Guards against a broken reverse()
        // that tombstones components but leaves the entity allocated.
        assert!(
            world.get_entity(owner_a_wall).is_err(),
            "owner_a's wall entity must be fully despawned after reverse()",
        );
        // Pre-condition: the surviving wall must have the Wall bundle (behavior 8).
        // Today's marker-only spawn does not include it, so this fails at RED.
        assert!(
            world.get::<Wall>(walls[0].0).is_some(),
            "surviving second-wind wall must have Wall marker",
        );
    }

    #[test]
    fn second_wind_reverse_on_owner_with_no_walls_is_noop() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner_a = world.spawn_empty().id();
        let owner_c = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner_a, "last_stand", &mut world);
        world.flush();

        // Must not panic.
        SecondWindConfig {}.reverse(owner_c, "", &mut world);
        world.flush();

        let walls: Vec<(Entity, &SecondWindOwner)> = world
            .query_filtered::<(Entity, &SecondWindOwner), With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(walls.len(), 1, "owner_a's wall should remain untouched");
        assert_eq!(walls[0].1.0, owner_a);
        // Pre-condition: owner_a's wall must have the Wall bundle (behavior 8).
        // Today's marker-only spawn does not include it, so this fails at RED.
        assert!(
            world.get::<Wall>(walls[0].0).is_some(),
            "owner_a's second-wind wall must have Wall marker",
        );
    }

    // ── reverse_all_by_source (default delegation) ──────────────────

    #[test]
    fn reverse_all_by_source_despawns_walls_via_default_delegation() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        SecondWindConfig {}.fire(owner, "last_stand", &mut world);
        world.flush();

        let count_before = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .count();
        assert_eq!(
            count_before, 1,
            "should have 1 second wind wall before reverse"
        );

        SecondWindConfig {}.reverse_all_by_source(owner, "last_stand", &mut world);

        let count_after = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .count();
        assert_eq!(
            count_after, 0,
            "all SecondWindWall entities should be despawned"
        );

        // Calling twice does not panic.
        SecondWindConfig {}.reverse_all_by_source(owner, "last_stand", &mut world);
    }
}
