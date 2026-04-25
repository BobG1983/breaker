//! Group E — `burnout_amplify_damage` boost consumption on cell impact
//! (Behaviors E1–E10).
//!
//! Pins the `BoltImpactCell` consumer:
//! - On a primed bolt (`BurnoutDamageBoost`), emits `DamageDealt<Cell>` with
//!   `amount = base * boost.multiplier`, `source =
//!   Some(SourceId::protocol(Burnout).build())`, gated on
//!   `amount > 0.0`.
//! - Removes `BurnoutDamageBoost` unconditionally (single-shot).
//! - Missing `BoltBaseDamage` falls back to `DEFAULT_BOLT_BASE_DAMAGE`.
//! - Non-primed bolts emit nothing.
//! - Same-tick pierce guard: only the first impact emits.
//! - Multi-bolt isolation.
//! - Despawned bolts tolerated.
//! - Harness-safe: early return + reader drain when `BurnoutConfig` absent;
//!   boost NOT consumed.

use bevy::prelude::*;

use super::{
    super::system::{BurnoutDamageBoost, config::BurnoutConfig},
    helpers::{
        build_burnout_app, build_burnout_app_no_config, canonical_burnout_config,
        collected_burnout_damage, install_burnout_config, install_burnout_damage_boost,
        seed_active_protocols_with_burnout, spawn_bolt_with_base_damage, spawn_cell_empty,
        write_bolt_impact_cell,
    },
};
use crate::{mutators::protocols::definition::ProtocolKind, prelude::*};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── E1 — Boosted bolt first impact emits amplified damage ──────────────────-

#[test]
fn boosted_bolt_first_cell_impact_emits_amplified_damage() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1, "expected exactly 1 Burnout damage message");
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
        Some(&SourceId::protocol(ProtocolKind::Burnout).build()),
        "source must be the Burnout builder-produced source"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "BurnoutDamageBoost must be removed after amplification"
    );
}

// ── E1b — Different base/multiplier combination ────────────────────────────-

#[test]
fn amplified_amount_is_base_times_multiplier_variant_values() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 8.0);
    install_burnout_damage_boost(&mut app, bolt, 2.5);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert!(
        (msgs[0].amount - 20.0).abs() < 1e-4,
        "amount expected 20.0 (= 8.0 * 2.5), got {}",
        msgs[0].amount
    );
}

// ── E2 — Boost is single-shot — does NOT persist to second cell impact ─────-

#[test]
fn boost_does_not_persist_to_second_cell_impact() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);

    // Tick 1 — impact cell_a.
    write_bolt_impact_cell(&mut app, cell_a, bolt);
    tick(&mut app);

    let tick1_msgs = collected_burnout_damage(&app);
    assert_eq!(
        tick1_msgs.len(),
        1,
        "tick 1 must emit exactly one amplified message"
    );
    assert_eq!(tick1_msgs[0].target, cell_a);
    assert!((tick1_msgs[0].amount - 40.0).abs() < 1e-4);
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "boost must be removed after tick 1"
    );

    // Tick 2 — impact cell_b.
    write_bolt_impact_cell(&mut app, cell_b, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(
        msgs.len(),
        0,
        "tick 2 must produce zero new amplified messages"
    );
}

// ── E3 — Non-boosted bolt cell impact — no amplified message ───────────────-

#[test]
fn non_boosted_bolt_cell_impact_emits_no_amplified_damage() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0); // NO boost
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "non-boosted bolt must emit zero Burnout damage messages"
    );
}

// ── E4 — Missing BoltBaseDamage falls back to DEFAULT_BOLT_BASE_DAMAGE ─────-

#[test]
fn bolt_without_base_damage_falls_back_to_default_bolt_base_damage() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    // Spawn bolt with BurnoutDamageBoost but NO BoltBaseDamage.
    let bolt = app
        .world_mut()
        .spawn((Bolt, BurnoutDamageBoost { multiplier: 4.0 }))
        .id();
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1);
    // DEFAULT_BOLT_BASE_DAMAGE is 10.0 → 10.0 * 4.0 = 40.0.
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amount expected 40.0 (= DEFAULT_BOLT_BASE_DAMAGE * 4.0), got {}",
        msgs[0].amount
    );
}

// ── E5 — Same-tick pierce guard — only the first impact emits ──────────────-

#[test]
fn same_tick_pierce_guard_only_first_impact_emits() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell_c = spawn_cell_empty(&mut app);
    let cell_d = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_c, bolt);
    write_bolt_impact_cell(&mut app, cell_d, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "pierce guard — only the FIRST impact emits amplified damage"
    );
    assert_eq!(
        msgs[0].target, cell_c,
        "target must be cell_c (first impact)"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "boost must be removed after first impact"
    );
}

// ── E6 — Zero multiplier — emission gated off but boost still consumed ─────-

#[test]
fn zero_multiplier_emits_no_message_but_still_consumes_boost() {
    let mut app = build_burnout_app();
    install_burnout_config(
        &mut app,
        BurnoutConfig {
            full_heat_damage_multiplier: 0.0,
            ..canonical_burnout_config()
        },
    );
    seed_active_protocols_with_burnout(&mut app, 4.0, 2.0, 1.5, 0.0, 2.0);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 0.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "zero amount must emit NO DamageDealt<Cell>"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "boost must be removed unconditionally even when emission gated off"
    );
}

// ── E7 — Multi-bolt isolation — only the impacting bolt's boost consumed ───-

#[test]
fn multi_bolt_isolation_only_impacting_bolt_boost_consumed() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_a, 4.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_b, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt_a);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].dealer, Some(bolt_a));
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt_a).is_none(),
        "bolt_a's boost must be removed"
    );
    let bolt_b_boost = app
        .world()
        .get::<BurnoutDamageBoost>(bolt_b)
        .expect("bolt_b must still carry BurnoutDamageBoost");
    assert!(
        (bolt_b_boost.multiplier - 4.0).abs() < f32::EPSILON,
        "bolt_b's boost must remain untouched (multiplier 4.0), got {}",
        bolt_b_boost.multiplier
    );
}

// ── E8 — Despawned bolt — no panic, no emit ────────────────────────────────-

#[test]
fn impact_for_despawned_bolt_is_tolerated() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    app.world_mut().entity_mut(bolt).despawn();
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app); // must not panic

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no amplified damage for a despawned bolt"
    );
}

// ── E9 — Harness-safe — early return when config absent; boost preserved ───-

#[test]
fn amplify_early_returns_when_config_absent_and_boost_preserved() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app); // tick 1 — config absent

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no amplified damage when BurnoutConfig absent"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_some(),
        "boost must NOT be consumed when config absent (early-return before remove)"
    );
}

// ── E9b — Second quiet tick — buffered message must NOT retro-apply ────────-

#[test]
fn amplify_buffered_message_does_not_retro_apply_on_quiet_tick_without_config() {
    let mut app = build_burnout_app_no_config();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app); // tick 1 — drain reader
    tick(&mut app); // tick 2 — quiet

    assert!(
        collected_burnout_damage(&app).is_empty(),
        "quiet tick must not produce new amplified damage"
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_some(),
        "boost must still be present after quiet tick"
    );
}

// ── E10 — Sentinel stamping — every emit carries the Burnout sentinel ──────-

#[test]
fn every_emitted_message_carries_burnout_sentinel_and_respective_dealer() {
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt_a = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_a, 4.0);
    let bolt_b = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt_b, 4.0);
    let cell_a = spawn_cell_empty(&mut app);
    let cell_b = spawn_cell_empty(&mut app);

    write_bolt_impact_cell(&mut app, cell_a, bolt_a);
    write_bolt_impact_cell(&mut app, cell_b, bolt_b);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 2);
    for msg in &msgs {
        assert_eq!(
            msg.source.as_ref(),
            Some(&SourceId::protocol(ProtocolKind::Burnout).build()),
            "every amplified message must carry the Burnout builder-produced source"
        );
    }
    let msg_a = msgs
        .iter()
        .find(|m| m.target == cell_a)
        .expect("msg for cell_a");
    assert_eq!(
        msg_a.dealer,
        Some(bolt_a),
        "dealer for cell_a msg must be bolt_a"
    );
    let msg_b = msgs
        .iter()
        .find(|m| m.target == cell_b)
        .expect("msg for cell_b");
    assert_eq!(
        msg_b.dealer,
        Some(bolt_b),
        "dealer for cell_b msg must be bolt_b"
    );
}

// ── B26: source matches builder-produced protocol:burnout ──

#[test]
fn amplified_damage_source_equals_builder_protocol_burnout() {
    use crate::{mutators::protocols::definition::ProtocolKind, prelude::SourceIdExt};
    let mut app = build_burnout_app();
    seed_canonical(&mut app);
    let bolt = spawn_bolt_with_base_damage(&mut app, 1.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);
    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    let msgs = collected_burnout_damage(&app);
    assert_eq!(msgs.len(), 1);
    let expected = SourceId::protocol(ProtocolKind::Burnout).build();
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&expected),
        "source must equal SourceId::protocol(Burnout).build()"
    );
}
