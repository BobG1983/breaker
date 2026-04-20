//! Group B — `RecklessDashConfig` + `RiskyDamageBoost` component shape
//! (Behaviors 5–6).
//!
//! Pins:
//! - `RecklessDashConfig` derive set includes `Resource`, `Debug`, `Clone`,
//!   `Copy`, and `PartialEq` (proven via `==`, `Clone::clone`, and
//!   pass-by-value).
//! - `RiskyDamageBoost::default()` has `multiplier: 0.0`, and non-default
//!   instances insert via `Commands` and round-trip through queries.

use bevy::{ecs::world::CommandQueue, prelude::Commands};

use super::super::system::{RecklessDashConfig, RiskyDamageBoost};
use crate::prelude::*;

// ── Behavior 5 — RecklessDashConfig is Copy + Clone + PartialEq ────────────-

#[test]
fn reckless_dash_config_is_copy_clone_partial_eq() {
    fn takes_by_value(cfg: RecklessDashConfig) -> f32 {
        cfg.damage_multiplier
    }
    fn require_clone<T: Clone>(value: &T) -> T {
        value.clone()
    }

    let orig = RecklessDashConfig {
        risky_zone_start:  0.7,
        damage_multiplier: 4.0,
        double_penalty:    true,
    };

    // PartialEq: two identical values compare equal.
    let same = RecklessDashConfig {
        risky_zone_start:  0.7,
        damage_multiplier: 4.0,
        double_penalty:    true,
    };
    assert_eq!(orig, same, "identical configs must compare equal");

    // Copy: two pass-by-value assignments both succeed (original not moved).
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
        (orig.damage_multiplier - 4.0).abs() < f32::EPSILON,
        "original remains usable after pass-by-value (Copy, not move)"
    );

    // Clone: routed through a generic that requires `T: Clone` — proves the
    // derive exists at compile time without tripping `clippy::clone_on_copy`.
    let cloned = require_clone(&orig);
    assert_eq!(orig, cloned, "Clone-returned copy must equal original");
}

// ── Behavior 6 — RiskyDamageBoost::default() has multiplier 0.0 ────────────-

#[test]
fn risky_damage_boost_default_has_zero_multiplier() {
    let boost = RiskyDamageBoost::default();
    assert!(
        (boost.multiplier - 0.0).abs() < f32::EPSILON,
        "RiskyDamageBoost::default().multiplier expected 0.0, got {}",
        boost.multiplier
    );
}

// ── Behavior 6 (edge case) — RiskyDamageBoost inserts via Commands ─────────-

#[test]
fn risky_damage_boost_non_default_inserts_via_commands_and_queryable() {
    let mut app = TestAppBuilder::new().build();
    let bolt = app.world_mut().spawn_empty().id();

    // Insert via Commands (the production system's only insertion path).
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        commands
            .entity(bolt)
            .insert(RiskyDamageBoost { multiplier: 4.0 });
    }
    queue.apply(app.world_mut());

    let boost = app
        .world()
        .get::<RiskyDamageBoost>(bolt)
        .expect("RiskyDamageBoost should be attached to the bolt after insert");
    assert!(
        (boost.multiplier - 4.0).abs() < f32::EPSILON,
        "RiskyDamageBoost.multiplier expected 4.0 after insert, got {}",
        boost.multiplier
    );
}
