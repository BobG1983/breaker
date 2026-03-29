use super::*;

// ── Behavior 11: tick_pulse_emitter spawns PulseRing when timer fires ──

#[test]
fn tick_pulse_emitter_spawns_ring_when_timer_reaches_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Transform::from_xyz(80.0, 120.0, 0.0),
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
        &Transform,
    )>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned, got {}",
        rings.len()
    );

    let (_ring, source, radius, max_radius, speed, damaged, transform) = rings[0];
    assert_eq!(
        source.0, bolt,
        "PulseSource should reference the bolt entity"
    );
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
        (transform.translation.x - 80.0).abs() < f32::EPSILON,
        "ring should spawn at bolt x position"
    );
    assert!(
        (transform.translation.y - 120.0).abs() < f32::EPSILON,
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

// ── Behavior 12: tick_pulse_emitter does NOT spawn before interval ──

#[test]
fn tick_pulse_emitter_does_not_spawn_before_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
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

// ── Behavior 13: tick_pulse_emitter reads bolt's current position ──

#[test]
fn tick_pulse_emitter_reads_current_bolt_position() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Bolt starts at (200, 300) -- this is its "current" position
    let _bolt = app
        .world_mut()
        .spawn((
            Transform::from_xyz(200.0, 300.0, 0.0),
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

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Transform)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(rings.len(), 1, "expected one ring spawned");

    let (_ring, transform) = rings[0];
    assert!(
        (transform.translation.x - 200.0).abs() < f32::EPSILON,
        "ring should spawn at bolt's current x (200.0), got {}",
        transform.translation.x
    );
    assert!(
        (transform.translation.y - 300.0).abs() < f32::EPSILON,
        "ring should spawn at bolt's current y (300.0), got {}",
        transform.translation.y
    );
}

// ── Behavior 14: tick_pulse_ring expands radius ──

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

// ── Behavior 15: despawn_finished_pulse_ring ──

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

// ── Damage scaling: Pulse emitter propagates damage multiplier to spawned rings ──

#[test]
fn tick_pulse_emitter_propagates_damage_multiplier_to_spawned_ring() {
    let mut app = test_app();
    enter_playing(&mut app);

    let _bolt = app
        .world_mut()
        .spawn((
            Transform::from_xyz(50.0, 50.0, 0.0),
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

    app.update();

    // Query the spawned ring for PulseRingDamageMultiplier
    let mut ring_query = app
        .world_mut()
        .query::<(&PulseRing, &PulseRingDamageMultiplier)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned, got {}",
        rings.len()
    );

    let (_ring, damage_mult) = rings[0];
    // When PulseEmitter has no captured EDM, the ring should carry default 1.0
    assert!(
        (damage_mult.0 - 1.0).abs() < f32::EPSILON,
        "ring should carry PulseRingDamageMultiplier(1.0) by default, got {}",
        damage_mult.0
    );
}

// Note: Testing propagation of a non-default multiplier (e.g., 2.5)
// requires adding `effective_damage_multiplier` field to `PulseEmitter`.
// The first test above covers the observable behavior: spawned rings
// must carry `PulseRingDamageMultiplier`. The writer-code will add the
// field to `PulseEmitter` and the full propagation test will be
// validated by the damage_tests that exercise the ring with non-1.0 multipliers.

// ── Behavior 4: tick_pulse_emitter respects custom interval ──

#[test]
fn tick_pulse_emitter_respects_custom_interval() {
    let mut app = test_app();
    enter_playing(&mut app);

    let bolt = app
        .world_mut()
        .spawn((
            Transform::from_xyz(80.0, 120.0, 0.0),
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

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Transform)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned with custom interval 0.25, got {}",
        rings.len()
    );

    let (_ring, transform) = rings[0];
    assert!(
        (transform.translation.x - 80.0).abs() < f32::EPSILON,
        "ring should spawn at bolt x position (80.0), got {}",
        transform.translation.x
    );
    assert!(
        (transform.translation.y - 120.0).abs() < f32::EPSILON,
        "ring should spawn at bolt y position (120.0), got {}",
        transform.translation.y
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
            Transform::from_xyz(0.0, 0.0, 0.0),
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

// -- Pulse Tick: EffectSourceChip propagation from emitter to ring ───────────────────

use crate::effect::core::EffectSourceChip;

#[test]
fn tick_pulse_emitter_copies_effect_source_chip_from_emitter_to_spawned_ring() {
    let mut app = test_app();
    enter_playing(&mut app);

    app.world_mut().spawn((
        Transform::from_xyz(50.0, 50.0, 0.0),
        PulseEmitter {
            base_range: 32.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 50.0,
            interval: 0.5,
            timer: 0.49, // About to fire
        },
        EffectSourceChip(Some("resonance".to_string())),
    ));

    app.update();

    // Query spawned rings for EffectSourceChip
    let mut ring_query = app.world_mut().query::<(&PulseRing, &EffectSourceChip)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing with EffectSourceChip, got {}",
        rings.len()
    );

    let (_ring, source_chip) = rings[0];
    assert_eq!(
        source_chip.0,
        Some("resonance".to_string()),
        "spawned ring should copy EffectSourceChip from emitter"
    );
}

#[test]
fn tick_pulse_emitter_copies_effect_source_chip_none_from_emitter() {
    let mut app = test_app();
    enter_playing(&mut app);

    app.world_mut().spawn((
        Transform::from_xyz(50.0, 50.0, 0.0),
        PulseEmitter {
            base_range: 32.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 50.0,
            interval: 0.5,
            timer: 0.49,
        },
        EffectSourceChip(None),
    ));

    app.update();

    let mut ring_query = app.world_mut().query::<(&PulseRing, &EffectSourceChip)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing with EffectSourceChip"
    );

    let (_ring, source_chip) = rings[0];
    assert_eq!(
        source_chip.0, None,
        "spawned ring should copy EffectSourceChip(None) from emitter"
    );
}

#[test]
fn tick_pulse_emitter_spawns_ring_with_default_effect_source_chip_when_emitter_has_none() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Emitter with NO EffectSourceChip component
    app.world_mut().spawn((
        Transform::from_xyz(50.0, 50.0, 0.0),
        PulseEmitter {
            base_range: 32.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 50.0,
            interval: 0.5,
            timer: 0.49,
        },
    ));

    app.update();

    // Ring should exist and have EffectSourceChip(None) — the system always
    // inserts EffectSourceChip, defaulting to None when the emitter lacks one.
    let mut ring_query = app.world_mut().query::<(&PulseRing, &EffectSourceChip)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing with EffectSourceChip"
    );

    let (_ring, source_chip) = rings[0];
    assert_eq!(
        source_chip.0, None,
        "ring should have EffectSourceChip(None) when emitter has no EffectSourceChip component"
    );
}
