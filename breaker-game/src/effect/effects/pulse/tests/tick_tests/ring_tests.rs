//! Tests for `tick_pulse_ring` expansion and `despawn_finished_pulse_ring` behaviors.

use super::super::helpers::*;

// -- Behavior 14: tick_pulse_ring expands radius --

#[test]
fn tick_pulse_ring_expands_radius_by_speed_times_dt() {
    let mut app = test_app();
    enter_playing(&mut app);

    let ring = app
        .world_mut()
        .spawn((
            PulseRing,
            PulseRadius(10.0),
            PulseMaxRadius(50.0),
            PulseSpeed(100.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let radius = app.world().get::<PulseRadius>(ring).unwrap();
    // After one tick, radius should have increased (speed * dt > 0)
    assert!(
        radius.0 > 10.0,
        "pulse radius should expand, got {}",
        radius.0
    );
}

#[test]
fn tick_pulse_ring_zero_speed_no_expansion() {
    let mut app = test_app();
    enter_playing(&mut app);

    let ring = app
        .world_mut()
        .spawn((
            PulseRing,
            PulseRadius(10.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let radius = app.world().get::<PulseRadius>(ring).unwrap();
    assert!(
        (radius.0 - 10.0).abs() < f32::EPSILON,
        "pulse radius should not change with zero speed, got {}",
        radius.0
    );
}

// -- Behavior 15: despawn_finished_pulse_ring --

#[test]
fn despawn_finished_pulse_ring_when_radius_equals_max() {
    let mut app = test_app();
    enter_playing(&mut app);

    let ring = app
        .world_mut()
        .spawn((
            PulseRing,
            PulseRadius(50.0),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    assert!(
        app.world().get_entity(ring).is_err(),
        "pulse ring should be despawned when radius >= max_radius"
    );
}

#[test]
fn despawn_finished_pulse_ring_when_radius_exceeds_max() {
    let mut app = test_app();
    enter_playing(&mut app);

    let ring = app
        .world_mut()
        .spawn((
            PulseRing,
            PulseRadius(50.1),
            PulseMaxRadius(50.0),
            PulseSpeed(0.0),
            PulseDamaged::default(),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    assert!(
        app.world().get_entity(ring).is_err(),
        "pulse ring should be despawned when radius > max_radius"
    );
}
