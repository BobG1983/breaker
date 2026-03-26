use bevy::prelude::*;
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::{
    components::{Position2D, Scale2D},
    propagation::PositionPropagation,
};

use super::{
    super::system::{compute_grid_scale, spawn_cells_from_layout},
    helpers::*,
};
use crate::{
    cells::{
        CellTypeDefinition,
        components::*,
        definition::{CellBehavior, ShieldBehavior},
        resources::{CellConfig, CellTypeRegistry},
    },
    run::node::{ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned},
    shared::{BOLT_LAYER, CELL_LAYER, PlayfieldConfig},
};

/// Creates a registry containing a shield cell type ('H') plus a normal cell ('N').
fn shield_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        'H',
        CellTypeDefinition {
            id: "shield".to_owned(),
            alias: 'H',
            hp: 20.0,
            color_rgb: [0.8, 0.8, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior {
                locked: false,
                regen_rate: None,
                shield: Some(ShieldBehavior {
                    count: 3,
                    radius: 60.0,
                    speed: std::f32::consts::FRAC_PI_2,
                    hp: 10.0,
                    color_rgb: [0.5, 0.8, 1.0],
                }),
            },
        },
    );
    registry.insert(
        'N',
        CellTypeDefinition {
            id: "normal".to_owned(),
            alias: 'N',
            hp: 1.0,
            color_rgb: [1.0, 0.5, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
        },
    );
    registry
}

fn shield_layout() -> NodeLayout {
    NodeLayout {
        name: "shield_test".to_owned(),
        timer_secs: 60.0,
        cols: 2,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!['H', 'N']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    }
}

fn shield_test_app(layout: NodeLayout, registry: CellTypeRegistry) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .init_resource::<CellConfig>()
        .init_resource::<PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(registry)
        .add_systems(Startup, spawn_cells_from_layout);
    app
}

// -- Behavior 1: Shield cell has ShieldParent + OrbitConfig + Locked + LockAdjacents --

#[test]
fn shield_cell_has_shield_parent_marker() {
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_count = app
        .world_mut()
        .query::<(&Cell, &ShieldParent)>()
        .iter(app.world())
        .count();
    assert_eq!(
        shield_count, 1,
        "shield cell ('H') should have ShieldParent marker"
    );
}

#[test]
fn shield_cell_has_locked_and_lock_adjacents() {
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_locked_count = app
        .world_mut()
        .query::<(&Cell, &ShieldParent, &Locked, &LockAdjacents)>()
        .iter(app.world())
        .count();
    assert_eq!(
        shield_locked_count, 1,
        "shield cell should have Locked + LockAdjacents"
    );
}

#[test]
fn shield_cell_lock_adjacents_contains_orbit_entity_ids() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: LockAdjacents contains exactly 3 entity IDs (the orbit children)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_adjacents: Vec<&LockAdjacents> = app
        .world_mut()
        .query_filtered::<&LockAdjacents, With<ShieldParent>>()
        .iter(app.world())
        .collect();
    assert_eq!(shield_adjacents.len(), 1);
    assert_eq!(
        shield_adjacents[0].0.len(),
        3,
        "shield LockAdjacents should contain 3 orbit entity IDs, got {}",
        shield_adjacents[0].0.len()
    );

    // Verify each entity in LockAdjacents is an actual OrbitCell
    for &orbit_entity in &shield_adjacents[0].0 {
        assert!(
            app.world().get::<OrbitCell>(orbit_entity).is_some(),
            "LockAdjacents entity {orbit_entity:?} should have OrbitCell component"
        );
    }
}

// -- Behavior 2: Orbit children spawned with correct components --

#[test]
fn orbit_cells_spawned_with_correct_count() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: 3 OrbitCell entities exist
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let orbit_count = app
        .world_mut()
        .query::<(&Cell, &OrbitCell)>()
        .iter(app.world())
        .count();
    assert_eq!(
        orbit_count, 3,
        "should spawn 3 orbit cells, got {orbit_count}"
    );
}

#[test]
fn orbit_cells_have_cell_health_from_shield_behavior() {
    // Given: shield with orbit_hp=10.0
    // When: spawn runs
    // Then: each OrbitCell has CellHealth { current: 10.0, max: 10.0 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for health in app
        .world_mut()
        .query_filtered::<&CellHealth, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (health.current - 10.0).abs() < f32::EPSILON,
            "orbit cell current HP should be 10.0, got {}",
            health.current
        );
        assert!(
            (health.max - 10.0).abs() < f32::EPSILON,
            "orbit cell max HP should be 10.0, got {}",
            health.max
        );
    }
}

#[test]
fn orbit_cells_have_square_20x20_scale() {
    // Given: orbit dimensions 20.0 x 20.0
    // When: spawn runs
    // Then: each OrbitCell has Scale2D { x: 20.0, y: 20.0 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for scale in app
        .world_mut()
        .query_filtered::<&Scale2D, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (scale.x - 20.0).abs() < f32::EPSILON,
            "orbit cell Scale2D.x should be 20.0, got {}",
            scale.x
        );
        assert!(
            (scale.y - 20.0).abs() < f32::EPSILON,
            "orbit cell Scale2D.y should be 20.0, got {}",
            scale.y
        );
    }
}

#[test]
fn orbit_cells_have_aabb2d_matching_square_size() {
    // Given: orbit dimensions 20.0 x 20.0
    // When: spawn runs
    // Then: each OrbitCell has Aabb2D with half_extents (10.0, 10.0)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for aabb in app
        .world_mut()
        .query_filtered::<&Aabb2D, With<OrbitCell>>()
        .iter(app.world())
    {
        assert_eq!(
            aabb.center,
            Vec2::ZERO,
            "orbit cell Aabb2D center should be ZERO"
        );
        assert!(
            (aabb.half_extents.x - 10.0).abs() < f32::EPSILON,
            "orbit cell Aabb2D half_extents.x should be 10.0, got {}",
            aabb.half_extents.x
        );
        assert!(
            (aabb.half_extents.y - 10.0).abs() < f32::EPSILON,
            "orbit cell Aabb2D half_extents.y should be 10.0, got {}",
            aabb.half_extents.y
        );
    }
}

#[test]
fn orbit_cells_have_collision_layers() {
    // Given: orbit cells are collidable
    // When: spawn runs
    // Then: each OrbitCell has CollisionLayers { membership: CELL_LAYER, mask: BOLT_LAYER }

    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for layers in app
        .world_mut()
        .query_filtered::<&CollisionLayers, With<OrbitCell>>()
        .iter(app.world())
    {
        assert_eq!(
            layers.membership, CELL_LAYER,
            "orbit cell membership should be CELL_LAYER"
        );
        assert_eq!(
            layers.mask, BOLT_LAYER,
            "orbit cell mask should be BOLT_LAYER"
        );
    }
}

#[test]
fn orbit_cells_have_orbit_angle_evenly_spaced() {
    // Given: shield with orbit_count=3
    // When: spawn runs
    // Then: OrbitAngle values are 0, 2*PI/3, 4*PI/3 (evenly spaced)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let mut angles: Vec<f32> = app
        .world_mut()
        .query_filtered::<&OrbitAngle, With<OrbitCell>>()
        .iter(app.world())
        .map(|a| a.0)
        .collect();
    angles.sort_by(f32::total_cmp);

    assert_eq!(angles.len(), 3, "should have 3 orbit angles");
    let expected = [
        0.0,
        2.0 * std::f32::consts::PI / 3.0,
        4.0 * std::f32::consts::PI / 3.0,
    ];
    for (i, (actual, exp)) in angles.iter().zip(expected.iter()).enumerate() {
        assert!(
            (actual - exp).abs() < 1e-5,
            "orbit {i} angle should be {exp:.4}, got {actual:.4}"
        );
    }
}

#[test]
fn orbit_cells_have_orbit_config_from_shield_behavior() {
    // Given: shield with orbit_radius=60.0, orbit_speed=PI/2
    // When: spawn runs
    // Then: each OrbitCell has OrbitConfig { radius: 60.0, speed: PI/2 }
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for config in app
        .world_mut()
        .query_filtered::<&OrbitConfig, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (config.radius - 60.0).abs() < f32::EPSILON,
            "orbit config radius should be 60.0, got {}",
            config.radius
        );
        assert!(
            (config.speed - std::f32::consts::FRAC_PI_2).abs() < f32::EPSILON,
            "orbit config speed should be PI/2, got {}",
            config.speed
        );
    }
}

// -- Behavior 3: Orbit cells NOT RequiredToClear --

#[test]
fn orbit_cells_not_required_to_clear() {
    // Given: shield with orbits
    // When: spawn runs
    // Then: NO OrbitCell has RequiredToClear
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let orbit_required = app
        .world_mut()
        .query::<(&OrbitCell, &RequiredToClear)>()
        .iter(app.world())
        .count();
    assert_eq!(
        orbit_required, 0,
        "orbit cells should NOT have RequiredToClear"
    );
}

#[test]
fn shield_parent_is_required_to_clear() {
    // Given: shield definition has required_to_clear=true
    // When: spawn runs
    // Then: the shield parent cell HAS RequiredToClear
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_required = app
        .world_mut()
        .query::<(&ShieldParent, &RequiredToClear)>()
        .iter(app.world())
        .count();
    assert_eq!(
        shield_required, 1,
        "shield parent cell should have RequiredToClear"
    );
}

// -- Behavior 4: Orbit radius scales by grid scale with min clamp --

#[test]
fn orbit_radius_scaled_by_grid_scale_factor() {
    // Given: a grid so large it requires grid scaling (e.g., 40x20)
    //        shield orbit_radius=60.0
    // When: spawn runs
    // Then: orbit OrbitConfig.radius = 60.0 * grid_scale (< 60.0)
    let layout = NodeLayout {
        name: "shield_scale_test".to_owned(),
        timer_secs: 60.0,
        cols: 40,
        rows: 20,
        grid_top_offset: 90.0,
        grid: {
            let mut grid = vec![vec!['N'; 40]; 20];
            grid[0][0] = 'H'; // shield in corner
            grid
        },
        pool: NodePool::default(),
        entity_scale: 1.0,
    };

    let cell_config = ron_like_cell_config();
    let playfield = ron_like_playfield_config();
    let dims = compute_grid_scale(&cell_config, &playfield, 40, 20, 90.0);
    assert!(
        dims.scale < 1.0,
        "40x20 grid should need scaling, got {}",
        dims.scale
    );
    let expected_radius = (60.0 * dims.scale).max(10.0); // min clamp

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CellsSpawned>()
        .insert_resource(cell_config)
        .insert_resource(playfield)
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>()
        .insert_resource(ActiveNodeLayout(layout))
        .insert_resource(shield_registry())
        .add_systems(Startup, spawn_cells_from_layout);
    app.update();

    for config in app
        .world_mut()
        .query_filtered::<&OrbitConfig, With<OrbitCell>>()
        .iter(app.world())
    {
        assert!(
            (config.radius - expected_radius).abs() < 0.5,
            "orbit radius should be ~{expected_radius:.2} (60.0 * {:.3}), got {:.2}",
            dims.scale,
            config.radius
        );
    }
}

// -- Behavior 5: Orbit initial Position2D = shield_pos + radius * (cos, sin) --

#[test]
fn orbit_initial_position_matches_shield_pos_plus_radius_offset() {
    // Given: shield at some grid position, orbit_count=3, orbit_radius=60.0
    //        angles: 0, 2PI/3, 4PI/3
    // When: spawn runs (no sync system registered, just spawn)
    // Then: each orbit has Position2D = shield_pos + 60.0 * (cos(angle), sin(angle))
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    // Find shield Position2D
    let shield_pos: Vec2 = app
        .world_mut()
        .query_filtered::<&Position2D, With<ShieldParent>>()
        .iter(app.world())
        .next()
        .expect("shield should exist")
        .0;

    let expected_angles = [
        0.0,
        2.0 * std::f32::consts::PI / 3.0,
        4.0 * std::f32::consts::PI / 3.0,
    ];

    let mut orbit_data: Vec<(f32, Vec2)> = app
        .world_mut()
        .query_filtered::<(&OrbitAngle, &Position2D), With<OrbitCell>>()
        .iter(app.world())
        .map(|(angle, pos)| (angle.0, pos.0))
        .collect();
    orbit_data.sort_by(|a, b| a.0.total_cmp(&b.0));

    assert_eq!(orbit_data.len(), 3, "should have 3 orbit cells");

    for (i, (angle, pos)) in orbit_data.iter().enumerate() {
        let expected_x = 60.0f32.mul_add(angle.cos(), shield_pos.x);
        let expected_y = 60.0f32.mul_add(angle.sin(), shield_pos.y);
        assert!(
            (pos.x - expected_x).abs() < 0.1,
            "orbit {i} x at angle {:.3}: expected {expected_x:.2}, got {:.2}",
            expected_angles[i],
            pos.x
        );
        assert!(
            (pos.y - expected_y).abs() < 0.1,
            "orbit {i} y at angle {:.3}: expected {expected_y:.2}, got {:.2}",
            expected_angles[i],
            pos.y
        );
    }
}

// -- Behavior 5 edge case: orbit cells use PositionPropagation::Absolute --

#[test]
fn orbit_cells_have_absolute_position_propagation() {
    // Orbit cells must use Absolute position propagation so the quadtree
    // sees correct world-space coordinates.
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    for prop in app
        .world_mut()
        .query_filtered::<&PositionPropagation, With<OrbitCell>>()
        .iter(app.world())
    {
        assert_eq!(
            *prop,
            PositionPropagation::Absolute,
            "orbit cell should have PositionPropagation::Absolute"
        );
    }
}

// -- Behavior 8: Shield destroyed -> orbit children auto-despawned via ChildOf --

#[test]
fn orbit_cells_are_children_of_shield_via_child_of() {
    // Given: shield with 3 orbits
    // When: spawn runs
    // Then: each OrbitCell entity is a child of the shield entity (ChildOf relationship)
    let mut app = shield_test_app(shield_layout(), shield_registry());
    app.update();

    let shield_entity = app
        .world_mut()
        .query_filtered::<Entity, With<ShieldParent>>()
        .iter(app.world())
        .next()
        .expect("shield should exist");

    // Collect orbit entities first to avoid borrow conflict
    let orbit_entities: Vec<Entity> = app
        .world_mut()
        .query_filtered::<Entity, With<OrbitCell>>()
        .iter(app.world())
        .collect();

    assert_eq!(orbit_entities.len(), 3, "should have 3 orbit cells");

    // Check via Children component on shield
    let children = app
        .world()
        .get::<Children>(shield_entity)
        .expect("shield should have Children component (from ChildOf on orbits)");

    for orbit in &orbit_entities {
        assert!(
            children.contains(orbit),
            "orbit entity {orbit:?} should be a child of shield entity {shield_entity:?}"
        );
    }
}

// -- Edge case: orbit_count=0 -> shield has empty LockAdjacents --

#[test]
fn shield_with_zero_orbit_count_has_empty_lock_adjacents() {
    // Given: shield with orbit_count=0
    // When: spawn runs
    // Then: shield has Locked + LockAdjacents(empty) -> will immediately unlock
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        'H',
        CellTypeDefinition {
            id: "shield_zero".to_owned(),
            alias: 'H',
            hp: 20.0,
            color_rgb: [0.8, 0.8, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior {
                locked: false,
                regen_rate: None,
                shield: Some(ShieldBehavior {
                    count: 0,
                    radius: 60.0,
                    speed: std::f32::consts::FRAC_PI_2,
                    hp: 10.0,
                    color_rgb: [0.5, 0.8, 1.0],
                }),
            },
        },
    );

    let layout = NodeLayout {
        name: "zero_orbits".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!['H']],
        pool: NodePool::default(),
        entity_scale: 1.0,
    };

    let mut app = shield_test_app(layout, registry);
    app.update();

    // No orbit cells spawned
    let orbit_count = app
        .world_mut()
        .query::<&OrbitCell>()
        .iter(app.world())
        .count();
    assert_eq!(
        orbit_count, 0,
        "shield with orbit_count=0 should spawn no orbit cells"
    );

    // Shield has LockAdjacents with empty vec
    let adjacents: Vec<&LockAdjacents> = app
        .world_mut()
        .query_filtered::<&LockAdjacents, With<ShieldParent>>()
        .iter(app.world())
        .collect();
    assert_eq!(adjacents.len(), 1);
    assert!(
        adjacents[0].0.is_empty(),
        "shield with orbit_count=0 should have empty LockAdjacents, got {} entries",
        adjacents[0].0.len()
    );
}
