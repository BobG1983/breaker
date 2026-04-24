use std::time::Duration;

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::{
    cells::components::{Cell, CellHeight, CellWidth},
    hazard::{definition::HazardKind, resources::ActiveHazards},
    prelude::*,
};

// Behavior 14 — debris carries full collision suite.
#[test]
fn debris_has_collision_components() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    app.world_mut().insert_resource(FractureConfig {
        base_splits:      2,
        per_level_splits: 1,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Fracture);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&Cell, &CellWidth, &CellHeight, &Aabb2D, &CollisionLayers)>();
    let count = query.iter(app.world()).count();
    assert_eq!(count, 2, "debris should have full collision suite");
}

#[test]
fn debris_collision_suite_includes_hp_and_killed_by() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(
        &Cell,
        &CellWidth,
        &CellHeight,
        &Aabb2D,
        &CollisionLayers,
        &Hp,
        &KilledBy,
    )>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

// Behavior 14b — debris CellWidth/CellHeight pinned to concrete values.
#[test]
fn debris_cell_width_and_height_are_seventy_and_twenty_four() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app
        .world_mut()
        .query::<(&FractureDebris, &CellWidth, &CellHeight)>();
    let dims: Vec<(f32, f32)> = query
        .iter(app.world())
        .map(|(_, cw, ch)| (cw.value, ch.value))
        .collect();
    assert_eq!(dims.len(), 2);
    for (w, h) in &dims {
        assert!((w - 70.0).abs() < f32::EPSILON, "width = {w}");
        assert!((h - 24.0).abs() < f32::EPSILON, "height = {h}");
    }
}

#[test]
fn debris_scale_matches_pristine_cell_footprint() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Scale2D)>();
    let scales: Vec<Scale2D> = query.iter(app.world()).map(|(_, s)| *s).collect();
    assert_eq!(scales.len(), 2);
    for scale in &scales {
        assert!(
            (scale.x - 70.0).abs() < f32::EPSILON,
            "scale.x = {}",
            scale.x
        );
        assert!(
            (scale.y - 24.0).abs() < f32::EPSILON,
            "scale.y = {}",
            scale.y
        );
    }
}

// Behavior 15 — FractureDebris marker is present on debris and only on debris.
#[test]
fn debris_carries_fracture_debris_marker_and_cell() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&Cell, &FractureDebris)>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

#[test]
fn fracture_debris_marker_does_not_attach_to_plain_cells() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 1);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    // Initial debris count is 2.
    {
        let mut query = app.world_mut().query::<&FractureDebris>();
        assert_eq!(query.iter(app.world()).count(), 2);
    }

    // Spawn a plain Cell with no FractureDebris marker.
    app.world_mut()
        .spawn((Cell, Position2D(Vec2::new(500.0, 500.0))));
    app.update();

    // Still only 2 FractureDebris entities.
    let mut query = app.world_mut().query::<&FractureDebris>();
    assert_eq!(query.iter(app.world()).count(), 2);
}

// Behavior 16 — debris HP = 1 regardless of stack count.
#[test]
fn debris_hp_is_one_at_stack_three() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 3);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Hp)>();
    let hps: Vec<f32> = query.iter(app.world()).map(|(_, hp)| hp.current).collect();
    assert_eq!(hps.len(), 4);
    for hp in &hps {
        assert!((hp - 1.0).abs() < f32::EPSILON, "hp = {hp}");
    }
}

#[test]
fn debris_hp_is_one_at_stack_ten_cap() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, fracture_on_death);
    install_fracture_config(&mut app, canonical_config());
    add_fracture_stacks(&mut app, 10);

    write_cell_destroyed(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs_f32(0.016));

    let mut query = app.world_mut().query::<(&FractureDebris, &Hp)>();
    let hps: Vec<f32> = query.iter(app.world()).map(|(_, hp)| hp.current).collect();
    assert_eq!(hps.len(), 4);
    for hp in &hps {
        assert!((hp - 1.0).abs() < f32::EPSILON, "hp = {hp}");
    }
}
