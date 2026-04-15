//! Tests for multiple bolts reflecting off the breaker independently.

use bevy::prelude::*;

use crate::{bolt::systems::bolt_breaker_collision::tests::helpers::*, prelude::*};

#[test]
fn multiple_bolts_each_reflect_off_breaker() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(HitBreakers::default()).add_systems(
        FixedUpdate,
        collect_breaker_hits
            .after(crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision),
    );
    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;

    let left_bolt = spawn_bolt(&mut app, -30.0, start_y, 0.0, -400.0);
    let right_bolt = spawn_bolt(&mut app, 30.0, start_y, 0.0, -400.0);

    tick(&mut app);

    let velocities: Vec<(Entity, Vec2)> = app
        .world_mut()
        .query::<(Entity, &Velocity2D)>()
        .iter(app.world())
        .map(|(e, v)| (e, v.0))
        .collect();

    for (entity, vel) in &velocities {
        assert!(
            vel.y > 0.0,
            "bolt {entity:?} should reflect upward, got vy={:.1}",
            vel.y
        );
    }

    let hits = app.world().resource::<HitBreakers>();
    assert_eq!(hits.0, 2, "both bolts should trigger hit messages");

    let left_vel = velocities.iter().find(|(e, _)| *e == left_bolt).unwrap().1;
    let right_vel = velocities.iter().find(|(e, _)| *e == right_bolt).unwrap().1;
    assert!(
        left_vel.x < 0.0,
        "left bolt should angle leftward, got vx={:.1}",
        left_vel.x
    );
    assert!(
        right_vel.x > 0.0,
        "right bolt should angle rightward, got vx={:.1}",
        right_vel.x
    );
}
