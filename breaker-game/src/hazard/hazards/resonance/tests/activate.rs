//! Group B — `activate` and `register` scaffold.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::super::system::{
    ResonanceActiveSlows, ResonanceConfig, ResonanceSlowEntry, ResonanceTracker, activate, register,
};
use crate::{hazard::definition::HazardTuning, prelude::*};

// ── B1 — activate inserts all three resources ───────────────────────────

#[test]
fn b1_activate_with_matching_tuning_inserts_all_three_resources() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Resonance {
                kills_to_trigger:      2,
                base_window:           0.5,
                window_per_level:      0.3,
                wave_speed:            200.0,
                base_slow_duration:    1.5,
                base_slow_strength:    0.5,
                slow_duration_scaling: 0.2,
                slow_strength_scaling: 0.15,
                contact_threshold:     16.0,
                wave_max_lifetime:     10.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app
        .world()
        .get_resource::<ResonanceConfig>()
        .expect("ResonanceConfig should be inserted after activate");
    assert_eq!(cfg.kills_to_trigger, 2);
    assert!((cfg.base_window - 0.5).abs() < f32::EPSILON);
    assert!((cfg.window_per_level - 0.3).abs() < f32::EPSILON);
    assert!((cfg.wave_speed - 200.0).abs() < f32::EPSILON);
    assert!((cfg.base_slow_duration - 1.5).abs() < f32::EPSILON);
    assert!((cfg.base_slow_strength - 0.5).abs() < f32::EPSILON);
    assert!((cfg.slow_duration_scaling - 0.2).abs() < f32::EPSILON);
    assert!((cfg.slow_strength_scaling - 0.15).abs() < f32::EPSILON);
    assert!((cfg.contact_threshold - 16.0).abs() < f32::EPSILON);
    assert!((cfg.wave_max_lifetime - 10.0).abs() < f32::EPSILON);

    let tracker = app
        .world()
        .get_resource::<ResonanceTracker>()
        .expect("ResonanceTracker should be inserted after activate");
    assert!(tracker.kills.is_empty());

    let slows = app
        .world()
        .get_resource::<ResonanceActiveSlows>()
        .expect("ResonanceActiveSlows should be inserted after activate");
    assert!(slows.slows.is_empty());
}

// ── B2 — mismatched tuning → no config, no panic ────────────────────────

#[test]
fn b2_activate_with_mismatched_tuning_does_nothing() {
    let mut app = TestAppBuilder::new().build();
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Decay {
                base_percent:      0.05,
                per_level_percent: 0.03,
            },
            &mut commands,
        );
    });
    app.update();
    assert!(
        app.world().get_resource::<ResonanceConfig>().is_none(),
        "Decay tuning must not insert a ResonanceConfig"
    );
}

// ── B3 — re-activation resets tracker and slows ─────────────────────────

#[test]
fn b3_reactivation_overwrites_config_and_resets_tracker_and_slows() {
    // Start with "mid-run state": Config at wave_speed=200, tracker + slows
    // carrying pre-existing entries.
    let mut app = TestAppBuilder::new().build();
    app.insert_resource(ResonanceConfig {
        kills_to_trigger:      2,
        base_window:           0.5,
        window_per_level:      0.3,
        wave_speed:            200.0,
        base_slow_duration:    1.5,
        base_slow_strength:    0.5,
        slow_duration_scaling: 0.2,
        slow_strength_scaling: 0.15,
        contact_threshold:     16.0,
        wave_max_lifetime:     10.0,
    });
    app.insert_resource({
        let mut t = ResonanceTracker::default();
        t.kills.push((0.5, Vec2::new(1.0, 2.0)));
        t
    });
    app.insert_resource({
        let mut s = ResonanceActiveSlows::default();
        s.slows.insert(
            "hazard:resonance:wave:42".into(),
            ResonanceSlowEntry {
                remaining:  1.0,
                multiplier: OrderedFloat(0.5),
            },
        );
        s
    });

    // Re-activate with wave_speed = 250.
    app.add_systems(Update, |mut commands: Commands| {
        activate(
            &HazardTuning::Resonance {
                kills_to_trigger:      2,
                base_window:           0.5,
                window_per_level:      0.3,
                wave_speed:            250.0,
                base_slow_duration:    1.5,
                base_slow_strength:    0.5,
                slow_duration_scaling: 0.2,
                slow_strength_scaling: 0.15,
                contact_threshold:     16.0,
                wave_max_lifetime:     10.0,
            },
            &mut commands,
        );
    });
    app.update();

    let cfg = app
        .world()
        .get_resource::<ResonanceConfig>()
        .expect("ResonanceConfig must be present after re-activate");
    assert!(
        (cfg.wave_speed - 250.0).abs() < f32::EPSILON,
        "wave_speed should be updated to 250.0, got {}",
        cfg.wave_speed
    );

    let tracker = app.world().resource::<ResonanceTracker>();
    assert!(
        tracker.kills.is_empty(),
        "Re-activation should reset tracker.kills to default, found {:?}",
        tracker.kills
    );
    let slows = app.world().resource::<ResonanceActiveSlows>();
    assert!(
        slows.slows.is_empty(),
        "Re-activation should reset slows to default, found len {}",
        slows.slows.len()
    );
}

// ── B4 — register initializes tracker + slows WITHOUT activate ─────────

#[test]
fn b4_register_initializes_resources_without_activate() {
    let mut app = TestAppBuilder::new().build();
    register(&mut app);
    app.update();

    assert!(
        app.world().get_resource::<ResonanceTracker>().is_some(),
        "register must init_resource::<ResonanceTracker>()"
    );
    assert!(
        app.world().resource::<ResonanceTracker>().kills.is_empty(),
        "ResonanceTracker default should have empty kills"
    );
    assert!(
        app.world().get_resource::<ResonanceActiveSlows>().is_some(),
        "register must init_resource::<ResonanceActiveSlows>()"
    );
    assert!(
        app.world()
            .resource::<ResonanceActiveSlows>()
            .slows
            .is_empty(),
        "ResonanceActiveSlows default should have empty slows map"
    );
    assert!(
        app.world().get_resource::<ResonanceConfig>().is_none(),
        "ResonanceConfig must NOT be present without activate"
    );
}
