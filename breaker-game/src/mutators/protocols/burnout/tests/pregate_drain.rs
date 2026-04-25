//! Pre-gate drain retrofit regression tests — `burnout_on_bump` and
//! `burnout_amplify_damage`.
//!
//! These tests pin the observable outcome of the in-body
//! `ActiveProtocols`/`NodeState` gate + `reader.clear()` retrofit applied
//! to Burnout's two `MessageReader` systems. They FAIL on the current
//! main branch (the retrofit does not exist yet) and PASS once writer-code
//! lands the gate check in `burnout/system.rs`.
//!
//! Parallels the harness-safe config-absent tests D8/D8b (`on_bump.rs`)
//! and E9/E9b (`amplify.rs`) — the shape is the same
//! (write-tick-tick-assert), but the gate being toggled mid-sequence is
//! `ActiveProtocols` rather than `BurnoutConfig`.

use bevy::prelude::*;

use super::{
    super::system::BurnoutDamageBoost,
    helpers::{
        build_burnout_app, collected_burnout_damage, count_shockwave_sources,
        install_burnout_damage_boost, read_damage_boost_multiplier, read_heat,
        seed_active_protocols_with_burnout, set_heat_state, spawn_bolt_with_base_damage,
        spawn_breaker_stationary, spawn_cell_empty, write_bolt_impact_cell, write_bump_performed,
    },
};
use crate::{
    breaker::messages::BumpGrade, mutators::protocols::definition::ProtocolKind, prelude::*,
};

fn seed_canonical(app: &mut App) {
    seed_active_protocols_with_burnout(app, 4.0, 2.0, 1.5, 4.0, 2.0);
}

// ── Behavior 1 — `burnout_on_bump` pre-gate retrofit ─────────────────────-

#[test]
fn burnout_on_bump_buffered_while_gate_off_is_not_retro_processed() {
    // Given: Burnout app built with canonical config, but ActiveProtocols
    // is EMPTY — `protocol_active(Burnout)` gate is OFF.
    let mut app = build_burnout_app();
    let breaker = spawn_breaker_stationary(&mut app, Vec2::new(0.0, -400.0));
    set_heat_state(&mut app, breaker, 1.0, 0.0, true);
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);

    // Write a BumpPerformed while the gate is OFF. Bevy's two-frame
    // double buffer keeps this message live across the gate flip, so
    // without the in-body drain the buffered message would be consumed
    // retroactively on frame N+1.
    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);

    // When: frame N — gate-off tick. The retrofit must `reader.clear()`.
    tick(&mut app);

    // Seed Burnout into ActiveProtocols — gate flips ON.
    seed_canonical(&mut app);

    // Frame N+1 — gate-on tick. Without the retrofit, the stale
    // BumpPerformed would fire `burnout_on_bump` here.
    tick(&mut app);

    // Then: the bolt MUST NOT carry BurnoutDamageBoost.
    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        None,
        "bolt must NOT carry BurnoutDamageBoost — buffered BumpPerformed \
         must NOT be retroactively consumed after gate opens",
    );
    // And no shockwave entity was spawned.
    assert_eq!(
        count_shockwave_sources(&mut app),
        0,
        "no shockwave entity may be spawned from a buffered BumpPerformed",
    );
    // Heat state: `mega_bump_charged` is the primary retroactive-consume
    // guard. If the buffered BumpPerformed had been retroactively
    // consumed, `burnout_on_bump` would clear the charge to `false`.
    let h = read_heat(&app, breaker).expect("BurnoutHeat should be present");
    assert!(
        h.mega_bump_charged,
        "mega_bump_charged must remain true — primary retroactive-consume \
         guard; the buffered BumpPerformed was NOT consumed",
    );
    // `burnout_update_heat` runs in FixedUpdate whenever the Burnout gate
    // is ON; on frame N+1 it drains heat below 1.0. This is expected
    // gate-on behavior, not a retroactive consume.
    assert!(
        h.heat < 1.0,
        "heat should have drained slightly on the gate-on tick (expected \
         behavior from burnout_update_heat, orthogonal to the retrofit) \
         — got {}",
        h.heat,
    );
    // Likewise, the breaker is stationary so `still_timer` advances on
    // frame N+1.
    assert!(
        h.still_timer > 0.0,
        "still_timer should be > 0 after one gate-on tick with a \
         stationary breaker — got {}",
        h.still_timer,
    );

    // Edge case: after the two-tick sequence above, write a FRESH
    // BumpPerformed and tick again. The retrofit MUST NOT break the
    // production consume path once the gate is open AND a fresh message
    // is written. Guards against a bad fix that clears the reader on
    // every invocation regardless of gate state.
    write_bump_performed(&mut app, breaker, Some(bolt), BumpGrade::Perfect);
    tick(&mut app);

    // `mega_bump_charged` is cleared ONLY by `burnout_on_bump` (the
    // consume path) or by `burnout_update_heat`'s still-threshold fire
    // path (when `still_timer` crosses `config.still_threshold = 1.5s`).
    // Two gate-on ticks at dt ≈ 0.0156s are nowhere near that threshold,
    // and `burnout_on_bump` drained the buffered message on tick N+1
    // without consuming the charge — so `mega_bump_charged` is still
    // true when the fresh BumpPerformed is processed on tick N+2. The
    // boost + shockwave assertions below are load-bearing.
    assert_eq!(
        read_damage_boost_multiplier(&app, bolt),
        Some(4.0),
        "fresh BumpPerformed on an already-charged breaker after gate is \
         open MUST produce a 4.0x BurnoutDamageBoost — the retrofit must \
         NOT disable the production consume path",
    );
    assert_eq!(
        count_shockwave_sources(&mut app),
        1,
        "fresh BumpPerformed must still spawn exactly ONE shockwave \
         entity once the gate is open",
    );
}

// ── Behavior 2 — `burnout_amplify_damage` pre-gate retrofit ──────────────-

#[test]
fn burnout_amplify_damage_buffered_while_gate_off_is_not_retro_processed() {
    // Given: Burnout app built with canonical config, but ActiveProtocols
    // is EMPTY — `protocol_active(Burnout)` gate is OFF.
    let mut app = build_burnout_app();
    let bolt = spawn_bolt_with_base_damage(&mut app, 10.0);
    install_burnout_damage_boost(&mut app, bolt, 4.0);
    let cell = spawn_cell_empty(&mut app);

    // Write a BoltImpactCell while the gate is OFF.
    write_bolt_impact_cell(&mut app, cell, bolt);

    // When: frame N — gate-off tick.
    tick(&mut app);

    // Seed Burnout — gate flips ON.
    seed_canonical(&mut app);

    // Frame N+1 — gate-on tick. Without the retrofit, the stale
    // BoltImpactCell would be processed here and emit a 40.0 DamageDealt.
    tick(&mut app);

    // Then: NO DamageDealt<Cell> carrying `source_chip == Some("protocol:burnout")`
    // was emitted across either frame. Note: `collected_burnout_damage`
    // filters on the Burnout sentinel, so this count is total across all
    // ticks so far.
    assert!(
        collected_burnout_damage(&app).is_empty(),
        "no DamageDealt<Cell> with Burnout sentinel may be emitted from \
         a buffered BoltImpactCell — got {:?}",
        collected_burnout_damage(&app),
    );
    // Fail-closed: the boost MUST NOT be removed (matches the
    // config-absent behavior in `amplify_early_returns_when_config_absent_and_boost_preserved`).
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_some(),
        "BurnoutDamageBoost must remain on the bolt — the retrofit must \
         NOT consume the boost on a buffered BoltImpactCell",
    );
    assert_eq!(
        app.world()
            .get::<BurnoutDamageBoost>(bolt)
            .map(|b| b.multiplier),
        Some(4.0),
        "BurnoutDamageBoost's multiplier must be unchanged",
    );

    // Edge case: after the two-tick sequence above, write a FRESH
    // BoltImpactCell and tick again.
    write_bolt_impact_cell(&mut app, cell, bolt);
    tick(&mut app);

    // Total count across all three ticks: 0 (gate-off frame N) + 0
    // (gate-on frame N+1 — buffered message drained by the retrofit) + 1
    // (gate-on frame N+2 — fresh message consumed) = 1.
    let msgs = collected_burnout_damage(&app);
    assert_eq!(
        msgs.len(),
        1,
        "exactly ONE Burnout DamageDealt<Cell> must be emitted — from the \
         fresh BoltImpactCell on frame N+2, not from the buffered one \
         drained by the retrofit",
    );
    assert!(
        (msgs[0].amount - 40.0).abs() < 1e-4,
        "amplified amount must be base * multiplier = 10.0 * 4.0 = 40.0 \
         within 1e-4 tolerance — got {}",
        msgs[0].amount,
    );
    assert_eq!(
        msgs[0].source.as_ref(),
        Some(&SourceId::protocol(ProtocolKind::Burnout).build()),
        "emitted DamageDealt must carry the builder-produced Burnout source",
    );
    assert!(
        app.world().get::<BurnoutDamageBoost>(bolt).is_none(),
        "BurnoutDamageBoost must be consumed after the fresh \
         BoltImpactCell amplifies damage — single-shot semantics",
    );
}
