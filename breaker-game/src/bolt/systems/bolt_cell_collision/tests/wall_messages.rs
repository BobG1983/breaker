use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::helpers::*;
use crate::bolt::components::Bolt;

#[test]
fn wall_hit_emits_bolt_hit_wall_with_correct_bolt_entity() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();

    spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

    let start_x = 200.0 - 50.0 - bc.radius - 5.0;
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x, 0.0)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "wall hit should emit exactly one BoltImpactWall message"
    );
    assert_eq!(
        msgs.0[0].bolt, bolt_entity,
        "BoltImpactWall.bolt should match the bolt entity that hit the wall"
    );
}

#[test]
fn cell_hit_does_not_emit_bolt_hit_wall() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();
    let cc = crate::cells::resources::CellConfig::default();

    let cell_y = 100.0;
    spawn_cell(&mut app, 0.0, cell_y);

    let start_y = cell_y - cc.height / 2.0 - bc.radius - 2.0;
    app.world_mut().spawn((
        Bolt,
        bolt_param_bundle(),
        Velocity2D(Vec2::new(0.0, 400.0)),
        Position2D(Vec2::new(0.0, start_y)),
    ));

    tick(&mut app);

    // BoltImpactCell should be sent (existing behavior)
    let hit_msgs = app.world().resource::<FullHitMessages>();
    assert_eq!(
        hit_msgs.0.len(),
        1,
        "BoltImpactCell should be sent for cell hit"
    );

    // BoltImpactWall should NOT be sent
    let wall_msgs = app.world().resource::<WallHitMessages>();
    assert!(
        wall_msgs.0.is_empty(),
        "cell hit should NOT emit BoltImpactWall, got {} messages",
        wall_msgs.0.len()
    );
}

#[test]
fn bolt_hit_wall_identifies_correct_bolt_among_two() {
    let mut app = test_app_with_damage_and_wall_messages();
    let bc = crate::bolt::resources::BoltConfig::default();

    spawn_wall(&mut app, 200.0, 0.0, 50.0, 300.0);

    // Bolt A: moving right toward wall
    let start_x_a = 200.0 - 50.0 - bc.radius - 5.0;
    let bolt_a = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(400.0, 0.1)),
            Position2D(Vec2::new(start_x_a, 0.0)),
        ))
        .id();

    // Bolt B: moving upward, far from wall -- will not hit it
    let _bolt_b = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(0.0, 400.0)),
            Position2D(Vec2::new(-100.0, 0.0)),
        ))
        .id();

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "only bolt A should hit the wall, got {} BoltImpactWall messages",
        msgs.0.len()
    );
    assert_eq!(
        msgs.0[0].bolt, bolt_a,
        "BoltImpactWall.bolt should be bolt A (the one that hit the wall)"
    );
}
