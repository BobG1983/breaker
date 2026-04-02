//! Tests for `tick_pulse_emitter` spawn and interval behaviors.

use super::super::helpers::*;

// -- Behavior 11: tick_pulse_emitter spawns PulseRing when timer fires --

#[test]
fn tick_pulse_emitter_spawns_ring_when_timer_reaches_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(80.0, 120.0)),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 0.5,
                timer: 0.49,
            },
        ))
        .id();

    // Advance time by injecting a delta that will push timer over interval.
    // MinimalPlugins provides Time, one update tick advances by a small dt.
    // We pre-set timer to 0.49 so even a tiny dt (e.g., 0.016) pushes it past 0.5.
    app.update();

    // Check that a PulseRing entity was spawned
    let mut ring_query = app.world_mut().query::<(
        &PulseRing,
        &PulseSource,
        &PulseRadius,
        &PulseMaxRadius,
        &PulseSpeed,
        &PulseDamaged,
        &Position2D,
    )>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned, got {}",
        rings.len()
    );

    let (_ring, _source, radius, max_radius, speed, damaged, pos) = rings[0];
    assert!(
        (radius.0 - 0.0).abs() < f32::EPSILON,
        "new ring radius should be 0.0"
    );
    assert!(
        (max_radius.0 - 32.0).abs() < f32::EPSILON,
        "max radius should match effective_max_radius (32.0)"
    );
    assert!(
        (speed.0 - 50.0).abs() < f32::EPSILON,
        "ring speed should be 50.0"
    );
    assert!(damaged.0.is_empty(), "ring PulseDamaged should be empty");
    assert!(
        (pos.0.x - 80.0).abs() < f32::EPSILON,
        "ring should spawn at bolt x position"
    );
    assert!(
        (pos.0.y - 120.0).abs() < f32::EPSILON,
        "ring should spawn at bolt y position"
    );

    // Emitter timer should have been reset (timer - interval preserves fractional part)
    let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
    assert!(
        emitter.timer < emitter.interval,
        "emitter timer should be reset after emission, got {}",
        emitter.timer
    );
}

// -- Behavior 12: tick_pulse_emitter does NOT spawn before interval --

#[test]
fn tick_pulse_emitter_does_not_spawn_before_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 100.0, // Very large interval -- will never fire
                timer: 0.0,
            },
        ))
        .id();

    app.update();

    let mut ring_query = app.world_mut().query::<&PulseRing>();
    let count = ring_query.iter(app.world()).count();
    assert_eq!(
        count, 0,
        "no PulseRing should spawn before interval elapses"
    );

    // Timer should have advanced
    let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
    assert!(
        emitter.timer > 0.0,
        "emitter timer should advance, got {}",
        emitter.timer
    );
}

// -- Behavior 13: tick_pulse_emitter reads bolt's current position --

#[test]
fn tick_pulse_emitter_reads_current_bolt_position() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Bolt starts at (200, 300) -- this is its "current" position
    let _bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(200.0, 300.0)),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 0.5,
                timer: 0.49, // About to fire
            },
        ))
        .id();

    app.update();

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Position2D)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(rings.len(), 1, "expected one ring spawned");

    let (_ring, pos) = rings[0];
    assert!(
        (pos.0.x - 200.0).abs() < f32::EPSILON,
        "ring should spawn at bolt's current x (200.0), got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 300.0).abs() < f32::EPSILON,
        "ring should spawn at bolt's current y (300.0), got {}",
        pos.0.y
    );
}

// -- Behavior 4: tick_pulse_emitter respects custom interval --

#[test]
fn tick_pulse_emitter_respects_custom_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(80.0, 120.0)),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 0.25,
                timer: 0.24,
            },
        ))
        .id();

    // One update tick should push timer past 0.25 and trigger emission
    app.update();

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Position2D)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned with custom interval 0.25, got {}",
        rings.len()
    );

    let (_ring, pos) = rings[0];
    assert!(
        (pos.0.x - 80.0).abs() < f32::EPSILON,
        "ring should spawn at bolt x position (80.0), got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 120.0).abs() < f32::EPSILON,
        "ring should spawn at bolt y position (120.0), got {}",
        pos.0.y
    );

    // Emitter timer should have been reset
    let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
    assert!(
        emitter.timer < 0.25,
        "emitter timer should be reset after emission with custom interval, got {}",
        emitter.timer
    );
}

#[test]
fn tick_pulse_emitter_large_interval_does_not_emit() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(0.0, 0.0)),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 0.0,
                stacks: 1,
                speed: 50.0,
                interval: 100.0,
                timer: 0.0,
            },
        ))
        .id();

    app.update();

    let mut ring_query = app.world_mut().query::<&PulseRing>();
    let count = ring_query.iter(app.world()).count();
    assert_eq!(
        count, 0,
        "PulseEmitter with interval 100.0 and timer 0.0 should NOT emit after one tick"
    );

    let emitter = app.world().get::<PulseEmitter>(bolt).unwrap();
    assert!(
        emitter.timer > 0.0,
        "emitter timer should advance even when not emitting, got {}",
        emitter.timer
    );
}
