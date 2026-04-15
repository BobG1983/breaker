use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{
    bolt::components::BoltBaseDamage,
    effect_v3::{components::EffectSourceChip, effects::pulse::components::*},
};

// ── F. tick_pulse — timer, interval, and range formula ─────────────────

// #30
#[test]
fn tick_pulse_does_not_fire_when_timer_positive_after_decrement() {
    let mut app = emitter_test_app();

    let emitter = app
        .world_mut()
        .spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           1.0,
                source_chip:     EffectSourceChip(None),
            },
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));

    let count = app
        .world_mut()
        .query::<&PulseRing>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "no ring should spawn while timer > 0 after decrement"
    );

    let timer = app.world().get::<PulseEmitter>(emitter).unwrap().timer;
    assert!(
        (timer - 0.75).abs() < f32::EPSILON,
        "timer should equal 1.0 - 0.25 = 0.75, got {timer}",
    );
}

// #31
#[test]
fn tick_pulse_fires_when_timer_at_or_below_zero_and_refills_by_interval() {
    let mut app = emitter_test_app();

    let emitter = app
        .world_mut()
        .spawn((
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.25,
                source_chip:     EffectSourceChip(None),
            },
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));

    let count = app
        .world_mut()
        .query::<&PulseRing>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "one ring should spawn when timer hits 0");

    let timer = app.world().get::<PulseEmitter>(emitter).unwrap().timer;
    assert!(
        (timer - 1.0).abs() < f32::EPSILON,
        "timer should refill to 0.25 - 0.25 + 1.0 = 1.0, got {timer}",
    );
}

// #32
#[test]
fn tick_pulse_refires_on_second_tick_after_another_interval() {
    let mut app = emitter_test_app();

    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseEmitter {
            base_range:      64.0,
            range_per_level: 16.0,
            stacks:          1,
            speed:           200.0,
            interval:        0.25,
            timer:           0.25,
            source_chip:     EffectSourceChip(None),
        },
    ));

    tick_with_dt(&mut app, Duration::from_millis(250));
    tick_with_dt(&mut app, Duration::from_millis(250));

    let count = app
        .world_mut()
        .query::<&PulseRing>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 2,
        "two ticks at the firing interval should spawn two rings"
    );
}

// #33
#[test]
fn tick_pulse_max_radius_uses_base_range_for_stacks_one() {
    let mut app = emitter_test_app();

    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseEmitter {
            base_range:      64.0,
            range_per_level: 16.0,
            stacks:          1,
            speed:           200.0,
            interval:        1.0,
            timer:           0.0,
            source_chip:     EffectSourceChip(None),
        },
    ));

    tick_with_dt(&mut app, Duration::from_millis(250));

    let max_radii: Vec<f32> = app
        .world_mut()
        .query::<&PulseRingMaxRadius>()
        .iter(app.world())
        .map(|r| r.0)
        .collect();
    assert_eq!(max_radii.len(), 1, "expected 1 spawned pulse ring");
    assert!(
        (max_radii[0] - 64.0).abs() < f32::EPSILON,
        "stacks==1 must yield max_radius == base_range (64.0), got {}",
        max_radii[0],
    );
}

// #34
#[test]
fn tick_pulse_max_radius_adds_range_per_level_for_stacks_three() {
    let mut app = emitter_test_app();

    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseEmitter {
            base_range:      64.0,
            range_per_level: 16.0,
            stacks:          3,
            speed:           200.0,
            interval:        1.0,
            timer:           0.0,
            source_chip:     EffectSourceChip(None),
        },
    ));

    tick_with_dt(&mut app, Duration::from_millis(250));

    let max_radii: Vec<f32> = app
        .world_mut()
        .query::<&PulseRingMaxRadius>()
        .iter(app.world())
        .map(|r| r.0)
        .collect();
    assert_eq!(max_radii.len(), 1);
    // 64.0 + (3 - 1) * 16.0 == 64.0 + 32.0 == 96.0
    assert!(
        (max_radii[0] - 96.0).abs() < f32::EPSILON,
        "stacks==3 must yield max_radius == 96.0, got {}",
        max_radii[0],
    );
}

// #35
#[test]
fn tick_pulse_max_radius_uses_saturating_sub_for_stacks_zero() {
    let mut app = emitter_test_app();

    app.world_mut().spawn((
        Position2D(Vec2::ZERO),
        PulseEmitter {
            base_range:      64.0,
            range_per_level: 16.0,
            stacks:          0,
            speed:           200.0,
            interval:        1.0,
            timer:           0.0,
            source_chip:     EffectSourceChip(None),
        },
    ));

    tick_with_dt(&mut app, Duration::from_millis(250));

    let max_radii: Vec<f32> = app
        .world_mut()
        .query::<&PulseRingMaxRadius>()
        .iter(app.world())
        .map(|r| r.0)
        .collect();
    assert_eq!(max_radii.len(), 1);
    // saturating_sub(1) on 0u32 -> 0 -> (stacks_f32 == 0.0) -> max_radius == base_range
    assert!(
        (max_radii[0] - 64.0).abs() < f32::EPSILON,
        "stacks==0 must collapse to base_range via saturating_sub, got {}",
        max_radii[0],
    );
}

// #36
#[test]
fn tick_pulse_two_independent_emitters_produce_two_independent_rings_in_one_tick() {
    let mut app = emitter_test_app();

    let emitter_a = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ))
        .id();
    let emitter_b = app
        .world_mut()
        .spawn((
            BoltBaseDamage(10.0),
            Position2D(Vec2::ZERO),
            PulseEmitter {
                base_range:      64.0,
                range_per_level: 16.0,
                stacks:          1,
                speed:           200.0,
                interval:        1.0,
                timer:           0.0,
                source_chip:     EffectSourceChip(None),
            },
        ))
        .id();

    tick_with_dt(&mut app, Duration::from_millis(250));

    let ring_count = app
        .world_mut()
        .query::<&PulseRing>()
        .iter(app.world())
        .count();
    assert_eq!(
        ring_count, 2,
        "each of the two emitters should spawn one ring"
    );

    let radii: Vec<f32> = app
        .world_mut()
        .query::<&PulseRingRadius>()
        .iter(app.world())
        .map(|r| r.0)
        .collect();
    assert_eq!(radii.len(), 2);
    for r in &radii {
        assert!(
            r.abs() < f32::EPSILON,
            "newly spawned ring should start at radius 0, got {r}",
        );
    }

    let max_radii: Vec<f32> = app
        .world_mut()
        .query::<&PulseRingMaxRadius>()
        .iter(app.world())
        .map(|r| r.0)
        .collect();
    for m in &max_radii {
        assert!(
            (m - 64.0).abs() < f32::EPSILON,
            "spawned ring max_radius should be 64.0, got {m}",
        );
    }

    let damaged_lens: Vec<usize> = app
        .world_mut()
        .query::<&PulseRingDamaged>()
        .iter(app.world())
        .map(|d| d.0.len())
        .collect();
    assert_eq!(damaged_lens.len(), 2);
    for len in damaged_lens {
        assert_eq!(len, 0, "fresh PulseRingDamaged must be empty");
    }

    // Timer math: starting 0.0, decrement by dt=0.25 → -0.25, fire then
    // `timer += interval` (1.0) → 0.75. Mirrors the `+= interval` production
    // path that #31 also exercises (starting 0.25 → 0.0 → 1.0).
    let timer_a = app.world().get::<PulseEmitter>(emitter_a).unwrap().timer;
    let timer_b = app.world().get::<PulseEmitter>(emitter_b).unwrap().timer;
    assert!((timer_a - 0.75).abs() < f32::EPSILON);
    assert!((timer_b - 0.75).abs() < f32::EPSILON);
}
