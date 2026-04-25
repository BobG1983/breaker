//! Group C — `register` schedule wiring (Behaviors 19–25).
//!
//! Pins that `iron_curtain_on_bolt_lost` is wired into `FixedUpdate`, gated
//! by `protocol_active(IronCurtain)` + `in_state(NodeState::Playing)`, ordered
//! `.after(BoltSystems::BoltLost)`, and harness-safe against a missing
//! `IronCurtainConfig` or `PlayfieldConfig`.

use bevy::prelude::*;

use super::{
    super::system::IronCurtainConfig,
    helpers::{
        build_iron_curtain_app, build_iron_curtain_app_in_chip_selecting,
        build_iron_curtain_app_no_config, build_iron_curtain_app_no_playfield,
        canonical_iron_curtain_config, collected_iron_curtain_damage, install_iron_curtain_config,
        seed_active_protocols_with_iron_curtain, spawn_bolt_with_base_damage, spawn_breaker_at,
        spawn_cell_at, write_bolt_lost,
    },
};
use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
};

// ── Behavior 19 — active + Playing fires the wave ──────────────────────────-

#[test]
fn register_wires_iron_curtain_on_bolt_lost_under_active_and_playing() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "expected exactly 1 wave message, got {}",
        msgs.len()
    );
    assert!(
        (msgs[0].amount - 10.0).abs() < 1e-4,
        "amount expected 10.0, got {}",
        msgs[0].amount
    );
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&SourceId::protocol(ProtocolKind::IronCurtain).build())
    );
}

// ── Behavior 20 — gated off when Iron Curtain NOT active ───────────────────-

#[test]
fn system_gated_off_when_iron_curtain_not_active() {
    let mut app = build_iron_curtain_app();
    // Do NOT seed ActiveProtocols.
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no Iron Curtain in ActiveProtocols → no wave"
    );
}

// ── Behavior 20 (edge case) — different protocol active ────────────────────-

#[test]
fn different_protocol_active_does_not_fire_iron_curtain_wave() {
    let mut app = build_iron_curtain_app();
    // Seed ActiveProtocols with a DIFFERENT protocol (DebtCollector).
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Debt Collector".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::DebtCollector {
                stack_per_bump: 0.5,
            },
        });
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "Iron Curtain wave must not fire when only another protocol is active"
    );
}

// ── Behavior 21 — gated off when NodeState != Playing ──────────────────────-

#[test]
fn system_gated_off_when_node_state_not_playing() {
    let mut app = build_iron_curtain_app_in_chip_selecting();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "system must not run when NodeState != Playing"
    );
}

// ── Behavior 22 — missing IronCurtainConfig → harness-safe, no panic ───────-

#[test]
fn missing_iron_curtain_config_is_harness_safe_and_does_not_emit() {
    let mut app = build_iron_curtain_app_no_config();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);
    tick(&mut app);
    tick(&mut app);

    assert!(
        app.world().get_resource::<IronCurtainConfig>().is_none(),
        "IronCurtainConfig must remain absent"
    );
    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no wave messages when config absent"
    );
}

// ── Behavior 22 (edge case) — adding config LATER must not resurface message

#[test]
fn missing_config_clears_reader_prior_messages_do_not_leak_when_config_added_later() {
    let mut app = build_iron_curtain_app_no_config();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    // Step 1: write BoltLost without config present.
    write_bolt_lost(&mut app, bolt);
    tick(&mut app);
    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no wave on tick-1 (no config)"
    );

    // Step 2: install config; NO further BoltLost write. The original
    // message must NOT re-trigger a wave.
    install_iron_curtain_config(&mut app, canonical_iron_curtain_config());
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "prior BoltLost message must NOT leak into a wave after config added later"
    );
}

// ── Behavior 23 — missing PlayfieldConfig → harness-safe, no panic ─────────-

#[test]
fn missing_playfield_config_is_harness_safe_and_does_not_emit() {
    let mut app = build_iron_curtain_app_no_playfield();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "no wave messages when PlayfieldConfig absent"
    );

    // Subsequent quiet tick must remain empty (no leakage).
    tick(&mut app);
    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "subsequent tick must not resurface the earlier BoltLost"
    );
}

// ── Behavior 24 — schedule ticks cleanly with no messages / entities ───────-

#[test]
fn schedule_ticks_cleanly_with_no_messages_no_bolts_no_cells() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "quiet schedule must not produce wave messages"
    );
}

// ── Behavior 25 — BoltLost same-tick consumption by iron_curtain_on_bolt_lost

#[test]
fn register_wires_iron_curtain_on_bolt_lost_to_consume_bolt_lost_same_tick() {
    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    // Writing then ticking once — MessageCollector clears at First and
    // collects at Last, so anything observed here came from this frame.
    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "BoltLost written in the same tick must be consumed and produce a wave"
    );
    assert!(
        (msgs[0].amount - 10.0).abs() < 1e-4,
        "amount expected 10.0 from same-tick consumption, got {}",
        msgs[0].amount
    );
}

// ── Pregate pin — protocol-inactive BoltLost drains cleanly ────────────────-
//
// Regression pin against accidental `.run_if` reintroduction on the reader
// system: `iron_curtain_on_bolt_lost` now enforces its gate in-body via
// `reader.clear()` so pre-gate `BoltLost` messages drain cleanly instead of
// replaying on gate open.

#[test]
fn pregate_bolt_lost_drains_cleanly_before_iron_curtain_activates() {
    let mut app = build_iron_curtain_app();
    // Gate closed: Iron Curtain NOT yet in ActiveProtocols.
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    // Tick 1 — write BoltLost with gate closed. Retrofit drains reader. No wave fires.
    write_bolt_lost(&mut app, bolt);
    tick(&mut app);
    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "gate closed → no wave on tick 1"
    );

    // Tick 2 — open the gate, no new message. The pre-gate message was
    // drained on tick 1; nothing remains to replay → no wave.
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "pre-gate BoltLost was drained on tick 1 — no wave may fire \
         post-activation"
    );
}

// Note: state-gate pregate accumulation (NodeState != Playing → transition
// to Playing) is tested implicitly: `system_gated_off_when_node_state_not_playing`
// covers the gate-closed case; the protocol_active pregate test above
// covers the retained-reader systemic behavior.

// ── damage_fraction = 0.0 edge — wave emits nothing even inside falloff_start

#[test]
fn damage_fraction_zero_emits_no_wave_even_for_close_cells() {
    let mut app = build_iron_curtain_app();
    install_iron_curtain_config(
        &mut app,
        IronCurtainConfig {
            damage_fraction: 0.0,
            falloff_start:   50.0,
        },
    );
    seed_active_protocols_with_iron_curtain(&mut app, 0.0, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0)); // distance 20.0, within falloff_start
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    write_bolt_lost(&mut app, bolt);
    tick(&mut app);

    assert!(
        collected_iron_curtain_damage(&app).is_empty(),
        "damage_fraction = 0.0 → wave_origin_damage = 0.0 → damage > 0.0 gate skips emission"
    );
}

// ── Adversarial ordering — .after(BoltSystems::BoltLost) must process same-frame

#[test]
fn bolt_lost_system_set_message_is_consumed_same_tick_not_deferred() {
    // This test proves the `.after(BoltSystems::BoltLost)` ordering: a
    // BoltLost message written inside a system that runs IN BoltSystems::BoltLost
    // during the FixedUpdate pass is visible to iron_curtain_on_bolt_lost on
    // the SAME tick, not deferred to the next. If the ordering were
    // `.before(BoltSystems::BoltLost)`, the system would miss the message
    // until the next tick (retained-reader semantics) — still visible
    // eventually but one tick late.
    use crate::bolt::sets::BoltSystems;

    let mut app = build_iron_curtain_app();
    seed_active_protocols_with_iron_curtain(&mut app, 0.5, 50.0);
    spawn_breaker_at(&mut app, Vec2::new(0.0, -200.0));
    spawn_cell_at(&mut app, Vec2::new(0.0, -180.0));
    let bolt = spawn_bolt_with_base_damage(&mut app, 20.0);

    // Inject a ghost producer that writes BoltLost within BoltSystems::BoltLost.
    app.add_systems(
        FixedUpdate,
        (move |mut w: MessageWriter<BoltLost>, mut done: Local<bool>| {
            if *done {
                return;
            }
            w.write(BoltLost {
                bolt,
                breaker: Entity::PLACEHOLDER,
            });
            *done = true;
        })
        .in_set(BoltSystems::BoltLost)
        .run_if(in_state(NodeState::Playing)),
    );

    tick(&mut app);

    let msgs = collected_iron_curtain_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "BoltLost written inside BoltSystems::BoltLost must be visible to \
         iron_curtain_on_bolt_lost on the same FixedUpdate tick via .after ordering"
    );
    assert!((msgs[0].amount - 10.0).abs() < 1e-4);
}
