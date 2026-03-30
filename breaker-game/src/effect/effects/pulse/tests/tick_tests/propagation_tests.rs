//! Tests for propagation of `PulseRingDamageMultiplier`, `EffectSourceChip`,
//! and `Position2D` from `PulseEmitter` to spawned `PulseRing` entities.

use super::super::*;
use crate::effect::core::EffectSourceChip;

// -- Damage scaling: Pulse emitter propagates damage multiplier to spawned rings --

#[test]
fn tick_pulse_emitter_propagates_damage_multiplier_to_spawned_ring() {
    let mut app = test_app();
    enter_playing(&mut app);

    let _bolt = app
        .world_mut()
        .spawn((
            Position2D(Vec2::new(50.0, 50.0)),
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

// -- Pulse Tick: EffectSourceChip propagation from emitter to ring --

#[test]
fn tick_pulse_emitter_copies_effect_source_chip_from_emitter_to_spawned_ring() {
    let mut app = test_app();
    enter_playing(&mut app);

    app.world_mut().spawn((
        Position2D(Vec2::new(50.0, 50.0)),
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
        Position2D(Vec2::new(50.0, 50.0)),
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
        Position2D(Vec2::new(50.0, 50.0)),
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

// -- Behavior: tick_pulse_emitter reads Position2D for ring spawn position --

#[test]
fn tick_pulse_emitter_reads_position2d_for_ring_spawn_position() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Emitter at Position2D(200, 300) with NO Transform
    app.world_mut().spawn((
        Position2D(Vec2::new(200.0, 300.0)),
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

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Position2D)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned, got {}",
        rings.len()
    );

    let (_ring, pos) = rings[0];
    assert!(
        (pos.0.x - 200.0).abs() < f32::EPSILON,
        "ring should spawn at emitter Position2D x=200.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 300.0).abs() < f32::EPSILON,
        "ring should spawn at emitter Position2D y=300.0, got {}",
        pos.0.y
    );
}

// -- Behavior: tick_pulse_emitter uses Position2D not Transform when both present --

#[test]
fn tick_pulse_emitter_uses_position2d_not_transform_when_both_present() {
    let mut app = test_app();
    enter_playing(&mut app);

    // Position2D and Transform are intentionally divergent
    app.world_mut().spawn((
        Position2D(Vec2::new(200.0, 300.0)),
        Transform::from_xyz(999.0, 888.0, 0.0),
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

    let mut ring_query = app.world_mut().query::<(&PulseRing, &Position2D)>();
    let rings: Vec<_> = ring_query.iter(app.world()).collect();
    assert_eq!(
        rings.len(),
        1,
        "expected one PulseRing spawned, got {}",
        rings.len()
    );

    let (_ring, pos) = rings[0];
    assert!(
        (pos.0.x - 200.0).abs() < f32::EPSILON,
        "ring should use Position2D x=200.0, not Transform x=999.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 300.0).abs() < f32::EPSILON,
        "ring should use Position2D y=300.0, not Transform y=888.0, got {}",
        pos.0.y
    );
}
