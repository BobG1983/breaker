//! Group B — `BurnoutConfig` / `BurnoutHeat` / `BurnoutSpeedBoost` /
//! `BurnoutDamageBoost` derive-set + default-shape tests (Behaviors B1–B4).
//!
//! Pins:
//! - `BurnoutConfig` derive set includes `Resource`, `Debug`, `Clone`,
//!   `Copy`, and `PartialEq` (proven via `==`, `Clone::clone`, and
//!   pass-by-value).
//! - `BurnoutHeat::default()` is `{ heat: 0.0, still_timer: 0.0,
//!   mega_bump_charged: false }` and round-trips through the ECS.
//! - `BurnoutSpeedBoost::default().remaining == 0.0` and non-default
//!   instances insert via `Commands` + round-trip.
//! - `BurnoutDamageBoost::default().multiplier == 0.0` and non-default
//!   instances insert via `Commands` + round-trip.

use bevy::{ecs::world::CommandQueue, prelude::Commands};

use super::super::system::{BurnoutConfig, BurnoutDamageBoost, BurnoutHeat, BurnoutSpeedBoost};
use crate::prelude::*;

// ── B1 — BurnoutConfig is Copy + Clone + PartialEq ─────────────────────────-

#[test]
fn burnout_config_is_copy_clone_partial_eq() {
    fn takes_by_value(cfg: BurnoutConfig) -> f32 {
        cfg.fill_duration
    }
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = BurnoutConfig {
        fill_duration:               4.0,
        drain_duration:              2.0,
        still_threshold:             1.5,
        full_heat_damage_multiplier: 4.0,
        speed_boost_duration:        2.0,
    };

    // PartialEq: two identical values compare equal.
    let same = BurnoutConfig {
        fill_duration:               4.0,
        drain_duration:              2.0,
        still_threshold:             1.5,
        full_heat_damage_multiplier: 4.0,
        speed_boost_duration:        2.0,
    };
    assert_eq!(orig, same, "identical configs must compare equal");

    // Copy: two pass-by-value assignments both succeed.
    let copy1 = orig;
    let copy2 = orig;
    assert_eq!(orig, copy1, "copy must equal original");
    assert_eq!(copy1, copy2, "both copies must equal each other");

    // Copy: pass-by-value into a function compiles and original remains usable.
    let returned = takes_by_value(orig);
    assert!(
        (returned - 4.0).abs() < f32::EPSILON,
        "takes_by_value returned {returned}, expected 4.0"
    );
    assert!(
        (orig.fill_duration - 4.0).abs() < f32::EPSILON,
        "original remains usable after pass-by-value (Copy, not move)"
    );

    // Clone: routed through a generic that requires `T: Clone`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");
}

// ── B2 — BurnoutHeat::default() shape ───────────────────────────────────────

#[test]
fn burnout_heat_default_has_zero_heat_zero_timer_unset_charge() {
    let h = BurnoutHeat::default();
    assert!(
        (h.heat - 0.0).abs() < f32::EPSILON,
        "BurnoutHeat::default().heat expected 0.0, got {}",
        h.heat
    );
    assert!(
        (h.still_timer - 0.0).abs() < f32::EPSILON,
        "BurnoutHeat::default().still_timer expected 0.0, got {}",
        h.still_timer
    );
    assert!(
        !h.mega_bump_charged,
        "BurnoutHeat::default().mega_bump_charged expected false, got {}",
        h.mega_bump_charged
    );
}

// ── B2b — BurnoutHeat round-trips through the ECS via Commands ─────────────-

#[test]
fn burnout_heat_default_round_trips_through_ecs() {
    let mut app = TestAppBuilder::new().build();
    let breaker = app.world_mut().spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands.entity(breaker).insert(BurnoutHeat::default());
    }
    queue.apply(app.world_mut());

    let h = app
        .world()
        .get::<BurnoutHeat>(breaker)
        .expect("BurnoutHeat should be attached to the breaker after insert");
    assert_eq!(
        *h,
        BurnoutHeat::default(),
        "round-tripped BurnoutHeat must match default"
    );
}

// ── B3 — BurnoutSpeedBoost::default().remaining == 0.0 ─────────────────────-

#[test]
fn burnout_speed_boost_default_has_zero_remaining() {
    let boost = BurnoutSpeedBoost::default();
    assert!(
        (boost.remaining - 0.0).abs() < f32::EPSILON,
        "BurnoutSpeedBoost::default().remaining expected 0.0, got {}",
        boost.remaining
    );
}

// ── B3b — non-default BurnoutSpeedBoost round-trips via Commands ───────────-

#[test]
fn burnout_speed_boost_non_default_inserts_via_commands_and_queryable() {
    let mut app = TestAppBuilder::new().build();
    let breaker = app.world_mut().spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands
            .entity(breaker)
            .insert(BurnoutSpeedBoost { remaining: 2.0 });
    }
    queue.apply(app.world_mut());

    let boost = app
        .world()
        .get::<BurnoutSpeedBoost>(breaker)
        .expect("BurnoutSpeedBoost should be attached to the breaker after insert");
    assert!(
        (boost.remaining - 2.0).abs() < f32::EPSILON,
        "BurnoutSpeedBoost.remaining expected 2.0 after insert, got {}",
        boost.remaining
    );
}

// ── B4 — BurnoutDamageBoost::default().multiplier == 0.0 ───────────────────-

#[test]
fn burnout_damage_boost_default_has_zero_multiplier() {
    let boost = BurnoutDamageBoost::default();
    assert!(
        (boost.multiplier - 0.0).abs() < f32::EPSILON,
        "BurnoutDamageBoost::default().multiplier expected 0.0, got {}",
        boost.multiplier
    );
}

// ── B4b — non-default BurnoutDamageBoost round-trips via Commands ──────────-

#[test]
fn burnout_damage_boost_non_default_inserts_via_commands_and_queryable() {
    let mut app = TestAppBuilder::new().build();
    let bolt = app.world_mut().spawn_empty().id();

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands
            .entity(bolt)
            .insert(BurnoutDamageBoost { multiplier: 4.0 });
    }
    queue.apply(app.world_mut());

    let boost = app
        .world()
        .get::<BurnoutDamageBoost>(bolt)
        .expect("BurnoutDamageBoost should be attached to the bolt after insert");
    assert!(
        (boost.multiplier - 4.0).abs() < f32::EPSILON,
        "BurnoutDamageBoost.multiplier expected 4.0 after insert, got {}",
        boost.multiplier
    );
}
