//! Invisible bottom wall that bounces bolt once, preventing bolt loss.

use std::collections::HashSet;

use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use crate::{
    bolt::messages::BoltImpactWall,
    shared::{BOLT_LAYER, PlayfieldConfig, WALL_LAYER},
    wall::components::{Wall, WallSize},
};

/// Marker for the invisible wall entity spawned by Second Wind.
#[derive(Component)]
pub struct SecondWindWall;

/// Spawns an invisible wall at the bottom of the playfield.
///
/// Permanent until used — bounces one bolt, then despawns.
pub(crate) fn fire(_entity: Entity, world: &mut World) {
    let playfield = world.resource::<PlayfieldConfig>();
    let bottom_y = playfield.bottom();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    let wall = world
        .spawn((
            SecondWindWall,
            Wall,
            WallSize {},
            Position2D(Vec2::new(0.0, bottom_y)),
            Scale2D {
                x: half_width,
                y: wall_ht,
            },
            Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, wall_ht)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
        ))
        .id();

    info!("spawned second wind wall {:?}", wall);
}

/// Despawns all `SecondWindWall` entities.
pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let walls: Vec<Entity> = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(world)
        .collect();

    for wall in walls {
        world.despawn(wall);
    }
}

/// Despawns a [`SecondWindWall`] entity after a bolt collides with it.
///
/// Reads [`BoltImpactWall`] messages and checks if the impacted wall
/// has the [`SecondWindWall`] marker. If so, despawns it — making the
/// `SecondWind` effect single-use.
///
/// Uses a `Local<HashSet<Entity>>` to track walls already scheduled for despawn
/// this frame, preventing double-despawn when two bolts hit the same wall.
fn despawn_second_wind_on_contact(
    mut commands: Commands,
    mut reader: MessageReader<BoltImpactWall>,
    wall_query: Query<Entity, With<SecondWindWall>>,
    mut despawned: Local<HashSet<Entity>>,
) {
    // Local<HashSet> reuses its heap allocation across frames — zero allocs after warmup.
    despawned.clear();
    for msg in reader.read() {
        if despawned.contains(&msg.wall) {
            continue;
        }
        if wall_query.get(msg.wall).is_ok() {
            despawned.insert(msg.wall);
            commands.entity(msg.wall).despawn();
        }
    }
}

/// Registers systems for `SecondWind` effect.
pub(crate) fn register(app: &mut App) {
    use crate::{bolt::BoltSystems, shared::playing_state::PlayingState};
    app.add_systems(
        FixedUpdate,
        despawn_second_wind_on_contact
            .run_if(in_state(PlayingState::Active))
            .after(BoltSystems::WallCollision),
    );
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
    use rantzsoft_spatial2d::components::{Position2D, Scale2D};

    use super::*;
    use crate::{
        shared::{BOLT_LAYER, PlayfieldConfig, WALL_LAYER},
        wall::components::Wall,
    };

    // ── Existing tests ──────────────────────────────────────────────

    #[test]
    fn fire_spawns_second_wind_wall() {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let entity = world.spawn_empty().id();

        fire(entity, &mut world);

        let walls: Vec<Entity> = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            walls.len(),
            1,
            "fire should spawn one SecondWindWall entity"
        );
    }

    #[test]
    fn reverse_despawns_all_second_wind_walls() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        world.spawn((
            SecondWindWall,
            Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
        ));
        world.spawn((
            SecondWindWall,
            Transform::from_translation(Vec3::new(0.0, -300.0, 0.0)),
        ));

        reverse(entity, &mut world);

        let remaining: Vec<Entity> = world
            .query_filtered::<Entity, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert!(
            remaining.is_empty(),
            "reverse should despawn all SecondWindWall entities"
        );
    }

    // =========================================================================
    // Wave 4B: SecondWind Collision Components
    // =========================================================================

    #[test]
    fn fire_spawns_wall_with_wall_marker() {
        // Behavior 13: SecondWind fire spawns wall with Wall marker component.
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let entity = world.spawn_empty().id();

        fire(entity, &mut world);

        let count = world
            .query_filtered::<Entity, (With<SecondWindWall>, With<Wall>)>()
            .iter(&world)
            .count();
        assert_eq!(
            count, 1,
            "SecondWind wall should have Wall marker component"
        );
    }

    #[test]
    fn fire_spawns_wall_with_position2d_at_playfield_bottom() {
        // Behavior 14: SecondWind wall position at playfield bottom.
        // Given: PlayfieldConfig::default() (width=800, height=600, bottom()=-300.0).
        // When: fire spawns the wall
        // Then: Position2D is Vec2::new(0.0, -300.0).
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let entity = world.spawn_empty().id();

        fire(entity, &mut world);

        let positions: Vec<Vec2> = world
            .query_filtered::<&Position2D, With<SecondWindWall>>()
            .iter(&world)
            .map(|p| p.0)
            .collect();
        assert_eq!(positions.len(), 1, "should spawn one wall with Position2D");
        let pos = positions[0];
        assert!(
            (pos.x - 0.0).abs() < f32::EPSILON,
            "SecondWind wall x should be 0.0, got {:.1}",
            pos.x
        );
        assert!(
            (pos.y - (-300.0)).abs() < f32::EPSILON,
            "SecondWind wall y should be -300.0 (playfield bottom), got {:.1}",
            pos.y
        );
    }

    #[test]
    fn fire_spawns_wall_with_collision_layers_matching_other_walls() {
        // Behavior 15: SecondWind wall has CollisionLayers matching other walls.
        // membership: WALL_LAYER (0x04), mask: BOLT_LAYER (0x01).
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        let entity = world.spawn_empty().id();

        fire(entity, &mut world);

        let layers: Vec<&CollisionLayers> = world
            .query_filtered::<&CollisionLayers, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(
            layers.len(),
            1,
            "wall should have CollisionLayers component"
        );
        assert_eq!(
            layers[0].membership, WALL_LAYER,
            "wall membership should be WALL_LAYER (0x{WALL_LAYER:02X}), got 0x{:02X}",
            layers[0].membership
        );
        assert_eq!(
            layers[0].mask, BOLT_LAYER,
            "wall mask should be BOLT_LAYER (0x{BOLT_LAYER:02X}), got 0x{:02X}",
            layers[0].mask
        );
    }

    #[test]
    fn fire_spawns_wall_with_aabb2d_spanning_playfield_width() {
        // Behavior 16: SecondWind wall has Aabb2D spanning playfield width.
        // Given: PlayfieldConfig::default() (width=800, half_width=400).
        // Then: Aabb2D center=Vec2::ZERO, half_extents=(400.0, wall_half_thickness).
        let mut world = World::new();
        let playfield = PlayfieldConfig::default();
        world.insert_resource(playfield.clone());
        let entity = world.spawn_empty().id();
        let half_width = playfield.width / 2.0;
        let wall_ht = playfield.wall_half_thickness();

        fire(entity, &mut world);

        let aabbs: Vec<&Aabb2D> = world
            .query_filtered::<&Aabb2D, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(aabbs.len(), 1, "wall should have Aabb2D component");
        let aabb = aabbs[0];
        assert_eq!(
            aabb.center,
            Vec2::ZERO,
            "Aabb2D center should be Vec2::ZERO, got {:?}",
            aabb.center
        );
        assert!(
            (aabb.half_extents.x - half_width).abs() < f32::EPSILON,
            "Aabb2D half_extents.x should be {half_width:.1}, got {:.1}",
            aabb.half_extents.x
        );
        assert!(
            (aabb.half_extents.y - wall_ht).abs() < f32::EPSILON,
            "Aabb2D half_extents.y should be {wall_ht:.1}, got {:.1}",
            aabb.half_extents.y
        );
    }

    #[test]
    fn fire_spawns_wall_with_scale2d_matching_playfield() {
        // Behavior 13: Scale2D matches playfield half-width and wall half-thickness.
        let mut world = World::new();
        let playfield = PlayfieldConfig::default();
        world.insert_resource(playfield.clone());
        let entity = world.spawn_empty().id();
        let half_width = playfield.width / 2.0;
        let wall_ht = playfield.wall_half_thickness();

        fire(entity, &mut world);

        let scales: Vec<&Scale2D> = world
            .query_filtered::<&Scale2D, With<SecondWindWall>>()
            .iter(&world)
            .collect();
        assert_eq!(scales.len(), 1, "wall should have Scale2D component");
        let scale = scales[0];
        assert!(
            (scale.x - half_width).abs() < f32::EPSILON,
            "Scale2D.x should be {half_width:.1}, got {:.1}",
            scale.x
        );
        assert!(
            (scale.y - wall_ht).abs() < f32::EPSILON,
            "Scale2D.y should be {wall_ht:.1}, got {:.1}",
            scale.y
        );
    }

    // ── Despawn on contact tests ────────────────────────────────────

    fn despawn_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<BoltImpactWall>()
            .add_systems(FixedUpdate, despawn_second_wind_on_contact);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[derive(Resource, Default)]
    struct TestBoltImpactWallMessages(Vec<BoltImpactWall>);

    fn enqueue_bolt_impact_wall(
        msg_res: Res<TestBoltImpactWallMessages>,
        mut writer: MessageWriter<BoltImpactWall>,
    ) {
        for msg in &msg_res.0 {
            writer.write(msg.clone());
        }
    }

    #[test]
    fn despawn_second_wind_wall_on_bolt_impact() {
        // Behavior 17: SecondWind wall despawned on first bolt impact via BoltImpactWall.
        // Given: SecondWind wall entity with SecondWindWall marker.
        //        BoltImpactWall { bolt, wall } message.
        // When: despawn_second_wind_on_contact runs
        // Then: Wall entity is despawned.
        let mut app = despawn_test_app();

        let bolt = app.world_mut().spawn_empty().id();
        let wall = app
            .world_mut()
            .spawn((SecondWindWall, Transform::default()))
            .id();

        app.init_resource::<TestBoltImpactWallMessages>();
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0 = vec![BoltImpactWall { bolt, wall }];
        app.add_systems(
            FixedUpdate,
            enqueue_bolt_impact_wall.before(despawn_second_wind_on_contact),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_err(),
            "SecondWind wall should be despawned after bolt impact"
        );
    }

    #[test]
    fn despawn_only_second_wind_wall_not_regular_walls() {
        // Behavior 17: Only SecondWind walls despawned — other walls unaffected.
        let mut app = despawn_test_app();

        let bolt = app.world_mut().spawn_empty().id();
        let sw_wall = app
            .world_mut()
            .spawn((SecondWindWall, Transform::default()))
            .id();
        let regular_wall = app.world_mut().spawn(Transform::default()).id();

        app.init_resource::<TestBoltImpactWallMessages>();
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0 = vec![
            BoltImpactWall {
                bolt,
                wall: sw_wall,
            },
            BoltImpactWall {
                bolt,
                wall: regular_wall,
            },
        ];
        app.add_systems(
            FixedUpdate,
            enqueue_bolt_impact_wall.before(despawn_second_wind_on_contact),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(sw_wall).is_err(),
            "SecondWind wall should be despawned"
        );
        assert!(
            app.world().get_entity(regular_wall).is_ok(),
            "regular wall should NOT be despawned"
        );
    }

    #[test]
    fn despawn_second_wind_wall_two_bolts_same_frame() {
        // Behavior 17 edge case: Two bolts hit SecondWind wall in same frame.
        // Wall is despawned once. Second message silently skipped.
        let mut app = despawn_test_app();

        let bolt_a = app.world_mut().spawn_empty().id();
        let bolt_b = app.world_mut().spawn_empty().id();
        let wall = app
            .world_mut()
            .spawn((SecondWindWall, Transform::default()))
            .id();

        app.init_resource::<TestBoltImpactWallMessages>();
        app.world_mut()
            .resource_mut::<TestBoltImpactWallMessages>()
            .0 = vec![
            BoltImpactWall { bolt: bolt_a, wall },
            BoltImpactWall { bolt: bolt_b, wall },
        ];
        app.add_systems(
            FixedUpdate,
            enqueue_bolt_impact_wall.before(despawn_second_wind_on_contact),
        );
        tick(&mut app);

        assert!(
            app.world().get_entity(wall).is_err(),
            "SecondWind wall should be despawned even with two impacts"
        );
    }
}
