//! Tests for tether beam visual components: `Spatial`, `Mesh2d`, `MeshMaterial2d`,
//! `GameDrawLayer::Fx` at spawn, and `sync_tether_beam_visual` system (`Position2D`,
//! `Scale2D`, `Rotation2D` sync).

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Rotation2D, Scale2D, Spatial};

use super::helpers::*;
use crate::shared::GameDrawLayer;

// ── Helper: App with asset resources + BoltRegistry for fire()-based tests ──

fn visual_fire_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    let mut registry = BoltRegistry::default();
    registry.insert(
        "Bolt".to_string(),
        BoltDefinition {
            name: "Bolt".to_owned(),
            base_speed: 400.0,
            min_speed: 200.0,
            max_speed: 800.0,
            radius: 8.0,
            base_damage: 10.0,
            effects: vec![],
            color_rgb: [6.0, 5.0, 0.5],
            min_angle_horizontal: 5.0,
            min_angle_vertical: 5.0,
            min_radius: None,
            max_radius: None,
        },
    );
    app.insert_resource(registry);
    app.insert_resource(GameRng::default());
    app.update();
    app
}

// ── Helper: App with sync_tether_beam_visual registered ──

fn visual_sync_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, sync_tether_beam_visual);
    app
}

/// Spawn two bolt entities and a beam entity referencing them.
fn spawn_beam_with_bolts(app: &mut App, pos_a: Vec2, pos_b: Vec2) -> (Entity, Entity, Entity) {
    let bolt_a = app.world_mut().spawn((Bolt, Position2D(pos_a))).id();
    let bolt_b = app.world_mut().spawn((Bolt, Position2D(pos_b))).id();
    let beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult: 1.5,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            Spatial::builder().at_position(Vec2::ZERO).build(),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();
    (bolt_a, bolt_b, beam)
}

// ── Behavior 31: fire_standard() spawns beam with Spatial marker ──

#[test]
fn fire_standard_spawns_beam_with_spatial_marker() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 1.5, false, "", app.world_mut());

    let mut query = app.world_mut().query::<(&TetherBeamComponent, &Spatial)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one beam entity with Spatial marker");
}

// ── Behavior 32: fire_standard() spawns beam with Mesh2d ──

#[test]
fn fire_standard_spawns_beam_with_mesh2d() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 1.5, false, "", app.world_mut());

    let mut query = app.world_mut().query::<(&TetherBeamComponent, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 1, "expected one beam entity with Mesh2d");
}

#[test]
fn fire_standard_twice_spawns_two_beams_each_with_mesh2d() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 1.5, false, "", app.world_mut());
    fire(entity, 1.5, false, "", app.world_mut());

    let mut query = app.world_mut().query::<(&TetherBeamComponent, &Mesh2d)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 2,
        "two fire() calls should produce two beams with Mesh2d"
    );
}

// ── Behavior 33: fire_standard() spawns beam with MeshMaterial2d<ColorMaterial> ──

#[test]
fn fire_standard_spawns_beam_with_mesh_material() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 1.5, false, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&TetherBeamComponent, &MeshMaterial2d<ColorMaterial>)>();
    let count = query.iter(app.world()).count();
    assert_eq!(
        count, 1,
        "expected one beam entity with MeshMaterial2d<ColorMaterial>"
    );
}

// ── Behavior 34: fire_standard() spawns beam with GameDrawLayer::Fx ──

#[test]
fn fire_standard_spawns_beam_with_game_draw_layer_fx() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    fire(entity, 1.5, false, "", app.world_mut());

    let mut query = app
        .world_mut()
        .query::<(&TetherBeamComponent, &GameDrawLayer)>();
    let results: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(results.len(), 1, "expected one beam with GameDrawLayer");

    let (_, draw_layer) = results[0];
    assert!(
        matches!(draw_layer, GameDrawLayer::Fx),
        "expected GameDrawLayer::Fx, got {draw_layer:?}",
    );
}

// ── Behavior 35: fire_chain() spawns chain beams with all visual components ──

#[test]
fn fire_chain_spawns_chain_beams_with_visual_components() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    // Spawn 3 bolt entities so fire_chain creates 2 chain beams
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(0.0, 0.0))));
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(50.0, 0.0))));
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(100.0, 0.0))));

    fire(entity, 1.5, true, "", app.world_mut());

    // Check chain beams for Spatial
    let mut spatial_query = app.world_mut().query::<(&TetherChainBeam, &Spatial)>();
    let spatial_count = spatial_query.iter(app.world()).count();
    assert_eq!(spatial_count, 2, "expected 2 chain beams with Spatial");

    // Check chain beams for Mesh2d
    let mut mesh_query = app.world_mut().query::<(&TetherChainBeam, &Mesh2d)>();
    let mesh_count = mesh_query.iter(app.world()).count();
    assert_eq!(mesh_count, 2, "expected 2 chain beams with Mesh2d");

    // Check chain beams for MeshMaterial2d
    let mut mat_query = app
        .world_mut()
        .query::<(&TetherChainBeam, &MeshMaterial2d<ColorMaterial>)>();
    let mat_count = mat_query.iter(app.world()).count();
    assert_eq!(mat_count, 2, "expected 2 chain beams with MeshMaterial2d");

    // Check chain beams for GameDrawLayer::Fx
    let mut dl_query = app
        .world_mut()
        .query::<(&TetherChainBeam, &GameDrawLayer)>();
    let dl_results: Vec<_> = dl_query.iter(app.world()).collect();
    assert_eq!(
        dl_results.len(),
        2,
        "expected 2 chain beams with GameDrawLayer"
    );
    for (_, draw_layer) in dl_results {
        assert!(
            matches!(draw_layer, GameDrawLayer::Fx),
            "expected GameDrawLayer::Fx, got {draw_layer:?}"
        );
    }
}

#[test]
fn fire_chain_with_single_bolt_spawns_no_beams() {
    let mut app = visual_fire_app();
    let entity = app
        .world_mut()
        .spawn(Position2D(Vec2::new(100.0, 200.0)))
        .id();

    // Only 1 bolt -- windows(2) produces no pairs
    app.world_mut().spawn((Bolt, Position2D(Vec2::ZERO)));

    fire(entity, 1.5, true, "", app.world_mut());

    let mut query = app.world_mut().query::<&TetherChainBeam>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 0, "single bolt should produce no chain beams");
}

// ── Behavior 36: maintain_tether_chain() spawns rebuilt beams with visual components ──

#[test]
fn maintain_tether_chain_spawns_rebuilt_beams_with_visual_components() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()));
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.add_systems(Update, maintain_tether_chain);

    // Insert TetherChainActive with last_bolt_count=2
    app.insert_resource(TetherChainActive {
        damage_mult: 1.5,
        effective_damage_multiplier: 1.0,
        base_damage: DEFAULT_BOLT_BASE_DAMAGE,
        source_chip: None,
        last_bolt_count: 2,
    });

    // Spawn 3 bolts (one more than last_bolt_count, triggering rebuild)
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(0.0, 0.0))));
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(50.0, 0.0))));
    app.world_mut()
        .spawn((Bolt, Position2D(Vec2::new(100.0, 0.0))));

    app.update();

    // Check rebuilt chain beams for Spatial
    let mut spatial_query = app.world_mut().query::<(&TetherChainBeam, &Spatial)>();
    let spatial_count = spatial_query.iter(app.world()).count();
    assert_eq!(
        spatial_count, 2,
        "expected 2 rebuilt chain beams with Spatial"
    );

    // Check rebuilt chain beams for Mesh2d
    let mut mesh_query = app.world_mut().query::<(&TetherChainBeam, &Mesh2d)>();
    let mesh_count = mesh_query.iter(app.world()).count();
    assert_eq!(mesh_count, 2, "expected 2 rebuilt chain beams with Mesh2d");

    // Check rebuilt chain beams for MeshMaterial2d
    let mut mat_query = app
        .world_mut()
        .query::<(&TetherChainBeam, &MeshMaterial2d<ColorMaterial>)>();
    let mat_count = mat_query.iter(app.world()).count();
    assert_eq!(
        mat_count, 2,
        "expected 2 rebuilt chain beams with MeshMaterial2d"
    );

    // Check rebuilt chain beams for GameDrawLayer::Fx
    let mut dl_query = app
        .world_mut()
        .query::<(&TetherChainBeam, &GameDrawLayer)>();
    let dl_count = dl_query.iter(app.world()).count();
    assert_eq!(
        dl_count, 2,
        "expected 2 rebuilt chain beams with GameDrawLayer"
    );
}

// ── Behavior 37: sync_tether_beam_visual sets Position2D to midpoint ──

#[test]
fn sync_tether_beam_visual_sets_position_to_midpoint() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) = spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));

    app.update();

    let pos = app.world().get::<Position2D>(beam).unwrap();
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "expected midpoint x=50.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 0.0).abs() < f32::EPSILON,
        "expected midpoint y=0.0, got {}",
        pos.0.y
    );
}

#[test]
fn sync_tether_beam_visual_midpoint_when_bolts_at_same_position() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) =
        spawn_beam_with_bolts(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0));

    app.update();

    let pos = app.world().get::<Position2D>(beam).unwrap();
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "expected midpoint x=50.0 when bolts at same position, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 50.0).abs() < f32::EPSILON,
        "expected midpoint y=50.0 when bolts at same position, got {}",
        pos.0.y
    );
}

// ── Behavior 38: sync_tether_beam_visual sets Scale2D for beam length and width ──

#[test]
fn sync_tether_beam_visual_sets_scale_for_beam_dimensions() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) = spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));

    app.update();

    let scale = app.world().get::<Scale2D>(beam).unwrap();
    assert!(
        (scale.x - 100.0).abs() < f32::EPSILON,
        "expected Scale2D.x=100.0 (beam length), got {}",
        scale.x
    );
    assert!(
        (scale.y - 4.0).abs() < f32::EPSILON,
        "expected Scale2D.y=4.0 (beam width), got {}",
        scale.y
    );
}

#[test]
fn sync_tether_beam_visual_zero_length_when_bolts_at_same_position() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) =
        spawn_beam_with_bolts(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0));

    app.update();

    let scale = app.world().get::<Scale2D>(beam).unwrap();
    assert!(
        scale.x.abs() < f32::EPSILON,
        "expected Scale2D.x=0.0 when bolts at same position, got {}",
        scale.x
    );
    assert!(
        (scale.y - 4.0).abs() < f32::EPSILON,
        "expected Scale2D.y=4.0 (beam width always 4.0), got {}",
        scale.y
    );
}

// ── Behavior 39: sync_tether_beam_visual sets Rotation2D to angle between bolts ──

#[test]
fn sync_tether_beam_visual_sets_rotation_for_vertical_beam() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) = spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(0.0, 100.0));

    app.update();

    let rotation = app.world().get::<Rotation2D>(beam).unwrap();
    let angle = rotation.0.as_radians();
    assert!(
        (angle - std::f32::consts::FRAC_PI_2).abs() < 0.01,
        "expected rotation ~PI/2 (vertical beam), got {angle}",
    );
}

#[test]
fn sync_tether_beam_visual_sets_rotation_for_horizontal_beam() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) = spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));

    app.update();

    let rotation = app.world().get::<Rotation2D>(beam).unwrap();
    let angle = rotation.0.as_radians();
    assert!(
        angle.abs() < 0.01,
        "expected rotation ~0 (horizontal beam), got {angle}",
    );
}

// ── Behavior 40: sync_tether_beam_visual for diagonal beam ──

#[test]
fn sync_tether_beam_visual_diagonal_beam() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) =
        spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));

    app.update();

    let pos = app.world().get::<Position2D>(beam).unwrap();
    assert!(
        (pos.0.x - 50.0).abs() < f32::EPSILON,
        "expected midpoint x=50.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 50.0).abs() < f32::EPSILON,
        "expected midpoint y=50.0, got {}",
        pos.0.y
    );

    let scale = app.world().get::<Scale2D>(beam).unwrap();
    let expected_length = 100.0_f32.hypot(100.0_f32);
    assert!(
        (scale.x - expected_length).abs() < 0.01,
        "expected Scale2D.x ~= {} (diagonal length), got {}",
        expected_length,
        scale.x
    );

    let rotation = app.world().get::<Rotation2D>(beam).unwrap();
    let angle = rotation.0.as_radians();
    assert!(
        (angle - std::f32::consts::FRAC_PI_4).abs() < 0.01,
        "expected rotation ~PI/4 (45 degrees), got {angle}",
    );
}

#[test]
fn sync_tether_beam_visual_negative_diagonal_beam() {
    let mut app = visual_sync_test_app();
    let (_, _, beam) =
        spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, -100.0));

    app.update();

    let rotation = app.world().get::<Rotation2D>(beam).unwrap();
    let angle = rotation.0.as_radians();
    assert!(
        (angle - (-std::f32::consts::FRAC_PI_4)).abs() < 0.01,
        "expected rotation ~-PI/4 (-45 degrees), got {angle}",
    );
}

// ── Behavior 41: sync_tether_beam_visual skips when bolt_a is missing ──

#[test]
fn sync_tether_beam_visual_skips_when_bolt_a_missing() {
    let mut app = visual_sync_test_app();

    // Spawn bolt_b but use a fake entity for bolt_a
    let fake_bolt_a = Entity::from_raw_u32(9999).unwrap();
    let bolt_b = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::new(100.0, 0.0))))
        .id();

    let initial_pos = Vec2::new(999.0, 999.0);
    let beam = app
        .world_mut()
        .spawn((
            TetherBeamComponent {
                bolt_a: fake_bolt_a,
                bolt_b,
                damage_mult: 1.5,
                effective_damage_multiplier: 1.0,
                base_damage: DEFAULT_BOLT_BASE_DAMAGE,
            },
            Spatial::builder().at_position(initial_pos).build(),
            Scale2D { x: 1.0, y: 1.0 },
        ))
        .id();

    app.update();

    // Position should remain unchanged (system skipped this beam)
    let pos = app.world().get::<Position2D>(beam).unwrap();
    assert!(
        (pos.0.x - initial_pos.x).abs() < f32::EPSILON,
        "beam Position2D should remain unchanged when bolt_a is missing, got x={}",
        pos.0.x
    );
}

// ── Behavior 42: sync_tether_beam_visual handles multiple beams independently ──

#[test]
fn sync_tether_beam_visual_handles_multiple_beams_independently() {
    let mut app = visual_sync_test_app();

    // Beam 1: horizontal, (0,0) to (100,0)
    let (_, _, beam_1) =
        spawn_beam_with_bolts(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0));

    // Beam 2: vertical, (50,50) to (50,150)
    let (_, _, beam_2) =
        spawn_beam_with_bolts(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 150.0));

    app.update();

    // Beam 1 assertions
    let pos_1 = app.world().get::<Position2D>(beam_1).unwrap();
    assert!(
        (pos_1.0.x - 50.0).abs() < f32::EPSILON,
        "beam_1 midpoint x should be 50.0, got {}",
        pos_1.0.x
    );
    assert!(
        (pos_1.0.y - 0.0).abs() < f32::EPSILON,
        "beam_1 midpoint y should be 0.0, got {}",
        pos_1.0.y
    );

    let scale_1 = app.world().get::<Scale2D>(beam_1).unwrap();
    assert!(
        (scale_1.x - 100.0).abs() < f32::EPSILON,
        "beam_1 Scale2D.x should be 100.0, got {}",
        scale_1.x
    );
    assert!(
        (scale_1.y - 4.0).abs() < f32::EPSILON,
        "beam_1 Scale2D.y should be 4.0, got {}",
        scale_1.y
    );

    let rot_1 = app.world().get::<Rotation2D>(beam_1).unwrap();
    assert!(
        rot_1.0.as_radians().abs() < 0.01,
        "beam_1 rotation should be ~0 (horizontal), got {}",
        rot_1.0.as_radians()
    );

    // Beam 2 assertions
    let pos_2 = app.world().get::<Position2D>(beam_2).unwrap();
    assert!(
        (pos_2.0.x - 50.0).abs() < f32::EPSILON,
        "beam_2 midpoint x should be 50.0, got {}",
        pos_2.0.x
    );
    assert!(
        (pos_2.0.y - 100.0).abs() < f32::EPSILON,
        "beam_2 midpoint y should be 100.0, got {}",
        pos_2.0.y
    );

    let scale_2 = app.world().get::<Scale2D>(beam_2).unwrap();
    assert!(
        (scale_2.x - 100.0).abs() < f32::EPSILON,
        "beam_2 Scale2D.x should be 100.0, got {}",
        scale_2.x
    );

    let rot_2 = app.world().get::<Rotation2D>(beam_2).unwrap();
    assert!(
        (rot_2.0.as_radians() - std::f32::consts::FRAC_PI_2).abs() < 0.01,
        "beam_2 rotation should be ~PI/2 (vertical), got {}",
        rot_2.0.as_radians()
    );
}
