use super::*;

// ── Behavior 8: fire() adds PulseEmitter to the target entity ──

#[test]
fn fire_adds_pulse_emitter_to_entity() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 32.0, 8.0, 1, 50.0, &mut world);

    let emitter = world
        .get::<PulseEmitter>(entity)
        .expect("entity should have PulseEmitter after fire()");

    assert!(
        (emitter.base_range - 32.0).abs() < f32::EPSILON,
        "expected base_range 32.0, got {}",
        emitter.base_range
    );
    assert!(
        (emitter.range_per_level - 8.0).abs() < f32::EPSILON,
        "expected range_per_level 8.0, got {}",
        emitter.range_per_level
    );
    assert_eq!(emitter.stacks, 1, "expected stacks 1");
    assert!(
        (emitter.speed - 50.0).abs() < f32::EPSILON,
        "expected speed 50.0, got {}",
        emitter.speed
    );
    assert!(
        (emitter.interval - 0.5).abs() < f32::EPSILON,
        "expected interval 0.5 (default), got {}",
        emitter.interval
    );
    assert!(
        (emitter.timer - 0.0).abs() < f32::EPSILON,
        "expected timer 0.0, got {}",
        emitter.timer
    );
}

#[test]
fn fire_overwrites_existing_pulse_emitter() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

    fire(entity, 32.0, 8.0, 1, 50.0, &mut world);
    fire(entity, 64.0, 16.0, 2, 100.0, &mut world);

    let emitter = world
        .get::<PulseEmitter>(entity)
        .expect("entity should have PulseEmitter after second fire()");

    assert!(
        (emitter.base_range - 64.0).abs() < f32::EPSILON,
        "expected overwritten base_range 64.0, got {}",
        emitter.base_range
    );
    assert_eq!(emitter.stacks, 2, "expected overwritten stacks 2");
}

// ── Behavior 9: effective_range scales with stacks ──

#[test]
fn effective_max_radius_scales_with_stacks() {
    let emitter = PulseEmitter {
        base_range: 32.0,
        range_per_level: 8.0,
        stacks: 3,
        speed: 50.0,
        interval: 0.5,
        timer: 0.0,
    };

    // 32.0 + (3-1)*8.0 = 48.0
    let radius = emitter.effective_max_radius();
    assert!(
        (radius - 48.0).abs() < f32::EPSILON,
        "expected effective_max_radius 48.0, got {radius}"
    );
}

#[test]
fn effective_max_radius_with_zero_stacks_uses_base_range() {
    let emitter = PulseEmitter {
        base_range: 32.0,
        range_per_level: 8.0,
        stacks: 0,
        speed: 50.0,
        interval: 0.5,
        timer: 0.0,
    };

    // saturating_sub(1) on 0 = 0, so effective = base_range = 32.0
    let radius = emitter.effective_max_radius();
    assert!(
        (radius - 32.0).abs() < f32::EPSILON,
        "expected effective_max_radius 32.0 for stacks=0, got {radius}"
    );
}

// ── Behavior 10: reverse() removes PulseEmitter ──

#[test]
fn reverse_removes_pulse_emitter() {
    let mut world = World::new();
    let entity = world
        .spawn((
            Transform::from_xyz(100.0, 200.0, 0.0),
            PulseEmitter {
                base_range: 32.0,
                range_per_level: 8.0,
                stacks: 1,
                speed: 50.0,
                interval: 0.5,
                timer: 0.0,
            },
        ))
        .id();

    reverse(entity, &mut world);

    assert!(
        world.get::<PulseEmitter>(entity).is_none(),
        "PulseEmitter should be removed after reverse()"
    );
}

#[test]
fn reverse_on_entity_without_emitter_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

    // Should not panic
    reverse(entity, &mut world);

    assert!(
        world.get_entity(entity).is_ok(),
        "entity should still exist after no-op reverse"
    );
}
