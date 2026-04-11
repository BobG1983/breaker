//! Tests for explode flat damage passthrough via `process_explode_requests`.

use bevy::prelude::*;

use super::{super::effect::*, helpers::*};

// -- Flat damage passthrough: process_explode_requests uses ExplodeRequest.damage directly ──

#[test]
fn process_explode_requests_passes_flat_damage_to_damage_cell() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    // Spawn ExplodeRequest with flat damage=40.0
    app.world_mut().spawn((
        ExplodeRequest {
            range: 50.0,
            damage: 40.0,
        },
        rantzsoft_spatial2d::components::Position2D(Vec2::ZERO),
    ));

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1, "expected one DamageCell");
    assert_eq!(collector.0[0].cell, cell);

    assert!(
        (collector.0[0].damage - 40.0).abs() < f32::EPSILON,
        "expected flat damage 40.0, got {}",
        collector.0[0].damage
    );
}

#[test]
fn fire_passes_flat_damage_to_explode_request() {
    let mut app = damage_test_app();

    let cell = spawn_test_cell(&mut app, 30.0, 0.0);

    let source = app
        .world_mut()
        .spawn(rantzsoft_spatial2d::components::Position2D(Vec2::ZERO))
        .id();

    fire(source, 50.0, 25.0, "", app.world_mut());

    tick(&mut app);

    let collector = app.world().resource::<DamageCellCollector>();
    assert_eq!(collector.0.len(), 1);
    assert_eq!(collector.0[0].cell, cell);

    assert!(
        (collector.0[0].damage - 25.0).abs() < f32::EPSILON,
        "expected flat damage 25.0 passthrough, got {}",
        collector.0[0].damage
    );
}
