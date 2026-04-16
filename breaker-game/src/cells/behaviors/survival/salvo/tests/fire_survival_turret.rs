//! Tests for `fire_survival_turret` — behaviors 15-23.

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    cells::{
        behaviors::{
            phantom::components::PhantomPhase,
            survival::{
                components::SurvivalTimer,
                salvo::components::{
                    SALVO_FIRE_INTERVAL, SALVO_HALF_EXTENT, SALVO_SPEED, Salvo, SalvoFireTimer,
                    SalvoSource,
                },
            },
        },
        definition::AttackPattern,
    },
    prelude::*,
};

/// Helper: counts Salvo entities in the world.
fn count_salvos(app: &mut App) -> usize {
    let mut query = app.world_mut().query_filtered::<Entity, With<Salvo>>();
    query.iter(app.world()).count()
}

// ── Behavior 15: Turret fires a salvo when SalvoFireTimer <= 0 (StraightDown) ──

#[test]
fn turret_fires_salvo_when_timer_expired_straight_down() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0, // SalvoFireTimer(0.0) — expired
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        1,
        "one salvo should be spawned for StraightDown pattern"
    );

    // Verify salvo components
    let mut query = app
        .world_mut()
        .query::<(Entity, &SalvoSource, &Position2D, &Velocity2D, &Aabb2D)>();
    let (_, salvo_source, salvo_pos, salvo_vel, salvo_aabb) = query
        .iter(app.world())
        .find(|(_, src, ..)| src.0 == turret)
        .expect("should find salvo with source == turret");

    assert_eq!(
        salvo_source.0, turret,
        "SalvoSource should reference the turret entity"
    );
    assert!(
        (salvo_pos.0.x - 50.0).abs() < f32::EPSILON,
        "salvo x should be at turret x (50.0), got {}",
        salvo_pos.0.x
    );
    // Velocity should be straight down at SALVO_SPEED
    assert!(
        salvo_vel.0.x.abs() < f32::EPSILON,
        "salvo vx should be 0.0 for StraightDown, got {}",
        salvo_vel.0.x
    );
    assert!(
        (salvo_vel.0.y - (-SALVO_SPEED)).abs() < 0.01,
        "salvo vy should be {} for StraightDown, got {}",
        -SALVO_SPEED,
        salvo_vel.0.y
    );
    // Aabb2D half-extents should be SALVO_HALF_EXTENT x SALVO_HALF_EXTENT
    assert!(
        (salvo_aabb.half_extents.x - SALVO_HALF_EXTENT).abs() < f32::EPSILON,
        "salvo AABB half-extent x should be {SALVO_HALF_EXTENT}, got {}",
        salvo_aabb.half_extents.x
    );
    assert!(
        (salvo_aabb.half_extents.y - SALVO_HALF_EXTENT).abs() < f32::EPSILON,
        "salvo AABB half-extent y should be {SALVO_HALF_EXTENT}, got {}",
        salvo_aabb.half_extents.y
    );
}

// Behavior 15 edge: SalvoFireTimer exactly 0.0 fires
#[test]
fn turret_fires_at_exactly_zero_timer() {
    let mut app = build_fire_survival_turret_app();

    spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(count_salvos(&mut app), 1, "timer == 0.0 should fire");
}

// ── Behavior 16: Turret sets SurvivalTimer.started = true on first fire ──

#[test]
fn turret_sets_survival_timer_started_on_first_fire() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    assert!(
        timer.started,
        "SurvivalTimer.started should be true after first fire"
    );
}

// Behavior 16 edge: already started remains true (idempotent)
#[test]
fn turret_already_started_stays_true() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   true,
        }),
        0.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let timer = app
        .world()
        .get::<SurvivalTimer>(turret)
        .expect("turret should have SurvivalTimer");
    assert!(timer.started, "started should remain true");
}

// ── Behavior 17: SalvoFireTimer resets to SALVO_FIRE_INTERVAL after firing ──

#[test]
fn fire_timer_resets_after_firing() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        -0.1, // recently expired
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let fire_timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    assert!(
        (fire_timer.0 - SALVO_FIRE_INTERVAL).abs() < f32::EPSILON,
        "SalvoFireTimer should reset to {SALVO_FIRE_INTERVAL}, got {}",
        fire_timer.0
    );
}

// Behavior 17 edge: very overdue timer still resets to interval (no catch-up)
#[test]
fn very_overdue_timer_resets_without_catch_up() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        -5.0, // very overdue
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    // Should only spawn 1 salvo, not catch up
    assert_eq!(
        count_salvos(&mut app),
        1,
        "should not fire multiple salvos to catch up"
    );

    let fire_timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    assert!(
        (fire_timer.0 - SALVO_FIRE_INTERVAL).abs() < f32::EPSILON,
        "timer should reset to {SALVO_FIRE_INTERVAL}, got {}",
        fire_timer.0
    );
}

// ── Behavior 18: Turret does NOT fire when SalvoFireTimer > 0 ──

#[test]
fn turret_does_not_fire_when_cooldown_active() {
    let mut app = build_fire_survival_turret_app();

    spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        1.5, // cooldown active
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        0,
        "no salvo should spawn when cooldown is active"
    );
}

// Behavior 18 edge: SalvoFireTimer(0.001) — strictly positive, does not fire
#[test]
fn turret_does_not_fire_at_barely_positive_timer() {
    let mut app = build_fire_survival_turret_app();

    spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.001,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        0,
        "SalvoFireTimer(0.001) is positive, should not fire"
    );
}

// ── Behavior 20: Turret in PhantomPhase::Ghost does NOT fire ──

#[test]
fn ghost_turret_does_not_fire() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );
    app.world_mut()
        .entity_mut(turret)
        .insert(PhantomPhase::Ghost);

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(count_salvos(&mut app), 0, "Ghost turret should not fire");
}

// Behavior 20 edge: PhantomPhase::Solid fires normally
#[test]
fn solid_phase_turret_fires_normally() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );
    app.world_mut()
        .entity_mut(turret)
        .insert(PhantomPhase::Solid);

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        1,
        "Solid phase turret should fire normally"
    );
}

// Behavior 20 edge: PhantomPhase::Telegraph fires normally
#[test]
fn telegraph_phase_turret_fires_normally() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );
    app.world_mut()
        .entity_mut(turret)
        .insert(PhantomPhase::Telegraph);

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        1,
        "Telegraph phase turret should fire normally"
    );
}

// Behavior 20 edge: no PhantomPhase component fires normally
#[test]
fn turret_without_phantom_phase_fires_normally() {
    let mut app = build_fire_survival_turret_app();

    spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );
    // No PhantomPhase inserted — Option<&PhantomPhase> is None

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        1,
        "turret without PhantomPhase should fire normally"
    );
}

// ── Behavior 21: Turret without SurvivalTimer (permanent) fires normally ──

#[test]
fn permanent_turret_fires_without_survival_timer() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        None, // no SurvivalTimer
        0.0,
        AttackPattern::StraightDown,
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(
        count_salvos(&mut app),
        1,
        "permanent turret (no SurvivalTimer) should fire normally"
    );

    // Should not panic from missing SurvivalTimer
    assert!(
        app.world().get::<SurvivalTimer>(turret).is_none(),
        "turret should still have no SurvivalTimer"
    );

    // Fire timer should be reset
    let fire_timer = app
        .world()
        .get::<SalvoFireTimer>(turret)
        .expect("turret should have SalvoFireTimer");
    assert!(
        (fire_timer.0 - SALVO_FIRE_INTERVAL).abs() < f32::EPSILON,
        "fire timer should reset to {SALVO_FIRE_INTERVAL}, got {}",
        fire_timer.0
    );
}

// ── Behavior 22: Turret fires spread pattern (Spread(3)) ──

#[test]
fn turret_fires_spread_3_spawns_three_salvos() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::Spread(3),
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    let mut query = app
        .world_mut()
        .query::<(Entity, &SalvoSource, &Velocity2D)>();
    let salvos: Vec<(Entity, Entity, Vec2)> = query
        .iter(app.world())
        .filter(|(_, src, _)| src.0 == turret)
        .map(|(e, s, v)| (e, s.0, v.0))
        .collect();

    assert_eq!(salvos.len(), 3, "Spread(3) should spawn 3 salvos");

    // All should have same speed magnitude
    for (_, _, vel) in &salvos {
        let speed = vel.length();
        assert!(
            (speed - SALVO_SPEED).abs() < 1.0,
            "each salvo should have speed ~{SALVO_SPEED}, got {speed}"
        );
    }

    // Check spread: should have left (negative x), center (0 x), right (positive x)
    let mut x_velocities: Vec<f32> = salvos.iter().map(|(_, _, v)| v.x).collect();
    x_velocities.sort_by(|a, b| a.partial_cmp(b).unwrap());

    assert!(
        x_velocities[0] < -0.1,
        "leftmost salvo should have negative x velocity, got {}",
        x_velocities[0]
    );
    assert!(
        x_velocities[1].abs() < 1.0,
        "center salvo should have near-zero x velocity, got {}",
        x_velocities[1]
    );
    assert!(
        x_velocities[2] > 0.1,
        "rightmost salvo should have positive x velocity, got {}",
        x_velocities[2]
    );

    // All y velocities should be negative (downward)
    for (_, _, vel) in &salvos {
        assert!(
            vel.y < 0.0,
            "all salvos should move downward, got vy={}",
            vel.y
        );
    }
}

// Behavior 22 edge: Spread(1) spawns 1 salvo straight down
#[test]
fn turret_fires_spread_1_spawns_one_salvo_straight_down() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::Spread(1),
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(count_salvos(&mut app), 1, "Spread(1) should spawn 1 salvo");

    let mut query = app.world_mut().query::<(&SalvoSource, &Velocity2D)>();
    let (_, vel) = query
        .iter(app.world())
        .find(|(src, _)| src.0 == turret)
        .expect("should find salvo");
    assert!(
        vel.0.x.abs() < 1.0,
        "Spread(1) salvo should go straight down, vx={}",
        vel.0.x
    );
    assert!(vel.0.y < 0.0, "Spread(1) salvo should move downward");
}

// Behavior 22 edge: Spread(0) spawns 0 salvos
#[test]
fn turret_fires_spread_0_spawns_no_salvos() {
    let mut app = build_fire_survival_turret_app();

    spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::Spread(0),
    );

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(count_salvos(&mut app), 0, "Spread(0) should spawn 0 salvos");
}

// ── Behavior 23: Dead turret does NOT fire ──

#[test]
fn dead_turret_does_not_fire() {
    let mut app = build_fire_survival_turret_app();

    let turret = spawn_turret_manual(
        &mut app,
        Vec2::new(50.0, 200.0),
        50.0,
        Some(SurvivalTimer {
            remaining: 10.0,
            started:   false,
        }),
        0.0,
        AttackPattern::StraightDown,
    );
    app.world_mut().entity_mut(turret).insert(Dead);

    advance_to_playing(&mut app);
    tick(&mut app);

    assert_eq!(count_salvos(&mut app), 0, "Dead turret should not fire");
}
