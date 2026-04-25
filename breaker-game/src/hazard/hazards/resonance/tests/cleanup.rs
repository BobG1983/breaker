//! Group I — node-end cleanup on `OnEnter(NodeState::Teardown)`.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_stateflow::cleanup_on_exit;

use super::{
    super::system::{
        ResonanceActiveSlows, ResonanceConfig, ResonanceSlowEntry, ResonanceTracker, ResonanceWave,
        resonance_cleanup_on_teardown,
    },
    helpers::{collect_sources_with_prefix, spawn_breaker_at, test_app_playing_with_effects},
};
use crate::{
    effect_v3::{effects::SpeedBoostConfig, stacking::EffectStack, traits::Fireable},
    prelude::*,
};

fn transition_to_teardown(app: &mut App) {
    // Playing → AnimateOut
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    // AnimateOut → Teardown
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();
}

// ── I1 — wave entities despawned via CleanupOnExit<NodeState> ─────────

#[test]
fn i1_waves_despawned_on_teardown_via_cleanup_on_exit() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);

    let wave_a = app
        .world_mut()
        .spawn((
            ResonanceWave {
                speed:             200.0,
                slow_duration:     1.5,
                slow_strength:     0.5,
                target_pos:        Vec2::ZERO,
                age:               0.0,
                max_lifetime:      10.0,
                contact_threshold: 16.0,
            },
            Position2D(Vec2::ZERO),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    let wave_b = app
        .world_mut()
        .spawn((
            ResonanceWave {
                speed:             200.0,
                slow_duration:     1.5,
                slow_strength:     0.5,
                target_pos:        Vec2::ZERO,
                age:               0.0,
                max_lifetime:      10.0,
                contact_threshold: 16.0,
            },
            Position2D(Vec2::new(10.0, 0.0)),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();
    let wave_c = app
        .world_mut()
        .spawn((
            ResonanceWave {
                speed:             200.0,
                slow_duration:     1.5,
                slow_strength:     0.5,
                target_pos:        Vec2::ZERO,
                age:               0.0,
                max_lifetime:      10.0,
                contact_threshold: 16.0,
            },
            Position2D(Vec2::new(20.0, 0.0)),
            CleanupOnExit::<NodeState>::default(),
        ))
        .id();

    transition_to_teardown(&mut app);
    app.update();

    for (label, e) in [("A", wave_a), ("B", wave_b), ("C", wave_c)] {
        assert!(
            app.world().get_entity(e).is_err(),
            "Wave {label} should be despawned on Teardown"
        );
    }
}

// ── I2 — tracker.kills drained on teardown ─────────────────────────────

#[test]
fn i2_teardown_drains_tracker_kills() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(OnEnter(NodeState::Teardown), resonance_cleanup_on_teardown);
    app.world_mut().resource_mut::<ResonanceTracker>().kills = vec![
        (0.1, Vec2::new(1.0, 2.0)),
        (0.2, Vec2::new(3.0, 4.0)),
        (0.3, Vec2::new(5.0, 6.0)),
    ];

    transition_to_teardown(&mut app);
    app.update();

    let tracker = app.world().resource::<ResonanceTracker>();
    assert!(
        tracker.kills.is_empty(),
        "tracker.kills should be drained on teardown, got {:?}",
        tracker.kills
    );
}

// ── I3 — active slows reversed; non-resonance entries survive ───────────

#[test]
fn i3_teardown_reverses_active_slows_preserves_non_resonance() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(OnEnter(NodeState::Teardown), resonance_cleanup_on_teardown);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    // Seed two resonance-sourced stack entries plus one non-resonance.
    let multiplier = OrderedFloat(0.5);
    let cfg = SpeedBoostConfig { multiplier };
    cfg.fire(breaker, "hazard:resonance:wave:11", app.world_mut());
    cfg.fire(breaker, "hazard:resonance:wave:22", app.world_mut());
    let chip_cfg = SpeedBoostConfig {
        multiplier: OrderedFloat(1.2),
    };
    chip_cfg.fire(breaker, "chip:overclock", app.world_mut());

    // Seed slows map.
    {
        let mut slows = app.world_mut().resource_mut::<ResonanceActiveSlows>();
        slows.slows.insert(
            "hazard:resonance:wave:11".into(),
            ResonanceSlowEntry {
                remaining: 1.0,
                multiplier,
            },
        );
        slows.slows.insert(
            "hazard:resonance:wave:22".into(),
            ResonanceSlowEntry {
                remaining: 0.5,
                multiplier,
            },
        );
    }

    transition_to_teardown(&mut app);
    app.update();

    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(
        slows.is_empty(),
        "slows map should be drained, keys {:?}",
        slows.keys().collect::<Vec<_>>()
    );

    let resonance_sources = collect_sources_with_prefix(&app, breaker, "hazard:resonance:");
    assert!(
        resonance_sources.is_empty(),
        "Resonance stack entries should be reversed, found {resonance_sources:?}"
    );
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("breaker stack must persist");
    assert_eq!(
        stack.len(),
        1,
        "Only the non-resonance chip entry should remain, len={}",
        stack.len()
    );
    let (src, cfg) = stack.iter().next().unwrap();
    assert_eq!(src.0.as_ref(), "chip:overclock");
    assert_eq!(cfg.multiplier, OrderedFloat(1.2));
}

// ── I4 — ResonanceConfig persists across teardown ─────────────────────

#[test]
fn i4_resonance_config_persists_across_teardown() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(OnEnter(NodeState::Teardown), resonance_cleanup_on_teardown);

    transition_to_teardown(&mut app);
    app.update();

    let cfg = app
        .world()
        .get_resource::<ResonanceConfig>()
        .expect("ResonanceConfig must persist across teardown");
    assert!(
        (cfg.wave_speed - 200.0).abs() < f32::EPSILON,
        "wave_speed should be unchanged at 200.0, got {}",
        cfg.wave_speed
    );
}

// ── I5 — idempotent on empty state ─────────────────────────────────────

#[test]
fn i5_cleanup_is_idempotent_on_empty_state() {
    let mut app = test_app_playing_with_effects();
    app.add_systems(OnEnter(NodeState::Teardown), resonance_cleanup_on_teardown);
    let breaker = spawn_breaker_at(&mut app, Vec2::new(0.0, 50.0));

    transition_to_teardown(&mut app);
    app.update();

    let tracker = app.world().resource::<ResonanceTracker>();
    assert!(tracker.kills.is_empty());
    let slows = &app.world().resource::<ResonanceActiveSlows>().slows;
    assert!(slows.is_empty());
    let stack = app
        .world()
        .get::<EffectStack<SpeedBoostConfig>>(breaker)
        .expect("breaker stack should exist");
    assert!(
        stack.is_empty(),
        "Stack should remain empty (idempotent on empty), got len {}",
        stack.len()
    );
}
