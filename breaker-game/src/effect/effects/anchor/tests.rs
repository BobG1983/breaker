//! Tests for the Anchor effect: `fire()`, `reverse()`, `tick_anchor()`, `register()`.

use bevy::prelude::*;

use super::effect::*;
use crate::{
    breaker::components::{Breaker, BreakerState, BreakerVelocity},
    effect::effects::bump_force::ActiveBumpForces,
    shared::{game_state::GameState, playing_state::PlayingState},
};

// ── helpers ────────────────────────────────────────────────────────

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, tick_anchor);
    app
}

fn test_app_fixed() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(FixedUpdate, tick_anchor);
    app
}

/// Accumulates one fixed timestep then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn register_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.init_state::<GameState>();
    app.add_sub_state::<PlayingState>();
    // Transition into Playing state so PlayingState::Active becomes active
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();
    app
}

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

// ── Behavior 7: tick_anchor inserts AnchorTimer on stationary breaker ──

#[test]
fn tick_anchor_inserts_timer_on_stationary_idle_breaker() {
    let mut app = test_app();
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

    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("tick_anchor should insert AnchorTimer on stationary idle breaker");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be initialized to plant_delay (0.3), got {}",
        timer.0
    );
}

#[test]
fn tick_anchor_inserts_timer_on_stationary_settling_breaker() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            BreakerVelocity { x: 0.0 },
            BreakerState::Settling,
        ))
        .id();

    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("tick_anchor should insert AnchorTimer on stationary settling breaker");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be initialized to plant_delay (0.3), got {}",
        timer.0
    );
}

// ── Behavior 8: AnchorTimer ticks down by dt each frame ───────────

#[test]
fn tick_anchor_timer_decrements_by_dt_while_stationary() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after one tick");
    // Fixed timestep is 1/64 by default in Bevy
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = 0.3 - dt;
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should be approximately {expected}, got {}",
        timer.0
    );
}

#[test]
fn tick_anchor_timer_accumulates_over_multiple_ticks() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // Run 6 fixed ticks
    for _ in 0..6 {
        tick(&mut app);
    }

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after 6 ticks");
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = (-6.0f32).mul_add(dt, 0.3);
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should be approximately {expected} after 6 ticks, got {}",
        timer.0
    );
}

// ── Behavior 9: AnchorTimer reaches zero -> AnchorPlanted inserted ──

#[test]
fn tick_anchor_timer_reaching_zero_inserts_planted_and_removes_timer() {
    let mut app = test_app_fixed();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();

    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            // Timer at 0.01 -- one tick at dt ~= 0.015625 will push it below zero.
            AnchorTimer(0.01),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer reaches zero (dt = {dt})"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when timer reaches zero"
    );
}

#[test]
fn tick_anchor_timer_exactly_one_dt_triggers_planted() {
    let mut app = test_app_fixed();
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();

    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            // Timer is exactly one dt -- after one tick it should be at or below zero.
            AnchorTimer(dt),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer was exactly one dt"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when timer was exactly one dt"
    );
}

// ── Behavior 10: breaker velocity cancels timer and planted ───────

#[test]
fn tick_anchor_velocity_cancels_timer_and_planted() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            AnchorPlanted,
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed when breaker has velocity"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed when breaker has velocity"
    );
    assert!(
        app.world().get::<AnchorActive>(entity).is_some(),
        "AnchorActive should still be present after movement cancel"
    );
}

// ── Behavior 11: dash state cancels timer and planted regardless of velocity ──

#[test]
fn tick_anchor_dashing_cancels_timer_and_planted_even_with_zero_velocity() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            AnchorPlanted,
            BreakerVelocity { x: 0.0 },
            BreakerState::Dashing,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed during Dashing state"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed during Dashing state"
    );
    assert!(
        app.world().get::<AnchorActive>(entity).is_some(),
        "AnchorActive should still be present after dash cancel"
    );
}

// ── Behavior 12: entity without AnchorActive is ignored ───────────

#[test]
fn tick_anchor_ignores_entity_without_anchor_active() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((BreakerVelocity { x: 0.0 }, BreakerState::Idle))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should not be inserted on entity without AnchorActive"
    );
    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should not be inserted on entity without AnchorActive"
    );
}

// ── Behavior 13: steady-state planted breaker stays planted ───────

#[test]
fn tick_anchor_steady_state_planted_remains_planted_no_timer() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should remain on a stationary planted breaker"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should not be re-inserted on an already planted breaker"
    );
}

// ── Behavior 14: movement cancel then re-stop inserts full delay timer ──

#[test]
fn tick_anchor_movement_cancel_then_restop_inserts_full_delay_timer() {
    let mut app = test_app();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.15),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: breaker is moving, timer should be cancelled.
    app.update();

    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be cancelled on first tick due to movement"
    );

    // Now stop the breaker.
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 0.0;

    // Second tick: breaker is stationary again, should get full plant_delay timer.
    app.update();

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should be re-inserted when breaker stops");
    assert!(
        (timer.0 - 0.3).abs() < f32::EPSILON,
        "AnchorTimer should be full plant_delay (0.3), not residual. Got {}",
        timer.0
    );
}

// ── Behavior 15: Settling state with zero velocity allows timer tick ──

#[test]
fn tick_anchor_settling_state_allows_timer_tick() {
    let mut app = test_app_fixed();
    let entity = app
        .world_mut()
        .spawn((
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.3),
            BreakerVelocity { x: 0.0 },
            BreakerState::Settling,
        ))
        .id();

    tick(&mut app);

    let timer = app
        .world()
        .get::<AnchorTimer>(entity)
        .expect("AnchorTimer should still exist after tick in Settling state");
    let dt = app
        .world()
        .resource::<Time<Fixed>>()
        .timestep()
        .as_secs_f32();
    let expected = 0.3 - dt;
    assert!(
        (timer.0 - expected).abs() < 0.001,
        "AnchorTimer should tick down in Settling state. Expected {expected}, got {}",
        timer.0
    );
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

// ── Behavior 9: Planted breaker has anchor bump_force_multiplier in ActiveBumpForces ──

#[test]
fn planted_breaker_has_anchor_force_in_active_bump_forces() {
    // Given: AnchorActive with bump_force_multiplier 2.0, ActiveBumpForces empty, stationary.
    // When: tick_anchor runs through timer countdown until planted.
    // Then: ActiveBumpForces contains [2.0].
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            ActiveBumpForces(vec![]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: inserts AnchorTimer(0.3)
    tick(&mut app);
    assert!(
        app.world().get::<AnchorTimer>(entity).is_some(),
        "first tick should insert AnchorTimer"
    );

    // Tick enough frames to exhaust the timer (~20 ticks at dt=1/64)
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "breaker should be planted after timer expires"
    );
    assert!(
        app.world().get::<AnchorTimer>(entity).is_none(),
        "AnchorTimer should be removed after planting"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain [2.0] after planting, got {:?}",
        forces.0
    );
}

#[test]
fn planted_breaker_appends_force_to_existing_entries() {
    // Edge case: ActiveBumpForces already has [1.5] from other effects.
    // After planting, should be [1.5, 2.0].
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // First tick: inserts AnchorTimer(0.3)
    tick(&mut app);

    // Tick enough frames for timer to expire
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "breaker should be planted"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5, 2.0],
        "ActiveBumpForces should append anchor's 2.0 to existing [1.5], got {:?}",
        forces.0
    );
}

// ── Behavior 10: Movement removes AnchorPlanted and pops bump_force_multiplier ──

#[test]
fn movement_removes_planted_and_pops_force_multiplier() {
    // Given: Planted with ActiveBumpForces [1.5, 2.0] (2.0 from anchor).
    // When: BreakerVelocity.x set to 200.0, tick_anchor runs.
    // Then: AnchorPlanted removed. ActiveBumpForces is [1.5].
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            ActiveBumpForces(vec![1.5, 2.0]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed on movement"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "anchor's 2.0 should be popped from ActiveBumpForces, got {:?}",
        forces.0
    );
}

#[test]
fn movement_pops_force_from_single_entry_forces() {
    // Edge case: ActiveBumpForces only contained [2.0], becomes [] after pop.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorPlanted,
            ActiveBumpForces(vec![2.0]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "AnchorPlanted should be removed on movement"
    );

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert!(
        forces.0.is_empty(),
        "ActiveBumpForces should be empty after popping sole entry, got {:?}",
        forces.0
    );
}

// ── Behavior 11: No AnchorPlanted means ActiveBumpForces unchanged ──

#[test]
fn no_planted_means_active_bump_forces_unchanged() {
    // AnchorActive present, moving (no timer/planted). ActiveBumpForces [1.5] stays.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "ActiveBumpForces should remain [1.5] when not planted, got {:?}",
        forces.0
    );
}

#[test]
fn no_anchor_active_means_active_bump_forces_unchanged() {
    // Edge case: No AnchorActive at all. ActiveBumpForces [1.5] stays.
    let mut app = test_app();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            ActiveBumpForces(vec![1.5]),
            BreakerVelocity { x: 200.0 },
            BreakerState::Idle,
        ))
        .id();

    app.update();

    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![1.5],
        "ActiveBumpForces should remain [1.5] without AnchorActive, got {:?}",
        forces.0
    );
}

// ── Behavior 12: Re-planting after cancellation re-adds force entry ──

#[test]
fn replanting_after_cancellation_readds_single_force_entry() {
    // Full cycle: stationary -> timer -> planted (force pushed) -> movement (force popped)
    // -> stationary again -> new timer -> new planted (force pushed).
    // After final planting, ActiveBumpForces contains exactly one anchor entry [2.0].
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            ActiveBumpForces(vec![]),
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    // Phase 1: stationary -> timer starts -> timer expires -> planted (force pushed)
    // First tick inserts timer
    tick(&mut app);
    // Tick enough for timer to expire (~20 ticks for 0.3s at 1/64)
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "should be planted after first cycle"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should be [2.0] after first planting"
    );

    // Phase 2: movement -> unplanted (force popped)
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 200.0;
    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_none(),
        "should be unplanted after movement"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert!(
        forces.0.is_empty(),
        "ActiveBumpForces should be empty after unplanting"
    );

    // Phase 3: stationary again -> new timer -> new planted (force pushed)
    app.world_mut()
        .get_mut::<BreakerVelocity>(entity)
        .unwrap()
        .x = 0.0;

    // First tick: new timer starts
    tick(&mut app);
    assert!(
        app.world().get::<AnchorTimer>(entity).is_some(),
        "new timer should start after stopping"
    );

    // Edge case verification: timer starts fresh from plant_delay, not residual
    let timer = app.world().get::<AnchorTimer>(entity).unwrap();
    assert!(
        (timer.0 - 0.3).abs() < 0.02,
        "new timer should start fresh from plant_delay (0.3), got {}",
        timer.0
    );

    // Tick enough for timer to expire again
    for _ in 0..20 {
        tick(&mut app);
    }

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "should be planted after second cycle"
    );
    let forces = app.world().get::<ActiveBumpForces>(entity).unwrap();
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain exactly one [2.0] after replanting (not cumulative)"
    );
}

// ── Behavior 13: tick_anchor handles missing ActiveBumpForces on plant ──

#[test]
fn tick_anchor_inserts_active_bump_forces_on_plant_when_missing() {
    // Given: No ActiveBumpForces. AnchorTimer about to expire.
    // When: tick_anchor runs, timer expires, AnchorPlanted inserted.
    // Then: ActiveBumpForces inserted with [2.0].
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.001), // about to expire
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
            // NO ActiveBumpForces
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted when timer expires"
    );

    let forces = app
        .world()
        .get::<ActiveBumpForces>(entity)
        .expect("ActiveBumpForces should be inserted when missing");
    assert_eq!(
        forces.0,
        vec![2.0],
        "ActiveBumpForces should contain [2.0], got {:?}",
        forces.0
    );
}

#[test]
fn tick_anchor_inserts_active_bump_forces_on_plant_when_no_active_forces() {
    // Edge case: no ActiveBumpForces on the entity before planting.
    // Only ActiveBumpForces should be inserted.
    let mut app = test_app_fixed();

    let entity = app
        .world_mut()
        .spawn((
            Breaker,
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay: 0.3,
            },
            AnchorTimer(0.001), // about to expire
            // NO ActiveBumpForces
            BreakerVelocity { x: 0.0 },
            BreakerState::Idle,
        ))
        .id();

    tick(&mut app);

    assert!(
        app.world().get::<AnchorPlanted>(entity).is_some(),
        "AnchorPlanted should be inserted"
    );

    let forces = app
        .world()
        .get::<ActiveBumpForces>(entity)
        .expect("ActiveBumpForces should be inserted");
    assert_eq!(forces.0, vec![2.0], "ActiveBumpForces should contain [2.0]");
}
