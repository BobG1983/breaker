//! Group F — `wire` wiring + run-condition gate (Behaviors 30–32).
//!
//! Pins that `wire` actually wires `greed_on_skip` (not a stub/warn),
//! that the run-condition holds it off when Greed isn't in `ActiveProtocols`,
//! and that the system does not panic when `GreedStacks` is absent.

use super::{
    super::system::GreedStacks,
    helpers::{
        build_greed_app, build_greed_app_no_stacks, seed_active_protocols_with_greed, write_skip,
    },
};

// ── Behavior 30 — wire wires greed_on_skip under protocol_active(Greed) ──

#[test]
fn register_wires_greed_on_skip_when_greed_is_active() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);

    write_skip(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        1,
        "wire must wire greed_on_skip such that it increments skips when \
         Greed is active; got {}",
        app.world().resource::<GreedStacks>().skips
    );
}

// ── Behavior 31 — wire-wired system is gated off when Greed NOT active ──

#[test]
fn register_wired_system_does_not_run_when_greed_is_not_active() {
    let mut app = build_greed_app();
    // ActiveProtocols intentionally left empty — Greed NOT seeded.

    write_skip(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        0,
        "run_if(protocol_active(Greed)) must hold greed_on_skip off when Greed \
         isn't in ActiveProtocols; got {}",
        app.world().resource::<GreedStacks>().skips
    );
}

// ── Behavior 32 — wire does not panic when GreedStacks is absent ────────

#[test]
fn register_does_not_panic_when_greed_stacks_absent() {
    let mut app = build_greed_app_no_stacks();
    seed_active_protocols_with_greed(&mut app, 0.05);

    write_skip(&mut app);
    app.update();

    // Reaching here without panic is the assertion. Also confirm the system
    // did not side-effect-insert the resource.
    assert!(
        app.world().get_resource::<GreedStacks>().is_none(),
        "wire-wired system must not insert GreedStacks as a side effect; \
         the plugin is responsible for init_resource"
    );
}
