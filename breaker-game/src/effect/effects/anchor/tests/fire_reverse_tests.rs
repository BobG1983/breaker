//! Tests for `fire()`, `reverse()`, `register()` lifecycle.

use super::helpers::*;

// ── Behavior 1: fire() inserts AnchorActive with correct values ───

#[test]
fn fire_inserts_anchor_active_with_correct_values() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 2.0, 1.5, 0.3, "", &mut world);

    let active = world
        .get::<AnchorActive>(entity)
        .expect("fire should insert AnchorActive");
    assert!(
        (active.bump_force_multiplier - 2.0).abs() < f32::EPSILON,
        "expected bump_force_multiplier 2.0, got {}",
        active.bump_force_multiplier
    );
    assert!(
        (active.perfect_window_multiplier - 1.5).abs() < f32::EPSILON,
        "expected perfect_window_multiplier 1.5, got {}",
        active.perfect_window_multiplier
    );
    assert!(
        (active.plant_delay - 0.3).abs() < f32::EPSILON,
        "expected plant_delay 0.3, got {}",
        active.plant_delay
    );
}

#[test]
fn fire_with_zero_plant_delay_inserts_anchor_active() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    fire(entity, 2.0, 1.5, 0.0, "", &mut world);

    let active = world
        .get::<AnchorActive>(entity)
        .expect("fire should insert AnchorActive even with plant_delay=0.0");
    assert!(
        active.plant_delay.abs() < f32::EPSILON,
        "expected plant_delay 0.0, got {}",
        active.plant_delay
    );
}

// ── Behavior 2: fire() overwrites existing AnchorActive ───────────

#[test]
fn fire_overwrites_existing_anchor_active_with_new_values() {
    let mut world = World::new();
    let entity = world
        .spawn(AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 1.5,
            plant_delay: 0.3,
        })
        .id();

    fire(entity, 3.0, 2.0, 0.5, "", &mut world);

    let active = world
        .get::<AnchorActive>(entity)
        .expect("fire should overwrite AnchorActive");
    assert!(
        (active.bump_force_multiplier - 3.0).abs() < f32::EPSILON,
        "expected bump_force_multiplier 3.0, got {}",
        active.bump_force_multiplier
    );
    assert!(
        (active.perfect_window_multiplier - 2.0).abs() < f32::EPSILON,
        "expected perfect_window_multiplier 2.0, got {}",
        active.perfect_window_multiplier
    );
    assert!(
        (active.plant_delay - 0.5).abs() < f32::EPSILON,
        "expected plant_delay 0.5, got {}",
        active.plant_delay
    );
}

#[test]
fn fire_with_identical_values_succeeds_without_error() {
    let mut world = World::new();
    let entity = world
        .spawn(AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 1.5,
            plant_delay: 0.3,
        })
        .id();

    fire(entity, 2.0, 1.5, 0.3, "", &mut world);

    let active = world
        .get::<AnchorActive>(entity)
        .expect("fire should succeed with identical values");
    assert!(
        (active.bump_force_multiplier - 2.0).abs() < f32::EPSILON,
        "bump_force_multiplier should remain 2.0"
    );
}

// ── Behavior 3: reverse() removes all three anchor components ─────

#[test]
fn reverse_removes_all_three_anchor_components() {
    let mut world = World::new();
    let entity = world
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            AnchorPlanted,
        ))
        .id();

    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);

    assert!(
        world.get::<AnchorActive>(entity).is_none(),
        "AnchorActive should be removed after reverse"
    );
    assert!(
        world.get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed after reverse"
    );
    assert!(
        world.get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed after reverse"
    );
}

#[test]
fn reverse_removes_only_anchor_active_when_no_timer_or_planted() {
    let mut world = World::new();
    let entity = world
        .spawn(AnchorActive {
            bump_force_multiplier: 2.0,
            perfect_window_multiplier: 1.5,
            plant_delay: 0.3,
        })
        .id();

    // Should not panic even though AnchorTimer and AnchorPlanted are absent.
    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);

    assert!(
        world.get::<AnchorActive>(entity).is_none(),
        "AnchorActive should be removed after reverse"
    );
}

// ── Behavior 4: reverse() on entity without anchor components ─────

#[test]
fn reverse_on_entity_without_anchor_components_is_noop() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    // Should not panic.
    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);

    assert!(
        world.get::<AnchorActive>(entity).is_none(),
        "entity should remain without AnchorActive"
    );
}

#[test]
fn reverse_called_twice_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);
    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);

    // Reaching here without panic is the assertion.
    assert!(
        world.get::<AnchorActive>(entity).is_none(),
        "entity should remain without AnchorActive"
    );
}

// ── Behavior 5: reverse() on despawned entity does not panic ──────

#[test]
fn reverse_on_despawned_entity_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.despawn(entity);

    // Should not panic on stale entity.
    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);
}

// ── Behavior 6: fire() then despawn then reverse() ────────────────

#[test]
fn fire_then_despawn_then_reverse_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    fire(entity, 2.0, 1.5, 0.3, "", &mut world);
    world.despawn(entity);

    // reverse() on the now-stale entity should not panic.
    reverse(entity, 2.0, 1.5, 0.3, "", &mut world);
}

// ── Behavior 16: fire() on despawned entity does not panic ────────

#[test]
fn fire_on_despawned_entity_does_not_panic() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    world.despawn(entity);

    // Should not panic on stale entity.
    fire(entity, 2.0, 1.5, 0.3, "", &mut world);
}

// ── Behavior 17: register() adds tick_anchor to FixedUpdate ───────

#[test]
fn register_adds_tick_anchor_system_to_fixed_update() {
    let mut app = register_test_app();

    register(&mut app);

    // Spawn an entity that tick_anchor should act on.
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // Run a fixed tick to exercise the registered system in PlayingState::Active.
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_some(),
        "register() should add tick_anchor to FixedUpdate gated by PlayingState::Active"
    );
}
