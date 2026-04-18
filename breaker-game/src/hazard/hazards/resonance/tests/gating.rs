//! Group H — run-condition gating on `hazard_active` + `NodeState::Playing`.

use std::time::Duration;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;

use super::{
    super::system::{
        ResonanceActiveSlows, ResonanceConfig, ResonanceSlowEntry, ResonanceTracker, ResonanceWave,
        resonance_spawn_waves, resonance_tick_slows, resonance_track_kills, resonance_wave_contact,
        resonance_wave_expire, resonance_wave_travel,
    },
    helpers::{
        spawn_breaker_at, test_app_playing, test_app_playing_no_stack, tick_with_dt,
        write_cell_destroyed,
    },
};
use crate::{
    breaker::components::Breaker,
    hazard::{
        definition::HazardKind,
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
};

// ── H1 — track_kills gated off when Resonance stack == 0 ────────────────

#[test]
fn h1_track_kills_blocked_when_resonance_stack_zero() {
    let mut app = test_app_playing_no_stack();
    app.add_systems(
        FixedUpdate,
        resonance_track_kills.run_if(hazard_active(HazardKind::Resonance)),
    );

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let tracker = app.world().resource::<ResonanceTracker>();
    assert!(
        tracker.kills.is_empty(),
        "Gate should block track_kills → empty tracker, got {:?}",
        tracker.kills
    );
    let mut q = app.world_mut().query::<&ResonanceWave>();
    assert_eq!(q.iter(app.world()).count(), 0);
}

// ── H2 — spawn_waves without ResonanceConfig gracefully skips ──────────

#[test]
fn h2_spawn_waves_does_nothing_without_resonance_config() {
    let mut app = test_app_playing();
    // Remove config after test_app_playing inserted it.
    app.world_mut().remove_resource::<ResonanceConfig>();
    app.add_systems(FixedUpdate, resonance_spawn_waves);
    spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];

    write_cell_destroyed(&mut app, Vec2::new(10.0, 10.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(0.1));

    let mut q = app.world_mut().query::<&ResonanceWave>();
    assert_eq!(
        q.iter(app.world()).count(),
        0,
        "No wave should spawn when ResonanceConfig is absent"
    );
}

// ── H3 — all six systems gated off when not in NodeState::Playing ──────

#[test]
fn h3_all_systems_gated_off_outside_playing() {
    // Build a Loading-state app (don't advance to Playing).
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveHazards>()
        .with_message::<crate::shared::death_pipeline::Destroyed<crate::cells::components::Cell>>()
        .build();
    app.insert_resource(super::helpers::canonical_config());
    app.init_resource::<ResonanceTracker>();
    app.init_resource::<ResonanceActiveSlows>();
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Resonance);

    app.add_systems(
        FixedUpdate,
        (
            resonance_track_kills,
            resonance_spawn_waves,
            resonance_wave_travel,
            resonance_wave_contact,
            resonance_wave_expire,
            resonance_tick_slows,
        )
            .run_if(in_state(NodeState::Playing)),
    );

    // Seed pre-existing tracker entries.
    app.world_mut().resource_mut::<ResonanceTracker>().kills =
        vec![(0.0, Vec2::new(1.0, 1.0)), (0.1, Vec2::new(2.0, 2.0))];
    // Spawn a wave manually.
    let wave = app
        .world_mut()
        .spawn((
            ResonanceWave {
                speed:             200.0,
                slow_duration:     1.5,
                slow_strength:     0.5,
                target_pos:        Vec2::new(0.0, 50.0),
                age:               0.0,
                max_lifetime:      10.0,
                contact_threshold: 16.0,
            },
            Position2D(Vec2::new(0.0, 300.0)),
        ))
        .id();
    // Seed ResonanceActiveSlows entry.
    app.world_mut()
        .resource_mut::<ResonanceActiveSlows>()
        .slows
        .insert(
            "hazard:resonance:wave:999".into(),
            ResonanceSlowEntry {
                remaining:  0.5,
                multiplier: OrderedFloat(0.5),
            },
        );

    write_cell_destroyed(&mut app, Vec2::new(50.0, 50.0));
    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    // Wave unchanged.
    let pos_after = app.world().get::<Position2D>(wave).unwrap().0;
    assert!(
        (pos_after - Vec2::new(0.0, 300.0)).length() < f32::EPSILON,
        "Travel must not run off-Playing, got {pos_after:?}"
    );
    let wave_age = app.world().get::<ResonanceWave>(wave).unwrap().age;
    assert!(
        (wave_age - 0.0).abs() < f32::EPSILON,
        "Age must not tick off-Playing, got {wave_age}"
    );

    // Tracker unchanged (2 entries — no new add).
    let tracker = app.world().resource::<ResonanceTracker>();
    assert_eq!(
        tracker.kills.len(),
        2,
        "track_kills gated off → tracker size unchanged (still the 2 seeded), got {:?}",
        tracker.kills
    );
    // Wave entity still exists (contact + expire gated off).
    assert!(
        app.world().get_entity(wave).is_ok(),
        "Wave must not be despawned when systems are gated off"
    );
    // Slows entry unchanged.
    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    let entry = slows
        .get("hazard:resonance:wave:999")
        .expect("slows entry should still exist");
    assert!(
        (entry.remaining - 0.5).abs() < f32::EPSILON,
        "Tick gated off → remaining unchanged, got {}",
        entry.remaining
    );
    assert_eq!(entry.multiplier, OrderedFloat(0.5));
}

// ── H4 — tick_slows does nothing when hazard is inactive ──────────────

#[test]
fn h4_tick_slows_gated_off_when_hazard_inactive() {
    let mut app = test_app_playing_no_stack();
    app.add_systems(
        FixedUpdate,
        resonance_tick_slows.run_if(hazard_active(HazardKind::Resonance)),
    );
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));
    app.world_mut()
        .resource_mut::<ResonanceActiveSlows>()
        .slows
        .insert(
            "hazard:resonance:wave:hold".into(),
            ResonanceSlowEntry {
                remaining:  0.1,
                multiplier: OrderedFloat(0.5),
            },
        );
    // Fire the matching stack entry so we can detect reversal.
    {
        use crate::effect_v3::{effects::SpeedBoostConfig, traits::Fireable};
        let cfg = SpeedBoostConfig {
            multiplier: OrderedFloat(0.5),
        };
        cfg.fire(breaker, "hazard:resonance:wave:hold", app.world_mut());
    }

    tick_with_dt(&mut app, Duration::from_secs_f32(1.0));

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    let entry = slows
        .get("hazard:resonance:wave:hold")
        .expect("entry should be untouched");
    assert!(
        (entry.remaining - 0.1).abs() < f32::EPSILON,
        "remaining must be unchanged when gate is off; got {}",
        entry.remaining
    );
    assert_eq!(entry.multiplier, OrderedFloat(0.5));
    let mut breakers = app.world_mut().query::<&Breaker>();
    assert_eq!(breakers.iter(app.world()).count(), 1);
}
