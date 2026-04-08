//! Tests for `RequiredToClear` behavior, orbit radius scaling, initial
//! `Position2D`, `PositionPropagation`, `ChildOf` hierarchy, and
//! zero-orbit edge case.

use bevy::prelude::*;
use rantzsoft_spatial2d::{components::Position2D, propagation::PositionPropagation};

use super::{
    super::{
        super::system::{compute_grid_scale, spawn_cells_from_layout},
        helpers::*,
    },
    helpers::*,
};
use crate::{
    cells::{
        CellTypeDefinition, components::*, definition::ShieldBehavior, resources::CellTypeRegistry,
    },
    state::run::node::{
        ActiveNodeLayout, NodeLayout, definition::NodePool, messages::CellsSpawned,
    },
};

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
            let mut grid = vec![vec!["N".to_owned(); 40]; 20];
            grid[0][0] = "H".to_owned(); // shield in corner
            grid
        },
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
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
        "H".to_owned(),
        CellTypeDefinition {
            id: "shield_zero".to_owned(),
            alias: "H".to_owned(),
            hp: 20.0,
            color_rgb: [0.8, 0.8, 1.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,
            shield: Some(ShieldBehavior {
                count: 0,
                radius: 60.0,
                speed: std::f32::consts::FRAC_PI_2,
                hp: 10.0,
                color_rgb: [0.5, 0.8, 1.0],
            }),
            effects: None,
        },
    );

    let layout = NodeLayout {
        name: "zero_orbits".to_owned(),
        timer_secs: 60.0,
        cols: 1,
        rows: 1,
        grid_top_offset: 50.0,
        grid: vec![vec!["H".to_owned()]],
        pool: NodePool::default(),
        entity_scale: 1.0,
        locks: None,
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
