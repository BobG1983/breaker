//! Tests for `SecondWind` wall despawn-on-contact behavior.

use bevy::prelude::*;

use super::{super::system::*, helpers::*};
use crate::bolt::messages::BoltImpactWall;

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
    // Behavior 17: Only SecondWind walls despawned -- other walls unaffected.
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
