//! `ShieldConfig` — shield wall protection.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
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

/// Configuration for a temporary shield wall that reflects bolts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShieldConfig {
    /// How long the shield wall lasts in seconds.
    pub duration:        OrderedFloat<f32>,
    /// Seconds subtracted from the shield's remaining time each time a bolt bounces off it.
    pub reflection_cost: OrderedFloat<f32>,
}

impl Fireable for ShieldConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Check for existing shield with same owner — reset duration instead of spawning.
        let existing: Option<Entity> = world
            .query_filtered::<(Entity, &ShieldOwner), With<ShieldWall>>()
            .iter(world)
            .find(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e);

        if let Some(existing_shield) = existing {
            if let Some(mut duration) = world.get_mut::<ShieldDuration>(existing_shield) {
                duration.0 = self.duration.0;
            }
            return;
        }

        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });
        let playfield = world.resource::<PlayfieldConfig>().clone();

        let mut commands = world.commands();
        let wall_entity = Wall::builder().floor(&playfield).spawn(&mut commands);
        commands.entity(wall_entity).insert((
            ShieldWall,
            ShieldOwner(entity),
            ShieldDuration(self.duration.0),
            ShieldReflectionCost(self.reflection_cost.0),
            chip,
        ));
    }

    fn register(app: &mut App) {
        use super::systems::{apply_shield_reflection_cost, tick_shield_duration};
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (tick_shield_duration, apply_shield_reflection_cost)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for ShieldConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        // Despawn all shield walls owned by this entity.
        let to_despawn: Vec<Entity> = world
            .query_filtered::<(Entity, &ShieldOwner), With<ShieldWall>>()
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
    use ordered_float::OrderedFloat;
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

    fn make_config() -> ShieldConfig {
        ShieldConfig {
            duration:        OrderedFloat(5.0),
            reflection_cost: OrderedFloat(0.5),
        }
    }

    // ── C5: Shield resets existing shield timer instead of spawning second ──

    #[test]
    fn shield_fire_resets_existing_shield_duration() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        // Spawn an existing shield with duration 1.0.
        world.spawn((
            ShieldWall,
            ShieldOwner(owner),
            ShieldDuration(1.0),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        // There should still be exactly 1 shield.
        let shields: Vec<(Entity, &ShieldDuration)> = world
            .query_filtered::<(Entity, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should have exactly 1 shield wall, not 2");
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "shield duration should be reset to 5.0, got {}",
            shields[0].1.0,
        );
    }

    #[test]
    fn shield_fire_resets_nearly_expired_shield() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        world.spawn((
            ShieldWall,
            ShieldOwner(owner),
            ShieldDuration(0.01),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<&ShieldDuration> = world
            .query_filtered::<&ShieldDuration, With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should have exactly 1 shield wall");
        assert!(
            (shields[0].0 - 5.0).abs() < f32::EPSILON,
            "nearly-expired shield should be reset to 5.0, got {}",
            shields[0].0,
        );
    }

    #[test]
    fn shield_fire_spawns_new_when_none_exists() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<(&ShieldOwner, &ShieldDuration, &ShieldReflectionCost)> = world
            .query_filtered::<(&ShieldOwner, &ShieldDuration, &ShieldReflectionCost), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 1, "should spawn exactly 1 shield wall");
        assert_eq!(shields[0].0.0, owner);
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "shield duration should be 5.0, got {}",
            shields[0].1.0,
        );
        assert!(
            (shields[0].2.0 - 0.5).abs() < f32::EPSILON,
            "shield reflection cost should be 0.5, got {}",
            shields[0].2.0,
        );
    }

    #[test]
    fn shield_fire_does_not_reset_another_owners_shield() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        // B's existing shield with duration 2.0.
        world.spawn((
            ShieldWall,
            ShieldOwner(owner_b),
            ShieldDuration(2.0),
            ShieldReflectionCost(0.5),
        ));

        make_config().fire(owner_a, "aegis", &mut world);
        world.flush();

        let shields: Vec<(&ShieldOwner, &ShieldDuration)> = world
            .query_filtered::<(&ShieldOwner, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            shields.len(),
            2,
            "should have 2 shield walls (A's new + B's existing)"
        );

        // Find B's shield and check it's unchanged.
        let b_shield = shields.iter().find(|(o, _)| o.0 == owner_b);
        assert!(b_shield.is_some(), "B's shield should still exist");
        assert!(
            (b_shield.unwrap().1.0 - 2.0).abs() < f32::EPSILON,
            "B's shield duration should remain 2.0, got {}",
            b_shield.unwrap().1.0,
        );

        // Find A's shield.
        let a_shield = shields.iter().find(|(o, _)| o.0 == owner_a);
        assert!(a_shield.is_some(), "A's new shield should exist");
        assert!(
            (a_shield.unwrap().1.0 - 5.0).abs() < f32::EPSILON,
            "A's shield duration should be 5.0, got {}",
            a_shield.unwrap().1.0,
        );
    }

    // ── Behavior 1: Shield fire() spawns a wall-layer entity with the full wall bundle ──

    #[test]
    fn shield_fire_spawns_wall_marker_and_bundle() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");

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
    fn shield_fire_places_markers_on_single_entity() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let count: usize = world
            .query_filtered::<Entity, (With<Wall>, With<ShieldWall>)>()
            .iter(&world)
            .count();
        assert_eq!(
            count, 1,
            "exactly 1 entity must carry both Wall and ShieldWall markers, got {count}",
        );
    }

    // ── Behavior 2: spawned entity carries all existing shield-specific markers ──

    #[test]
    fn shield_fire_carries_shield_specific_markers() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");

        let shield_owner = world
            .get::<ShieldOwner>(entity)
            .expect("must have ShieldOwner");
        assert_eq!(shield_owner.0, owner);

        let duration = world
            .get::<ShieldDuration>(entity)
            .expect("must have ShieldDuration");
        assert!(
            (duration.0 - 5.0).abs() < f32::EPSILON,
            "ShieldDuration.0 should be 5.0, got {}",
            duration.0,
        );

        let cost = world
            .get::<ShieldReflectionCost>(entity)
            .expect("must have ShieldReflectionCost");
        assert!(
            (cost.0 - 0.5).abs() < f32::EPSILON,
            "ShieldReflectionCost.0 should be 0.5, got {}",
            cost.0,
        );

        let chip = world
            .get::<EffectSourceChip>(entity)
            .expect("must have EffectSourceChip");
        assert_eq!(chip.0, Some("aegis".to_owned()));

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
    fn shield_fire_with_empty_source_sets_chip_none() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");

        let chip = world
            .get::<EffectSourceChip>(entity)
            .expect("must have EffectSourceChip");
        assert_eq!(
            chip.0, None,
            "empty source string must produce EffectSourceChip(None)",
        );
    }

    // ── Behavior 3: spawned entity sits at the playfield floor ──

    #[test]
    fn shield_fire_positions_entity_at_default_floor() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");

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
    fn shield_fire_positions_entity_at_custom_floor() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig {
            width: 1600.0,
            height: 900.0,
            ..PlayfieldConfig::default()
        });
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");

        let pos = world.get::<Position2D>(entity).unwrap();
        assert!(
            (pos.0.y - (-450.0)).abs() < f32::EPSILON,
            "Position2D.y should be -450.0, got {}",
            pos.0.y,
        );

        let aabb = world.get::<Aabb2D>(entity).unwrap();
        assert!(
            (aabb.half_extents.x - 800.0).abs() < f32::EPSILON,
            "Aabb2D.half_extents.x should be 800.0, got {}",
            aabb.half_extents.x,
        );

        let scale = world
            .get::<Scale2D>(entity)
            .expect("spawned entity must have Scale2D");
        assert!(
            (scale.x - 800.0).abs() < f32::EPSILON,
            "Scale2D.x should be 800.0, got {}",
            scale.x,
        );
        assert!(
            (scale.y - 90.0).abs() < f32::EPSILON,
            "Scale2D.y should be 90.0, got {}",
            scale.y,
        );
    }

    // ── Behavior 4: Reset-existing-shield behavior preserved (after migration) ──

    #[test]
    fn shield_fire_twice_resets_in_place_single_entity() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        ShieldConfig {
            duration:        OrderedFloat(1.0),
            reflection_cost: OrderedFloat(0.5),
        }
        .fire(owner, "aegis", &mut world);
        world.flush();

        // Capture the entity after the first fire().
        let first_entity = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity after first fire()");

        // Pre-condition: first fire() must produce a full wall-bundle entity
        // (behavior 1). Today's marker-only spawn does not, so this fails at RED.
        assert!(
            world.get::<Wall>(first_entity).is_some(),
            "first fire() must produce an entity with the Wall marker",
        );

        // Second fire() with a different duration.
        ShieldConfig {
            duration:        OrderedFloat(5.0),
            reflection_cost: OrderedFloat(0.5),
        }
        .fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<(Entity, &ShieldDuration)> = world
            .query_filtered::<(Entity, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            shields.len(),
            1,
            "second fire() must reset in place — exactly 1 ShieldWall should exist, got {}",
            shields.len(),
        );
        assert_eq!(
            shields[0].0, first_entity,
            "second fire() must reset in place — entity ID must be unchanged",
        );
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "ShieldDuration should be reset to 5.0, got {}",
            shields[0].1.0,
        );
    }

    // ── Behavior 5: Reset with nearly-expired shield (0.0 edge) ──

    #[test]
    fn shield_fire_resets_zero_duration_shield() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        // Overwrite duration to exactly 0.0 to drive the edge case.
        let existing = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");
        // Pre-condition: first fire() must produce a full wall-bundle entity
        // (behavior 1). Today's marker-only spawn does not, so this fails at RED.
        assert!(
            world.get::<Wall>(existing).is_some(),
            "first fire() must produce an entity with the Wall marker",
        );
        world.get_mut::<ShieldDuration>(existing).unwrap().0 = 0.0;

        // Second fire() — should still reset in place.
        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let shields: Vec<(Entity, &ShieldDuration)> = world
            .query_filtered::<(Entity, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            shields.len(),
            1,
            "exactly 1 shield should exist, got {}",
            shields.len(),
        );
        assert!(
            (shields[0].1.0 - 5.0).abs() < f32::EPSILON,
            "zero-duration shield should be reset to 5.0, got {}",
            shields[0].1.0,
        );
    }

    // ── Behavior 6: Multi-owner isolation preserved ──

    #[test]
    fn shield_multi_owner_produces_two_entities_both_with_wall_bundle() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        // B fires first.
        ShieldConfig {
            duration:        OrderedFloat(2.0),
            reflection_cost: OrderedFloat(0.5),
        }
        .fire(owner_b, "aegis", &mut world);
        world.flush();

        // A fires.
        ShieldConfig {
            duration:        OrderedFloat(5.0),
            reflection_cost: OrderedFloat(0.5),
        }
        .fire(owner_a, "aegis", &mut world);
        world.flush();

        let shields: Vec<(Entity, &ShieldOwner, &ShieldDuration)> = world
            .query_filtered::<(Entity, &ShieldOwner, &ShieldDuration), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(shields.len(), 2, "should have 2 shields, one per owner");

        let a_shield = shields
            .iter()
            .find(|(_, o, _)| o.0 == owner_a)
            .expect("owner_a shield should exist");
        let b_shield = shields
            .iter()
            .find(|(_, o, _)| o.0 == owner_b)
            .expect("owner_b shield should exist");

        assert!(
            (a_shield.2.0 - 5.0).abs() < f32::EPSILON,
            "A's duration should be 5.0, got {}",
            a_shield.2.0,
        );
        assert!(
            (b_shield.2.0 - 2.0).abs() < f32::EPSILON,
            "B's duration should remain 2.0, got {}",
            b_shield.2.0,
        );

        // Both entities must carry Wall + Aabb2D + CollisionLayers.
        for (entity, ..) in &shields {
            assert!(
                world.get::<Wall>(*entity).is_some(),
                "entity {entity:?} must have Wall marker",
            );
            assert!(
                world.get::<Aabb2D>(*entity).is_some(),
                "entity {entity:?} must have Aabb2D",
            );
            let layers = world
                .get::<CollisionLayers>(*entity)
                .expect("entity must have CollisionLayers");
            assert_eq!(layers.membership, WALL_LAYER);
            assert_eq!(layers.mask, BOLT_LAYER);
        }
    }

    // ── Behavior 7: reverse() despawns all wall entities owned by the entity ──

    #[test]
    fn shield_reverse_despawns_all_owned_walls() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        // Pre-condition: the spawned shield must be a real wall entity (part of
        // behavior 1's bundle). After migration, fire() spawns with the Wall
        // marker. Today it does not, so this assertion fails at the RED gate.
        let spawned = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .single(&world)
            .expect("should have exactly one ShieldWall entity");
        assert!(
            world.get::<Wall>(spawned).is_some(),
            "spawned shield must have Wall marker (fire() bundle migration)",
        );

        make_config().reverse(owner, "", &mut world);
        world.flush();

        let count = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .iter(&world)
            .count();
        assert_eq!(
            count, 0,
            "reverse() should despawn all ShieldWall entities, got {count}",
        );
    }

    #[test]
    fn shield_reverse_does_not_affect_other_owner() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner_a = world.spawn_empty().id();
        let owner_b = world.spawn_empty().id();

        make_config().fire(owner_a, "aegis", &mut world);
        world.flush();
        make_config().fire(owner_b, "aegis", &mut world);
        world.flush();

        make_config().reverse(owner_a, "", &mut world);
        world.flush();

        let shields: Vec<(Entity, &ShieldOwner)> = world
            .query_filtered::<(Entity, &ShieldOwner), With<ShieldWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            shields.len(),
            1,
            "only owner_b's shield should remain, got {}",
            shields.len(),
        );
        assert_eq!(shields[0].1.0, owner_b);

        let b_entity = shields[0].0;
        assert!(
            world.get::<Wall>(b_entity).is_some(),
            "B's shield must still have Wall marker",
        );
        assert!(
            world.get::<Aabb2D>(b_entity).is_some(),
            "B's shield must still have Aabb2D",
        );
        let layers = world
            .get::<CollisionLayers>(b_entity)
            .expect("B's shield must have CollisionLayers");
        assert_eq!(layers.membership, WALL_LAYER);
        assert_eq!(layers.mask, BOLT_LAYER);
    }

    // ── reverse_all_by_source (default delegation) ──────────────────

    #[test]
    fn reverse_all_by_source_despawns_shield_walls_via_default_delegation() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let owner = world.spawn_empty().id();

        make_config().fire(owner, "aegis", &mut world);
        world.flush();

        let count_before = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .iter(&world)
            .count();
        assert_eq!(count_before, 1, "should have 1 shield wall before reverse");

        make_config().reverse_all_by_source(owner, "aegis", &mut world);

        let count_after = world
            .query_filtered::<Entity, With<ShieldWall>>()
            .iter(&world)
            .count();
        assert_eq!(
            count_after, 0,
            "all ShieldWall entities should be despawned"
        );

        // Calling twice does not panic.
        make_config().reverse_all_by_source(owner, "aegis", &mut world);
    }
}
