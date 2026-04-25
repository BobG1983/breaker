//! Group C — `greed_on_skip` system behavior (Behaviors 13–18, 18a + 29a).
//!
//! Pins that `greed_on_skip` is a pure accumulator: each `ChipOfferSkipped`
//! message increments `GreedStacks.skips` by 1; the run-condition gate holds
//! the system off when Greed isn't in `ActiveProtocols`; absence of
//! `GreedStacks` must not panic; no state transitions are driven; and the
//! stacks resource is run-persistent across arbitrary ticks.

use bevy::prelude::*;

use super::{
    super::system::GreedStacks,
    helpers::{
        build_greed_app, build_greed_app_no_stacks, install_greed_stacks,
        seed_active_protocols_with_greed, write_n_skips, write_skip,
    },
};
use crate::{
    mutators::protocols::resources::ActiveProtocols, prelude::*,
    state::run::chip_select::messages::ChipOfferSkipped,
};

// ── Behavior 13 — first skip increments from 0 to 1 ─────────────────────────

#[test]
fn first_chip_offer_skipped_increments_from_zero_to_one() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);
    write_skip(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        1,
        "first ChipOfferSkipped should leave GreedStacks.skips == 1"
    );
}

#[test]
fn subsequent_tick_without_new_skip_leaves_stacks_at_one() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);
    write_skip(&mut app);
    app.update();
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        1,
        "no new message in frame 2 should leave skips unchanged at 1"
    );
}

// ── Behavior 14 — N messages in one frame increment by N ────────────────────

#[test]
fn three_chip_offer_skipped_messages_in_one_frame_produce_three_increments() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);
    write_n_skips(&mut app, 3);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        3,
        "3 messages in one frame should increment skips to 3 (one per message, \
         not once per tick)"
    );
}

// ── Behavior 15 — skips accumulate across frames ────────────────────────────

#[test]
fn skips_accumulate_across_multiple_frames() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);
    install_greed_stacks(&mut app, 2);

    // Frame 1: 1 skip → 2 + 1 = 3.
    write_skip(&mut app);
    app.update();

    // Frame 2: 2 skips → 3 + 2 = 5.
    write_n_skips(&mut app, 2);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        5,
        "skips must accumulate monotonically across frames; expected 5, got {}",
        app.world().resource::<GreedStacks>().skips
    );
}

// ── Behavior 16 — system is gated off when Greed not active ─────────────────

#[test]
fn greed_on_skip_does_not_increment_when_greed_is_not_active() {
    let mut app = build_greed_app();
    // ActiveProtocols intentionally left empty — Greed NOT seeded.
    assert!(
        app.world().resource::<ActiveProtocols>().is_empty(),
        "precondition: Greed must NOT be active for this test"
    );

    write_skip(&mut app);
    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        0,
        "run-condition must gate the system off when Greed is not active; \
         expected skips == 0, got {}",
        app.world().resource::<GreedStacks>().skips
    );
}

// ── Behavior 17 — absence of GreedStacks must not panic ─────────────────────

#[test]
fn greed_on_skip_does_not_panic_when_greed_stacks_resource_absent() {
    let mut app = build_greed_app_no_stacks();
    seed_active_protocols_with_greed(&mut app, 0.05);

    write_skip(&mut app);
    app.update();

    assert!(
        app.world().get_resource::<GreedStacks>().is_none(),
        "greed_on_skip must not side-effect-insert GreedStacks when absent"
    );
}

// ── Behavior 18 — no messages → stacks unchanged ────────────────────────────

#[test]
fn no_chip_offer_skipped_messages_leaves_stacks_unchanged() {
    let mut app = build_greed_app();
    seed_active_protocols_with_greed(&mut app, 0.05);
    install_greed_stacks(&mut app, 4);

    app.update();

    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        4,
        "empty reader must not touch GreedStacks; expected 4, got {}",
        app.world().resource::<GreedStacks>().skips
    );
}

// ── Behavior 18a — greed_on_skip does not drive state transitions ───────────

#[test]
fn greed_on_skip_does_not_transition_chip_select_state() {
    // Build an app with the full state hierarchy so ChipSelectState is live.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .with_resource::<GreedStacks>()
        .with_message::<ChipOfferSkipped>()
        .in_state_chip_selecting()
        .build();
    super::super::system::register(&mut app);
    seed_active_protocols_with_greed(&mut app, 0.05);

    let before = *app.world().resource::<State<ChipSelectState>>().get();

    write_skip(&mut app);
    app.update();

    // Skip was processed.
    assert_eq!(
        app.world().resource::<GreedStacks>().skips,
        1,
        "precondition: greed_on_skip must have incremented skips"
    );

    let after = *app.world().resource::<State<ChipSelectState>>().get();
    assert_eq!(
        before, after,
        "greed_on_skip must NOT drive a ChipSelectState transition; state was \
         {before:?}, now {after:?}"
    );

    // NextState must not hold a pending transition.
    let next = app.world().resource::<NextState<ChipSelectState>>();
    assert!(
        matches!(next, NextState::Unchanged),
        "greed_on_skip must leave NextState<ChipSelectState> unchanged; got {next:?}"
    );
}

// ── Behavior 29a — GreedStacks persists across ticks without reset ──────────

#[test]
fn greed_stacks_persists_across_five_ticks_when_nothing_touches_it() {
    // Build an app that does NOT run greed_on_skip or reset_run_state; just
    // has GreedStacks inserted and ticks the Main schedule. `GreedStacks`
    // must be run-persistent across arbitrary ticks.
    let mut app = TestAppBuilder::new().with_resource::<GreedStacks>().build();
    install_greed_stacks(&mut app, 3);

    for tick_index in 0..5 {
        app.update();
        assert_eq!(
            app.world().resource::<GreedStacks>().skips,
            3,
            "GreedStacks.skips must remain 3 across every tick (tick {}); got {}",
            tick_index,
            app.world().resource::<GreedStacks>().skips
        );
    }
}
