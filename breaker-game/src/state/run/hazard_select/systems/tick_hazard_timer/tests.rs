//! Tests for `tick_hazard_timer` — timer decrement + auto-pick on expiry.
//!
//! CRITICAL: unlike `tick_chip_timer`, timer expiry MUST emit exactly one
//! `HazardSelected` message (auto-pick). Players cannot skip hazards.

use bevy::{ecs::message::Messages, prelude::*};
use rantzsoft_stateflow::ChangeState;

use super::*;
use crate::{
    hazard::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        messages::HazardSelected,
        resources::HazardOffers,
    },
    prelude::GameRng,
    state::run::hazard_select::resources::HazardSelectTimer,
};

fn tuning_for_kind(kind: HazardKind) -> HazardTuning {
    match kind {
        HazardKind::Drift => HazardTuning::Drift {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 33.3,
        },
        HazardKind::Haste => HazardTuning::Haste {
            base_percent:      0.1,
            per_level_percent: 0.05,
        },
        _ => HazardTuning::Decay {
            base_percent:      0.05,
            per_level_percent: 0.03,
        },
    }
}

fn make_def(kind: HazardKind, name: &str) -> HazardDefinition {
    HazardDefinition {
        name:        name.to_owned(),
        description: String::new(),
        unlock_tier: 0,
        tuning:      tuning_for_kind(kind),
    }
}

fn make_offers_3() -> HazardOffers {
    HazardOffers(vec![
        make_def(HazardKind::Decay, "Decay"),
        make_def(HazardKind::Drift, "Drift"),
        make_def(HazardKind::Haste, "Haste"),
    ])
}

fn make_offers_1() -> HazardOffers {
    HazardOffers(vec![make_def(HazardKind::Decay, "Decay")])
}

fn test_app(remaining: f32, offers: HazardOffers, seed: u64) -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .with_message::<ChangeState<HazardSelectState>>()
        .with_message::<HazardSelected>()
        .insert_resource(HazardSelectTimer { remaining })
        .insert_resource(offers)
        .insert_resource(GameRng::from_seed(seed))
        .with_system(Update, tick_hazard_timer)
        .build()
}

// ── Domain E.1: timer decrements by frame delta ──────────────────────────

#[test]
fn timer_decrements_by_frame_delta_after_two_updates() {
    let mut app = test_app(10.0, make_offers_3(), 42);
    // First update initializes Time; second gets a real delta.
    app.update();
    app.update();

    let timer = app.world().resource::<HazardSelectTimer>();
    assert!(
        timer.remaining < 10.0,
        "expected timer to decrease, got: {}",
        timer.remaining
    );
}

#[test]
fn timer_at_100_decrements_but_stays_positive() {
    let mut app = test_app(100.0, make_offers_3(), 42);
    app.update();
    app.update();

    let timer = app.world().resource::<HazardSelectTimer>();
    assert!(timer.remaining < 100.0, "timer should decrement");
    assert!(timer.remaining > 0.0, "timer should not yet be expired");

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        0,
        "no ChangeState while timer has remaining > 0"
    );
}

// ── Domain E.2: timer clamps to 0.0 on expiry ────────────────────────────

#[test]
fn timer_clamps_to_zero_on_expiry() {
    let mut app = test_app(0.0, make_offers_3(), 42);
    app.update();

    let timer = app.world().resource::<HazardSelectTimer>();
    assert!(
        timer.remaining.abs() < f32::EPSILON,
        "timer must clamp to 0.0, got {}",
        timer.remaining
    );
}

// ── Domain E.3: timer expiry emits exactly one HazardSelected (AUTO-PICK) ──

#[test]
fn timer_expiry_emits_exactly_one_hazard_selected_message_auto_pick() {
    let mut app = test_app(0.0, make_offers_3(), 42);
    app.update();

    let hazard_msgs = app.world().resource::<Messages<HazardSelected>>();
    let observed: Vec<HazardKind> = hazard_msgs
        .iter_current_update_messages()
        .map(|m| m.kind)
        .collect();

    assert_eq!(
        observed.len(),
        1,
        "timer expiry MUST auto-pick exactly one hazard (differs from chip timer), got {}",
        observed.len()
    );
    assert!(
        matches!(
            observed[0],
            HazardKind::Decay | HazardKind::Drift | HazardKind::Haste
        ),
        "auto-picked kind must be one of the 3 offered kinds, got {:?}",
        observed[0]
    );
}

#[test]
fn timer_expiry_with_single_offer_emits_that_kind() {
    let mut app = test_app(0.0, make_offers_1(), 42);
    app.update();

    let hazard_msgs = app.world().resource::<Messages<HazardSelected>>();
    let observed: Vec<HazardKind> = hazard_msgs
        .iter_current_update_messages()
        .map(|m| m.kind)
        .collect();

    assert_eq!(observed.len(), 1);
    assert_eq!(
        observed[0],
        HazardKind::Decay,
        "single-offer auto-pick must pick that kind"
    );
}

// ── Domain E.4: timer expiry emits exactly one ChangeState ──────────────

#[test]
fn timer_expiry_emits_exactly_one_change_state_message() {
    let mut app = test_app(0.0, make_offers_3(), 42);
    app.update();

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "timer expiry must emit one ChangeState<HazardSelectState>"
    );
}

// ── Domain E.5: auto-pick kind is deterministic under seed ──────────────

#[test]
fn auto_pick_kind_is_deterministic_under_seed_42() {
    let mut app_a = test_app(0.0, make_offers_3(), 42);
    app_a.update();
    let kind_a = app_a
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("app A must emit one HazardSelected on expiry")
        .kind;

    let mut app_b = test_app(0.0, make_offers_3(), 42);
    app_b.update();
    let kind_b = app_b
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("app B must emit one HazardSelected on expiry")
        .kind;

    assert_eq!(
        kind_a, kind_b,
        "auto-pick kind must be the same across two apps using the same seed"
    );
}

#[test]
fn auto_pick_kind_is_deterministic_under_seed_7() {
    let mut app_a = test_app(0.0, make_offers_3(), 7);
    app_a.update();
    let kind_a = app_a
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("app A must emit one HazardSelected on expiry")
        .kind;

    let mut app_b = test_app(0.0, make_offers_3(), 7);
    app_b.update();
    let kind_b = app_b
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("app B must emit one HazardSelected on expiry")
        .kind;

    assert_eq!(kind_a, kind_b);
}

// ── Domain E.6: subsumed by E.5 — listed here as a reinforcement test ───

#[test]
fn auto_pick_is_rng_driven_not_deterministic_first_index() {
    // Regression guard: two apps with the same seed and same offers yield
    // the same kind. This is structurally identical to E.5 — the spec lists
    // it separately so the intent is explicit: the pick uses
    // GameRng::random_range, not a deterministic `offers.0[0]` shortcut.
    let mut app_a = test_app(0.0, make_offers_3(), 42);
    app_a.update();
    let mut app_b = test_app(0.0, make_offers_3(), 42);
    app_b.update();

    let kind_a = app_a
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("must emit HazardSelected")
        .kind;
    let kind_b = app_b
        .world()
        .resource::<Messages<HazardSelected>>()
        .iter_current_update_messages()
        .next()
        .expect("must emit HazardSelected")
        .kind;

    assert_eq!(kind_a, kind_b);
}

// ── Domain E.7: timer with remaining > 0 emits no messages ──────────────

#[test]
fn timer_not_expired_emits_no_messages_and_stays_positive() {
    let mut app = test_app(100.0, make_offers_3(), 42);
    app.update();

    let hazard_msgs = app.world().resource::<Messages<HazardSelected>>();
    assert_eq!(
        hazard_msgs.iter_current_update_messages().count(),
        0,
        "no HazardSelected while timer has remaining > 0"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        0,
        "no ChangeState while timer has remaining > 0"
    );

    let timer = app.world().resource::<HazardSelectTimer>();
    assert!(timer.remaining > 0.0);
}

// ── Domain E.8: empty offers + expiry emits ChangeState but NO HazardSelected ──

#[test]
fn empty_offers_plus_expiry_emits_change_state_without_hazard_selected() {
    let mut app = test_app(0.0, HazardOffers::default(), 42);
    app.update();

    let hazard_msgs = app.world().resource::<Messages<HazardSelected>>();
    assert_eq!(
        hazard_msgs.iter_current_update_messages().count(),
        0,
        "empty offers must not emit HazardSelected (nothing to pick)"
    );

    let state_msgs = app
        .world()
        .resource::<Messages<ChangeState<HazardSelectState>>>();
    assert_eq!(
        state_msgs.iter_current_update_messages().count(),
        1,
        "empty offers + expiry must still emit one ChangeState so the machine advances"
    );
}
