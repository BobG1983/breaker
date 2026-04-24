//! Group D — `reckless_dash_amplify_damage` boost consumption on cell impact
//! (Behaviors 21–31).
//!
//! Pins the `BoltImpactCell` consumer:
//! - Reads `BoltImpactCell`. On a primed bolt, emits
//!   `DamageDealt<Cell>` with `amount = base * boost.multiplier` and
//!   `source = Some(RECKLESS_DASH_SENTINEL.into())` when `amount > 0.0`.
//! - Removes `RiskyDamageBoost` from the bolt (unconditional, single-shot).
//! - Emission is gated on `amount > 0.0` (mirrors Iron Curtain / Echo
//!   Strike).
//! - Missing `BoltBaseDamage` falls back to `DEFAULT_BOLT_BASE_DAMAGE`.
//! - Non-primed bolts are no-ops.
//! - Same-tick pierce guard: only the first impact emits amplified damage.
//! - Despawned bolts tolerated.
//! - Multi-bolt isolation.
//! - Harness-safe: early return + reader drain when `RecklessDashConfig`
//!   absent.

use bevy::prelude::*;

use super::{
    super::system::{RecklessDashConfig, RiskyDamageBoost},
    helpers::{
        build_reckless_dash_app, build_reckless_dash_app_no_config, collected_reckless_dash_damage,
        install_reckless_dash_config, seed_active_protocols_with_reckless_dash,
        spawn_bolt_with_base_damage, spawn_cell_empty, write_bolt_impact_cell,
    },
};
use crate::prelude::*;

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_reckless_dash(app, 0.7, 4.0, true);
}

/// Installs `RiskyDamageBoost { multiplier }` directly on the given bolt via
/// a world-level insert (bypasses the production on-bump path).
fn install_risky_boost(app: &mut App, bolt: Entity, multiplier: f32) {
    app.world_mut()
        .entity_mut(bolt)
        .insert(RiskyDamageBoost { multiplier });
}

// ── Behavior 21 — primed bolt first impact emits amplified damage ──────────-

#[test]
fn primed_bolt_first_cell_impact_emits_amplified_damage() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "expected exactly 1 Reckless Dash damage message"
    );
    let msg = &msgs[0];
    assert_eq!(msg.dealer, Some(bolt), "dealer must be the impacting bolt");
    assert_eq!(msg.target, cell, "target must be the impacted cell");
    assert!(
        (msg.amount - 40.0).abs() < 1e-4,
        "amount expected 40.0 (= 10.0 * 4.0), got {}",
        msg.amount
    );
    assert_eq!(
        msg.source.as_ref(),
        Some(&SourceId::from("protocol:reckless_dash")),
        "source must be the Reckless Dash sentinel"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_none(),
        "RiskyDamageBoost must be removed after amplification"
    );
}

// ── Behavior 21 (edge case) — different base/multiplier combination ────────-

#[test]
fn primed_bolt_first_cell_impact_amount_is_base_times_multiplier() {
    let mut app = build_reckless_dash_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 2.5,
            double_penalty:    true,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 2.5, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 8.0);
    install_risky_boost(&mut app, bolt, 2.5);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 20.0).abs() < 1e-4,
        "amount expected 20.0 (= 8.0 * 2.5), got {}",
        msgs[0].amount
    );
}

// ── Behavior 22 — boost is single-shot, does NOT persist to 2nd cell ───────-

#[test]
fn boost_does_not_persist_to_second_cell_impact() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);

    // Tick 1: impact cell_a — must emit exactly one amplified damage message
    // targeting cell_a with amount 10.0 * 4.0 = 40.0.
    write_bolt_impact_cell(&mut app, cell_a, bolt);
    tick(&mut app);

    let tick1_msgs = collected_reckless_dash_damage(&app);
    assert_eq!(
        tick1_msgs.len(),
        1,
        "tick 1 must emit exactly one Reckless Dash damage message"
    );
    assert_eq!(
        tick1_msgs[0].target, cell_a,
        "tick 1 message must target cell_a"
    );
    assert!(
        (tick1_msgs[0].amount - 40.0).abs() < 1e-4,
        "tick 1 amount expected 40.0 (= 10.0 * 4.0), got {}",
        tick1_msgs[0].amount
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_none(),
        "RiskyDamageBoost must be removed after tick 1"
    );

    // Tick 2: impact cell_b — should emit NO Reckless Dash damage.
    write_bolt_impact_cell(&mut app, cell_b, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(
        msgs.len(),
        0,
        "after tick 2, the captured Reckless Dash damage messages must be empty — \
         MessageCollector clears each frame and no new amplification should fire"
    );
}

// ── Behavior 23 — non-primed bolt cell impact is a no-op ───────────────────-

#[test]
fn non_primed_bolt_cell_impact_emits_no_amplified_damage() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0); // NO RiskyDamageBoost
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "non-primed bolt must emit zero Reckless Dash damage messages"
    );
}

// ── Behavior 24 — missing BoltBaseDamage falls back to DEFAULT ─────────────-

#[test]
fn bolt_without_base_damage_falls_back_to_default_bolt_base_damage() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    // Spawn a bolt with RiskyDamageBoost but NO BoltBaseDamage.
    let bolt = app
        .world_mut()
        .spawn((Bolt, RiskyDamageBoost { multiplier: 4.0 }))
        .id();
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1);
    // DEFAULT_BOLT_BASE_DAMAGE is 10.0 → 10.0 * 4.0 = 40.0.
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amount expected 40.0 (= DEFAULT_BOLT_BASE_DAMAGE * 4.0), got {}",
        msgs[0].amount
    );
}

// ── Behavior 25 — boost persists even if dash ended before impact ──────────-

#[test]
fn boost_persists_even_after_dash_ends_before_cell_impact() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    // Boost lives on the bolt, not the breaker — no breaker state manipulation
    // is required. Simulate "dash ended between bump and cell impact" by
    // having the boost already on the bolt with NO breaker to inspect.
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1, "amplified message must still emit");
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amount expected 40.0 after the dash ends — boost persists on the bolt, got {}",
        msgs[0].amount
    );
}

// ── Behavior 26 — multi-bolt isolation: only the impacting bolt's boost is
// consumed ──────────────────────────────────────────────────────────────────

#[test]
fn multi_bolt_isolation_only_impacting_bolt_boost_consumed() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt_a, 4.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt_b, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt_a);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 1, "exactly one amplified message expected");
    assert_eq!(msgs[0].dealer, Some(bolt_a), "dealer must be bolt_a");
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt_a).is_none(),
        "bolt_a's RiskyDamageBoost must be removed"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt_b).is_some(),
        "bolt_b's RiskyDamageBoost must remain untouched"
    );
}

// ── Behavior 27 — impact for despawned bolt is tolerated ───────────────────-

#[test]
fn impact_for_despawned_bolt_is_tolerated() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    app.world_mut().entity_mut(bolt).despawn();
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage for a despawned bolt"
    );
}

// ── Behavior 28 — same-tick pierce guard ───────────────────────────────────-

#[test]
fn same_tick_pierce_guard_only_first_impact_emits() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    let cell_c = spawn_cell_empty(&mut app);
    let cell_d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_c, bolt);
    write_bolt_impact_cell(&mut app, cell_d, bolt);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "pierce guard: only the FIRST impact emits amplified damage"
    );
    assert_eq!(
        msgs[0].target, cell_c,
        "target must be cell_c (first impact)"
    );
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amount expected 40.0 (= 10.0 * 4.0), got {}",
        msgs[0].amount
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_none(),
        "RiskyDamageBoost must be removed after first impact"
    );
}

// ── Behavior 29 — zero damage_multiplier emits no message, still consumes ──-

#[test]
fn zero_damage_multiplier_emits_no_message_but_still_consumes_boost() {
    let mut app = build_reckless_dash_app();
    install_reckless_dash_config(
        &mut app,
        RecklessDashConfig {
            risky_zone_start:  0.7,
            damage_multiplier: 0.0,
            double_penalty:    true,
        },
    );
    seed_active_protocols_with_reckless_dash(&mut app, 0.7, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 0.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "zero amount must emit NO DamageDealt<Cell> (amount > 0.0 gate)"
    );
    // Unconditional consumption: boost must still be removed.
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_none(),
        "RiskyDamageBoost must be removed unconditionally, even when emission is gated off"
    );
}

// ── Behavior 30 — harness-safe: amplify early-returns when config absent ───-

#[test]
fn amplify_early_returns_and_clears_reader_when_config_absent() {
    let mut app = build_reckless_dash_app_no_config();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app); // tick 1 — config absent

    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "no amplified damage when config absent"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_some(),
        "boost NOT consumed when config absent (early return before removal)"
    );

    // Second quiet tick — buffered message must NOT retro-apply.
    tick(&mut app);
    assert!(
        collected_reckless_dash_damage(&app).is_empty(),
        "buffered BoltImpactCell must not retro-apply on quiet tick 2"
    );
    assert!(
        app.world().get::<RiskyDamageBoost>(bolt).is_some(),
        "boost still present after quiet tick 2"
    );
}

// ── Behavior 31 — every emitted message carries the sentinel + dealer ─────-

#[test]
fn every_emitted_message_has_sentinel_and_respective_dealer() {
    let mut app = build_reckless_dash_app();
    seed_canonical(&mut app);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt_a, 4.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_risky_boost(&mut app, bolt_b, 4.0);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_a, bolt_a);
    write_bolt_impact_cell(&mut app, cell_b, bolt_b);
    tick(&mut app);

    let msgs = collected_reckless_dash_damage(&app);
    assert_eq!(msgs.len(), 2, "two independent amplified messages expected");

    for msg in &msgs {
        assert_eq!(
            msg.source.as_ref(),
            Some(&SourceId::from("protocol:reckless_dash")),
            "every amplified message must carry the Reckless Dash sentinel"
        );
    }

    // Per-bolt dealer assertion.
    let amt_a = msgs
        .iter()
        .find(|m| m.target == cell_a)
        .expect("msg for cell_a");
    assert_eq!(
        amt_a.dealer,
        Some(bolt_a),
        "dealer for cell_a msg must be bolt_a"
    );
    let amt_b = msgs
        .iter()
        .find(|m| m.target == cell_b)
        .expect("msg for cell_b");
    assert_eq!(
        amt_b.dealer,
        Some(bolt_b),
        "dealer for cell_b msg must be bolt_b"
    );
}
