use bevy::prelude::*;
use breaker::effect::effects::gravity_well::GravityWellMarker;
use rantzsoft_physics2d::aabb::Aabb2D;

use super::mutations::*;
use crate::invariants::ScenarioTagBolt;

// -------------------------------------------------------------------------
// apply_inject_mismatched_bolt_aabb — behavior 24-25
// -------------------------------------------------------------------------

#[test]
fn apply_inject_mismatched_bolt_aabb_corrupts_bolt_aabb() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let bolt_entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
        ))
        .id();

    // Run a system that calls the helper
    app.add_systems(
        Update,
        |mut bolts: Query<&mut Aabb2D, With<ScenarioTagBolt>>| {
            apply_inject_mismatched_bolt_aabb(&mut bolts);
        },
    );
    app.update();

    let aabb = app.world().get::<Aabb2D>(bolt_entity).unwrap();
    assert_eq!(
        aabb.half_extents,
        Vec2::splat(999.0),
        "apply_inject_mismatched_bolt_aabb should set half_extents to Vec2::splat(999.0)"
    );
}

#[test]
fn apply_inject_mismatched_bolt_aabb_noop_when_no_bolts() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    // No ScenarioTagBolt entities — should not panic
    app.add_systems(
        Update,
        |mut bolts: Query<&mut Aabb2D, With<ScenarioTagBolt>>| {
            apply_inject_mismatched_bolt_aabb(&mut bolts);
        },
    );
    app.update();
    // If we reach here without panic, the test passes the "no panic" assertion.
    // But per RED phase, this test should fail because the stub is a no-op.
    // Since an empty query is a no-op either way, this test passes even with the stub.
    // We need the first test (behavior 24) to be the failing one.
}

// -------------------------------------------------------------------------
// apply_spawn_extra_gravity_wells — behavior 26
// -------------------------------------------------------------------------

#[test]
fn apply_spawn_extra_gravity_wells_spawns_n_entities() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, |mut commands: Commands| {
        apply_spawn_extra_gravity_wells(3, &mut commands);
    });
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<GravityWellMarker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 3,
        "apply_spawn_extra_gravity_wells(3) should spawn 3 GravityWellMarker entities, got {count}"
    );
}

#[test]
fn apply_spawn_extra_gravity_wells_zero_spawns_nothing() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    app.add_systems(Update, |mut commands: Commands| {
        apply_spawn_extra_gravity_wells(0, &mut commands);
    });
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<GravityWellMarker>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "apply_spawn_extra_gravity_wells(0) should spawn zero entities, got {count}"
    );
}
