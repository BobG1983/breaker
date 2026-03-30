use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::system::*;
use crate::{
    bolt::messages::BoltImpactWall,
    shared::{BOLT_LAYER, PlayfieldConfig, WALL_LAYER},
    wall::components::Wall,
};

// ── Existing tests ──────────────────────────────────────────────

#[test]
fn fire_spawns_second_wind_wall() {
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    fire(entity, "", &mut world);

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

    reverse(entity, "", &mut world);

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

    fire(entity, "", &mut world);

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

    fire(entity, "", &mut world);

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

    fire(entity, "", &mut world);

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

    fire(entity, "", &mut world);

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

    fire(entity, "", &mut world);

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

// ── Regression: fire() must not spawn duplicate walls ────────────

#[test]
fn fire_with_existing_wall_does_not_spawn_second() {
    // Regression: fire() spawned a wall unconditionally, allowing wall count > 1.
    // Given: A SecondWindWall entity already exists.
    // When: fire() is called again.
    // Then: Wall count remains 1 (no additional wall spawned).
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    // First fire — should spawn the wall
    fire(entity, "", &mut world);
    let count_after_first: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after_first, 1,
        "first fire should spawn exactly one SecondWindWall"
    );

    // Second fire — should NOT spawn another wall
    fire(entity, "", &mut world);
    let count_after_second: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after_second, 1,
        "second fire should not spawn another wall when one already exists, got {count_after_second}"
    );
}

#[test]
fn fire_without_existing_wall_spawns_wall() {
    // Positive companion: fire() spawns a wall when none exists.
    // Given: No SecondWindWall entities exist.
    // When: fire() is called.
    // Then: Exactly one SecondWindWall is spawned.
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig::default());
    let entity = world.spawn_empty().id();

    let count_before: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(count_before, 0, "precondition: no walls should exist");

    fire(entity, "", &mut world);

    let count_after: usize = world
        .query_filtered::<Entity, With<SecondWindWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count_after, 1,
        "fire should spawn exactly one SecondWindWall when none exists"
    );
}
