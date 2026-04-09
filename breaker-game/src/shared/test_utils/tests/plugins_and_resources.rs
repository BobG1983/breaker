use bevy::prelude::*;

use super::super::*;
use crate::{cells::resources::CellConfig, shared::playfield::PlayfieldConfig};

// ════════════════════════════════════════════════════════════════════
// Section E: with_physics()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 10: with_physics() adds RantzPhysics2dPlugin ──

#[test]
fn with_physics_adds_collision_quadtree() {
    let app = TestAppBuilder::new().with_physics().build();
    assert!(
        app.world()
            .get_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>()
            .is_some(),
        "with_physics() must add CollisionQuadtree resource"
    );
}

#[test]
fn with_physics_works_with_state_hierarchy_and_navigation() {
    let app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .build();
    assert!(
        app.world()
            .get_resource::<rantzsoft_physics2d::resources::CollisionQuadtree>()
            .is_some(),
        "with_physics() should work alongside state hierarchy"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section F: with_playfield()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 11: with_playfield() registers config resources ──

#[test]
fn with_playfield_registers_playfield_config() {
    let app = TestAppBuilder::new().with_playfield().build();
    let config = app.world().get_resource::<PlayfieldConfig>();
    assert!(
        config.is_some(),
        "with_playfield() must register PlayfieldConfig"
    );
    let config = config.unwrap();
    assert!(
        (config.width - 800.0).abs() < f32::EPSILON,
        "PlayfieldConfig default width should be 800.0, got {}",
        config.width
    );
    assert!(
        (config.height - 600.0).abs() < f32::EPSILON,
        "PlayfieldConfig default height should be 600.0, got {}",
        config.height
    );
}

#[test]
fn with_playfield_registers_cell_config() {
    let app = TestAppBuilder::new().with_playfield().build();
    let config = app.world().get_resource::<CellConfig>();
    assert!(
        config.is_some(),
        "with_playfield() must register CellConfig"
    );
    let config = config.unwrap();
    assert!(
        (config.width - 70.0).abs() < f32::EPSILON,
        "CellConfig default width should be 70.0, got {}",
        config.width
    );
    assert!(
        (config.height - 24.0).abs() < f32::EPSILON,
        "CellConfig default height should be 24.0, got {}",
        config.height
    );
}

#[test]
fn with_playfield_registers_mesh_and_color_material_assets() {
    let app = TestAppBuilder::new().with_playfield().build();
    assert!(
        app.world().get_resource::<Assets<Mesh>>().is_some(),
        "with_playfield() must register Assets<Mesh>"
    );
    assert!(
        app.world()
            .get_resource::<Assets<ColorMaterial>>()
            .is_some(),
        "with_playfield() must register Assets<ColorMaterial>"
    );
}

#[test]
fn with_playfield_overwrite_with_insert_resource() {
    let app = TestAppBuilder::new()
        .with_playfield()
        .insert_resource(PlayfieldConfig {
            width: 400.0,
            ..Default::default()
        })
        .build();
    assert!(
        (app.world().resource::<PlayfieldConfig>().width - 400.0).abs() < f32::EPSILON,
        "insert_resource after with_playfield should overwrite PlayfieldConfig"
    );
}

// ════════════════════════════════════════════════════════════════════
// Section G: with_resource() and insert_resource()
// ════════════════════════════════════════════════════════════════════

// ── Behavior 12: with_resource() initializes from Default ──

#[test]
fn with_resource_initializes_default() {
    let app = TestAppBuilder::new()
        .with_resource::<PlayfieldConfig>()
        .build();
    assert!(
        (app.world().resource::<PlayfieldConfig>().width - 800.0).abs() < f32::EPSILON,
        "with_resource::<PlayfieldConfig>() should init with Default (width 800.0)"
    );
}

#[test]
fn with_resource_called_twice_does_not_panic() {
    let app = TestAppBuilder::new()
        .with_resource::<PlayfieldConfig>()
        .with_resource::<PlayfieldConfig>()
        .build();
    assert!(
        app.world().get_resource::<PlayfieldConfig>().is_some(),
        "calling with_resource twice should be idempotent"
    );
}

// ── Behavior 13: insert_resource() inserts concrete value ──

#[test]
fn insert_resource_inserts_concrete_value() {
    let app = TestAppBuilder::new()
        .insert_resource(PlayfieldConfig {
            width: 1920.0,
            height: 1080.0,
            ..Default::default()
        })
        .build();
    let config = app.world().resource::<PlayfieldConfig>();
    assert!(
        (config.width - 1920.0).abs() < f32::EPSILON,
        "insert_resource should set width to 1920.0, got {}",
        config.width
    );
    assert!(
        (config.height - 1080.0).abs() < f32::EPSILON,
        "insert_resource should set height to 1080.0, got {}",
        config.height
    );
}

#[test]
fn insert_resource_overwrites_with_resource() {
    let app = TestAppBuilder::new()
        .with_resource::<PlayfieldConfig>()
        .insert_resource(PlayfieldConfig {
            width: 42.0,
            ..Default::default()
        })
        .build();
    assert!(
        (app.world().resource::<PlayfieldConfig>().width - 42.0).abs() < f32::EPSILON,
        "insert_resource after with_resource should overwrite to 42.0"
    );
}
