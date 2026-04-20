//! Group H — harness-safety (Behaviors 30–31).
//!
//! Pins that when `ConductorConfig` is absent:
//! - The swap system drains buffered `BumpPerformed` messages and does NOT
//!   swap.
//! - The system is "live" after the config is inserted — subsequent fresh
//!   messages fire a swap.
//! - `register` alone (no config, no active protocol, no bolts) ticks
//!   cleanly without panics or side-effects.

use super::{
    super::system::ConductorConfig,
    helpers::{
        bound_fingerprints, build_conductor_app_no_config, canonical_conductor_config, has_extra,
        has_primary, make_distinct_bound, seed_active_protocols_with_conductor,
        spawn_dummy_breaker, spawn_extra_bolt_with_bound, spawn_primary_bolt_with_bound,
        write_bump_performed,
    },
};
use crate::{breaker::messages::BumpGrade, prelude::*};

// ── Behavior 30 — config absent: drain reader, no swap; live after insert ───

#[test]
fn config_absent_drains_reader_and_does_not_swap_then_goes_live_after_insert() {
    let mut app = build_conductor_app_no_config();
    seed_active_protocols_with_conductor(&mut app, 0.2);
    let breaker = spawn_dummy_breaker(&mut app);
    let primary = spawn_primary_bolt_with_bound(&mut app, make_distinct_bound("PRIMARY_BOUND"));
    let extra = spawn_extra_bolt_with_bound(&mut app, make_distinct_bound("EXTRA_BOUND"));

    // Tick 1 — config absent; buffered Perfect bump must not swap.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "tick 1: primary unchanged while config absent"
    );
    assert!(has_extra(&app, extra), "tick 1: extra unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "tick 1: primary's bound unchanged"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "tick 1: extra's bound unchanged"
    );

    // Tick 2 — insert config, NO new message. The pre-drain consumed the
    // tick-1 message, so tick 2 must see an empty reader and not swap.
    app.world_mut()
        .insert_resource(canonical_conductor_config());
    tick(&mut app);

    assert!(
        has_primary(&app, primary),
        "tick 2: no replay — primary unchanged"
    );
    assert!(has_extra(&app, extra), "tick 2: extra unchanged");
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["PRIMARY_BOUND".to_string()],
        "tick 2: primary's bound unchanged (buffered message was drained)"
    );
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["EXTRA_BOUND".to_string()],
        "tick 2: extra's bound unchanged (buffered message was drained)"
    );

    // Tick 3 — fresh message, config present — swap SHOULD fire now.
    write_bump_performed(&mut app, breaker, Some(extra), BumpGrade::Perfect);
    tick(&mut app);

    assert!(
        has_primary(&app, extra),
        "tick 3: system is live — extra promoted on fresh message"
    );
    assert!(has_extra(&app, primary), "tick 3: primary demoted");
    assert_eq!(
        bound_fingerprints(&app, extra),
        vec!["PRIMARY_BOUND".to_string()],
        "tick 3: swap ran — bound moved to bumped bolt"
    );
    assert_eq!(
        bound_fingerprints(&app, primary),
        vec!["EXTRA_BOUND".to_string()],
        "tick 3: swap ran — bound moved to old primary"
    );
}

// ── Behavior 30 (edge case) — quiet ticks with no config, no messages ───────

#[test]
fn multiple_quiet_ticks_with_config_absent_do_not_panic() {
    let mut app = build_conductor_app_no_config();
    seed_active_protocols_with_conductor(&mut app, 0.2);

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<ConductorConfig>().is_none(),
        "ConductorConfig must remain absent — register must not side-effect-insert"
    );
}

// ── Behavior 31 — register does not panic when config absent ────────────────

#[test]
fn register_does_not_panic_when_config_absent_and_no_active_protocol() {
    let mut app = build_conductor_app_no_config();
    // Do NOT seed ActiveProtocols. No bolts. No messages.

    for _ in 0..3 {
        tick(&mut app);
    }

    assert!(
        app.world().get_resource::<ConductorConfig>().is_none(),
        "register must not side-effect-insert ConductorConfig"
    );
}
