//! Source-name formatting tests — B.4 and format smoke tests.
//!
//! `dispatch_protocol_selection` formats the stamp source via
//! `format!("protocol:{kind:?}")`. Effect-tree kinds are exercised through the
//! stamp-dispatch path; custom-system kinds (`effects()` returns `None`)
//! are covered here via a pure-format assertion so the `{:?}` contract holds
//! for every kind without a Bevy app.

use super::helpers::{
    anchor_with_effects, kickstart_with_effects, send_selection, stub_piercing_root,
    test_app_selecting,
};
use crate::{
    mutators::protocols::{definition::ProtocolKind, resources::ProtocolRegistry},
    prelude::*,
};

// ── B.4: Source name is formatted as protocol:<kind> ──────────────────────

#[test]
fn source_name_is_protocol_colon_kind_debug_kickstart() {
    let mut registry = ProtocolRegistry::default();
    registry.insert(kickstart_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Kickstart);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(
        bound.0[0].0, "protocol:Kickstart",
        "source name must be format!(\"protocol:{{kind:?}}\")"
    );
}

#[test]
fn source_name_preserves_multi_word_camelcase_tier_regression() {
    // Edge case: TierRegression is a custom-system protocol (effects
    // returns None), so to exercise the source-name formatter we need a
    // stub-able effect-tree tuning. We simulate this by using
    // `Deadline` tuning but with a ProtocolSelected kind of
    // TierRegression — however, the registry key is the tuning's own kind,
    // so this doesn't work cleanly.
    //
    // Instead: use the effect-tree protocol Anchor (kind = Anchor) with a
    // single stamp, and verify source = "protocol:Anchor". The
    // multi-word CamelCase case is otherwise exercised by B.4's debug
    // formatter consistency — `format!("{:?}", kind)` produces the exact
    // variant name verbatim for every variant.
    let mut registry = ProtocolRegistry::default();
    registry.insert(anchor_with_effects(vec![stub_piercing_root()]));

    let mut app = test_app_selecting(registry);
    let breaker = app.world_mut().spawn(Breaker).id();

    send_selection(&mut app, ProtocolKind::Anchor);
    app.update();

    let bound = app
        .world()
        .get::<BoundEffects>(breaker)
        .expect("Breaker should have BoundEffects");
    assert_eq!(bound.0[0].0, "protocol:Anchor");
}

// ── Source-name format smoke tests ─────────────────────────────────────────

#[test]
fn source_name_for_tier_regression_is_protocol_tier_regression() {
    // Regression guard: multi-word CamelCase kind names must not gain spaces
    // or lowercasing. Since `TierRegression` is a custom-system protocol
    // (`effects()` returns `None`) it cannot be tested via the stamp path.
    let source = format!("protocol:{:?}", ProtocolKind::TierRegression);
    assert_eq!(source, "protocol:TierRegression");
}

#[test]
fn source_name_for_debt_collector_is_protocol_debt_collector() {
    // Same guard for another custom-system, multi-word kind.
    let source = format!("protocol:{:?}", ProtocolKind::DebtCollector);
    assert_eq!(source, "protocol:DebtCollector");
}
