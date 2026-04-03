use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::helpers::*;
use crate::{
    shared::{BOLT_LAYER, WALL_LAYER},
    wall::components::Wall,
};

// ── Section A: fire() spawns timed visible floor wall ────────────────

// Behavior 1: fire() spawns a ShieldWall entity with Wall marker

#[test]
fn fire_spawns_shield_wall_with_wall_marker() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let count = world
        .query_filtered::<Entity, (With<ShieldWall>, With<Wall>)>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 1,
        "fire should spawn exactly one entity with both ShieldWall and Wall markers, got {count}"
    );
}

#[test]
fn fire_spawns_wall_with_short_duration() {
    // Edge case: duration 0.1 still spawns the wall (timer handles expiry, not fire)
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 0.1, 0.0, "parry", &mut world);

    let count = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        count, 1,
        "fire with duration 0.1 should still spawn a ShieldWall entity"
    );
}

// Behavior 2: fire() spawns wall at playfield bottom

#[test]
fn fire_spawns_wall_at_playfield_bottom() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let positions: Vec<Vec2> = world
        .query_filtered::<&Position2D, With<ShieldWall>>()
        .iter(&world)
        .map(|p| p.0)
        .collect();
    assert_eq!(positions.len(), 1, "should spawn one wall with Position2D");
    let pos = positions[0];
    assert!(
        (pos.x - 0.0).abs() < f32::EPSILON,
        "ShieldWall x should be 0.0, got {:.4}",
        pos.x
    );
    assert!(
        (pos.y - (-300.0)).abs() < f32::EPSILON,
        "ShieldWall y should be -300.0 (playfield bottom), got {:.4}",
        pos.y
    );
}

#[test]
fn fire_spawns_wall_at_custom_playfield_bottom() {
    // Edge case: custom PlayfieldConfig with bottom()=-500.0
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 1000.0,
        ..Default::default()
    });
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let positions: Vec<Vec2> = world
        .query_filtered::<&Position2D, With<ShieldWall>>()
        .iter(&world)
        .map(|p| p.0)
        .collect();
    assert_eq!(positions.len(), 1, "should spawn one wall with Position2D");
    let pos = positions[0];
    assert!(
        (pos.y - (-500.0)).abs() < f32::EPSILON,
        "ShieldWall y should be -500.0 for height=1000.0, got {:.4}",
        pos.y
    );
}

// Behavior 3: fire() spawns wall with correct collision physics

#[test]
fn fire_spawns_wall_with_collision_layers() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let layers: Vec<&CollisionLayers> = world
        .query_filtered::<&CollisionLayers, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(layers.len(), 1, "wall should have CollisionLayers");
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
fn fire_spawns_wall_with_aabb2d_spanning_playfield() {
    let mut world = test_world();
    let playfield = world.resource::<PlayfieldConfig>().clone();
    let entity = world.spawn_empty().id();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let aabbs: Vec<&Aabb2D> = world
        .query_filtered::<&Aabb2D, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(aabbs.len(), 1, "wall should have Aabb2D");
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
    let mut world = test_world();
    let playfield = world.resource::<PlayfieldConfig>().clone();
    let entity = world.spawn_empty().id();
    let half_width = playfield.width / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let scales: Vec<&Scale2D> = world
        .query_filtered::<&Scale2D, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(scales.len(), 1, "wall should have Scale2D");
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

#[test]
fn fire_spawns_wall_with_custom_playfield_extents() {
    // Edge case: PlayfieldConfig with width=1000.0, wall_thickness=200.0
    let mut world = World::new();
    world.insert_resource(PlayfieldConfig {
        width: 1000.0,
        wall_thickness: 200.0,
        ..Default::default()
    });
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<ColorMaterial>>();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let aabbs: Vec<&Aabb2D> = world
        .query_filtered::<&Aabb2D, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(aabbs.len(), 1, "wall should have Aabb2D");
    assert!(
        (aabbs[0].half_extents.x - 500.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.x should be 500.0, got {:.1}",
        aabbs[0].half_extents.x
    );
    assert!(
        (aabbs[0].half_extents.y - 100.0).abs() < f32::EPSILON,
        "Aabb2D half_extents.y should be 100.0, got {:.1}",
        aabbs[0].half_extents.y
    );
}

// Behavior 4: fire() spawns wall with visible mesh and material (blue HDR)

#[test]
fn fire_spawns_wall_with_mesh2d_and_material() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let has_mesh = world
        .query_filtered::<&Mesh2d, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(has_mesh, 1, "ShieldWall should have Mesh2d component");

    let has_material = world
        .query_filtered::<&MeshMaterial2d<ColorMaterial>, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        has_material, 1,
        "ShieldWall should have MeshMaterial2d<ColorMaterial> component"
    );
}

#[test]
fn fire_spawns_wall_with_blue_hdr_color() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let material_handles: Vec<&MeshMaterial2d<ColorMaterial>> = world
        .query_filtered::<&MeshMaterial2d<ColorMaterial>, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(material_handles.len(), 1, "should have material handle");

    let materials = world.resource::<Assets<ColorMaterial>>();
    let material = materials
        .get(material_handles[0].id())
        .expect("material handle should be valid");

    let expected_color = Color::srgb(0.3, 0.6, 2.0);
    assert_eq!(
        material.color, expected_color,
        "ShieldWall material color should be srgb(0.3, 0.6, 2.0), got {:?}",
        material.color
    );
}

// Behavior 5: fire() spawns wall with ShieldWallTimer component

#[test]
fn fire_spawns_wall_with_shield_wall_timer() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let timers: Vec<&ShieldWallTimer> = world
        .query_filtered::<&ShieldWallTimer, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(timers.len(), 1, "ShieldWall should have ShieldWallTimer");
    assert!(
        (timers[0].0.remaining_secs() - 5.0).abs() < 0.01,
        "timer remaining should be ~5.0, got {:.4}",
        timers[0].0.remaining_secs()
    );
    assert!(
        !timers[0].0.is_finished(),
        "timer should not be finished immediately after spawning"
    );
}

#[test]
fn fire_spawns_wall_with_short_timer() {
    // Edge case: fire with duration 0.5
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 0.5, 0.0, "parry", &mut world);

    let timers: Vec<&ShieldWallTimer> = world
        .query_filtered::<&ShieldWallTimer, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(timers.len(), 1, "ShieldWall should have ShieldWallTimer");
    assert!(
        (timers[0].0.remaining_secs() - 0.5).abs() < 0.01,
        "timer remaining should be ~0.5, got {:.4}",
        timers[0].0.remaining_secs()
    );
}

// Behavior 6: fire() resets timer in-place when ShieldWall already exists

#[test]
fn fire_resets_timer_when_shield_wall_already_exists() {
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    // Pre-spawn a ShieldWall with a partially-elapsed timer
    let mut timer = Timer::from_seconds(3.0, TimerMode::Once);
    timer.tick(std::time::Duration::from_secs_f32(1.5));
    let wall_entity = world.spawn((ShieldWall, ShieldWallTimer(timer))).id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    // Exactly one ShieldWall should exist
    let wall_count = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .count();
    assert_eq!(
        wall_count, 1,
        "should still have exactly one ShieldWall entity after re-fire, got {wall_count}"
    );

    // Entity ID should be preserved (same wall, not despawned and respawned)
    let remaining_walls: Vec<Entity> = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(
        remaining_walls[0], wall_entity,
        "ShieldWall entity ID should be preserved after re-fire"
    );

    // Timer should be reset to new duration
    let timer_val = world.get::<ShieldWallTimer>(wall_entity).unwrap();
    assert!(
        (timer_val.0.remaining_secs() - 5.0).abs() < 0.01,
        "timer should be reset to ~5.0, got {:.4}",
        timer_val.0.remaining_secs()
    );
    assert!(
        !timer_val.0.is_finished(),
        "timer should not be finished after reset"
    );
}

#[test]
fn fire_resets_nearly_expired_timer() {
    // Edge case: existing timer at remaining ~0.001 (nearly expired) resets to 5.0
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
    timer.tick(std::time::Duration::from_secs_f32(0.999));
    world.spawn((ShieldWall, ShieldWallTimer(timer)));

    fire(entity, 5.0, 0.0, "parry", &mut world);

    let timers: Vec<&ShieldWallTimer> = world
        .query_filtered::<&ShieldWallTimer, With<ShieldWall>>()
        .iter(&world)
        .collect();
    assert_eq!(timers.len(), 1);
    assert!(
        (timers[0].0.remaining_secs() - 5.0).abs() < 0.01,
        "nearly-expired timer should be reset to ~5.0, got {:.4}",
        timers[0].0.remaining_secs()
    );
}

// Behavior 7: fire() does NOT insert ShieldActive on the target entity

#[test]
fn fire_does_not_insert_shield_active_on_target() {
    // ShieldActive no longer exists as a type. This test verifies the target entity
    // gets no shield-related component besides those on the wall entity itself.
    let mut world = test_world();
    let entity = world.spawn_empty().id();

    fire(entity, 5.0, 0.0, "parry", &mut world);

    // The target entity should NOT have ShieldWall — that's on a separate wall entity
    assert!(
        world.get::<ShieldWall>(entity).is_none(),
        "fire should not put ShieldWall on the target entity itself"
    );
}
