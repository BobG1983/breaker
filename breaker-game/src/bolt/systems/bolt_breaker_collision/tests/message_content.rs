use bevy::prelude::*;

use super::helpers::*;

#[test]
fn bolt_entity_in_hit_message() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(CapturedHitBolts::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_bolts.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    spawn_breaker_at(&mut app, 0.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 0.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let captured = app.world().resource::<CapturedHitBolts>();
    assert_eq!(
        captured.0.len(),
        1,
        "should capture exactly one BoltImpactBreaker message"
    );
    assert_eq!(
        captured.0[0], bolt_entity,
        "BoltImpactBreaker.bolt should carry the actual bolt entity that hit the breaker"
    );
}

#[test]
fn breaker_entity_in_hit_message() {
    let mut app = test_app();
    let hh = default_breaker_height();
    let y_pos = -250.0;
    app.insert_resource(CapturedHitPairs::default())
        .add_systems(
            FixedUpdate,
            collect_breaker_hit_pairs.after(
                crate::bolt::systems::bolt_breaker_collision::system::bolt_breaker_collision,
            ),
        );

    let breaker_entity = spawn_breaker_at(&mut app, 200.0, y_pos);

    let start_y = y_pos + hh.half_height() + default_bolt_radius().0 + 3.0;
    let bolt_entity = spawn_bolt(&mut app, 200.0, start_y, 0.0, -400.0);
    tick(&mut app);

    let captured = app.world().resource::<CapturedHitPairs>();
    assert_eq!(
        captured.0.len(),
        1,
        "should capture exactly one BoltImpactBreaker message"
    );
    assert_eq!(
        captured.0[0].0, bolt_entity,
        "BoltImpactBreaker.bolt should carry the actual bolt entity"
    );
    assert_eq!(
        captured.0[0].1, breaker_entity,
        "BoltImpactBreaker.breaker should carry the actual breaker entity, not Entity::PLACEHOLDER"
    );
}
