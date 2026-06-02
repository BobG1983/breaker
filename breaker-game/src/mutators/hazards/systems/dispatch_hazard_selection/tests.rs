//! Tests for `dispatch_hazard_selection` — consumes `HazardSelected` messages
//! and increments `ActiveHazards.add_stack(kind)` while gated on
//! `HazardSelectState::Selecting`.

use bevy::{ecs::message::Messages, prelude::*};

use super::dispatch_hazard_selection;
use crate::{
    input::InputConfig,
    mutators::hazards::{
        definition::{HazardDefinition, HazardKind, HazardTuning},
        messages::HazardSelected,
        resources::{ActiveHazards, HazardOffers, HazardRegistry},
    },
    prelude::*,
    shared::rng::HazardRng,
    state::run::hazard_select::{
        resources::HazardSelectSelection, sets::HazardSelectSystems,
        systems::handle_hazard_input::handle_hazard_input,
    },
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
        HazardKind::Sympathy => HazardTuning::Sympathy {
            base_heal_frac:      0.25,
            per_level_heal_frac: 0.05,
            depth_every_levels:  5,
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

/// Seeds a [`HazardRegistry`] with every kind referenced by tests in this
/// module so `dispatch_hazard_selection` can resolve the definition and
/// increment the stack. Dispatch now fails closed on missing definitions —
/// see `dispatch_with_missing_registry_definition_does_not_increment_stack`.
fn seeded_registry() -> HazardRegistry {
    let mut registry = HazardRegistry::default();
    for kind in [
        HazardKind::Decay,
        HazardKind::Drift,
        HazardKind::Haste,
        HazardKind::Sympathy,
    ] {
        registry.insert(make_def(kind, "test"));
    }
    registry
}

fn test_app_selecting() -> App {
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_hazard_selecting()
        .with_resource::<ActiveHazards>()
        .insert_resource(seeded_registry())
        .with_message::<HazardSelected>()
        .with_system(
            Update,
            dispatch_hazard_selection.run_if(in_state(HazardSelectState::Selecting)),
        )
        .build()
}

fn send_hazard(app: &mut App, kind: HazardKind) {
    app.world_mut()
        .resource_mut::<Messages<HazardSelected>>()
        .write(HazardSelected { kind });
}

// ── Domain F.1: single message increments stacks(kind) by 1 ──────────────

#[test]
fn single_hazard_selected_message_increments_stacks_decay() {
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(active.stacks(HazardKind::Decay), 1);
    assert!(active.is_active(HazardKind::Decay));
}

#[test]
fn single_hazard_selected_message_increments_stacks_sympathy() {
    // Edge case of F.1: a different variant follows the same dispatch path.
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Sympathy);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(active.stacks(HazardKind::Sympathy), 1);
    assert!(active.is_active(HazardKind::Sympathy));
}

// ── Domain F.2: two same-kind messages produce stacks == 2 ───────────────

#[test]
fn two_same_kind_messages_in_same_frame_produce_stacks_two() {
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Decay);
    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        2,
        "two same-frame Decay messages must produce stacks == 2"
    );
}

#[test]
fn two_same_kind_messages_across_frames_produce_stacks_two() {
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Decay);
    app.update();
    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        2,
        "two cross-frame Decay messages must produce stacks == 2"
    );
}

#[test]
fn three_same_kind_messages_produce_stacks_three() {
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Decay);
    send_hazard(&mut app, HazardKind::Decay);
    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(active.stacks(HazardKind::Decay), 3);
}

// ── Domain F.3: messages of different kinds each have stacks == 1 ────────

#[test]
fn two_different_kind_messages_produce_stacks_one_each() {
    let mut app = test_app_selecting();
    send_hazard(&mut app, HazardKind::Decay);
    send_hazard(&mut app, HazardKind::Drift);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(active.stacks(HazardKind::Decay), 1);
    assert_eq!(active.stacks(HazardKind::Drift), 1);
    assert_eq!(active.len(), 2);
}

// ── Domain F.4: message outside HazardSelectState::Selecting is NOT consumed ──

#[test]
fn message_outside_hazard_select_selecting_is_not_consumed() {
    // Build the app at NodeState::Playing instead of HazardSelectState::Selecting.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveHazards>()
        .insert_resource(seeded_registry())
        .with_message::<HazardSelected>()
        .with_system(
            Update,
            dispatch_hazard_selection.run_if(in_state(HazardSelectState::Selecting)),
        )
        .build();
    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        0,
        "dispatch must be gated off when not in HazardSelectState::Selecting"
    );
    assert!(active.is_empty());
}

// ── Domain F.5: .after(HazardSelectSystems::HandleInput) — same-frame race guard ──

#[test]
fn dispatch_after_handle_input_consumes_same_frame_message() {
    // Wire handle_hazard_input (which emits HazardSelected when Enter is
    // pressed) BEFORE dispatch_hazard_selection, using the production
    // .after(HazardSelectSystems::HandleInput) ordering. Pressing Enter this
    // frame must result in ActiveHazards.stacks(Decay) == 1 by the end of
    // the same app.update() — regression guard: dispatch must consume
    // HazardSelected emitted by input within the same frame.
    use rantzsoft_stateflow::ChangeState;

    let offers = HazardOffers(vec![
        make_def(HazardKind::Decay, "Decay"),
        make_def(HazardKind::Drift, "Drift"),
        make_def(HazardKind::Haste, "Haste"),
    ]);

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_hazard_selecting()
        .with_resource::<ActiveHazards>()
        .insert_resource(seeded_registry())
        .with_resource::<ButtonInput<KeyCode>>()
        .insert_resource(InputConfig::default())
        .insert_resource(HazardSelectSelection { card_index: 0 })
        .insert_resource(offers)
        .with_message::<HazardSelected>()
        .with_message::<ChangeState<HazardSelectState>>()
        .with_system(
            Update,
            (
                handle_hazard_input.in_set(HazardSelectSystems::HandleInput),
                dispatch_hazard_selection.after(HazardSelectSystems::HandleInput),
            )
                .run_if(in_state(HazardSelectState::Selecting)),
        )
        .build();

    crate::shared::test_utils::press_key(&mut app, KeyCode::Enter);

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        1,
        "same-frame race guard: dispatch must consume HazardSelected emitted by input this frame"
    );
}

// ── Domain F.6: .after(HazardSelectSystems::TickTimer) — timer-expiry race guard ──

#[test]
fn dispatch_after_tick_timer_consumes_expiry_message() {
    // Wire tick_hazard_timer (which emits HazardSelected on expiry when offers
    // are present) BEFORE dispatch_hazard_selection, using the production
    // .after(HazardSelectSystems::TickTimer) ordering. When the timer hits 0
    // with a single-kind HazardOffers, ActiveHazards.stacks(Decay) must be 1
    // at the end of the same app.update() — regression guard for the race
    // where dispatch runs before tick_timer and drops the auto-pick.
    use std::time::Duration;

    use rantzsoft_stateflow::ChangeState;

    use crate::state::run::hazard_select::{
        resources::HazardSelectTimer, systems::tick_hazard_timer::tick_hazard_timer,
    };

    let offers = HazardOffers(vec![make_def(HazardKind::Decay, "Decay")]);

    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_hazard_selecting()
        .with_resource::<ActiveHazards>()
        .insert_resource(seeded_registry())
        .insert_resource(HazardSelectTimer { remaining: 0.0 })
        .insert_resource(offers)
        .with_resource::<HazardRng>()
        .with_message::<HazardSelected>()
        .with_message::<ChangeState<HazardSelectState>>()
        .with_system(
            Update,
            (
                tick_hazard_timer.in_set(HazardSelectSystems::TickTimer),
                dispatch_hazard_selection.after(HazardSelectSystems::TickTimer),
            )
                .run_if(in_state(HazardSelectState::Selecting)),
        )
        .build();

    // Advance virtual time so delta_secs > 0 and the timer-expiry branch fires.
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(16));
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        1,
        "timer-expiry race guard: dispatch must consume HazardSelected emitted by tick_hazard_timer this frame"
    );
}

// ── Domain F.7: dispatch is a stub — no side effects other than add_stack ──

#[test]
fn dispatch_does_not_spawn_entities_or_install_bound_effects() {
    // After a single HazardSelected(Decay) is processed, no Breaker entity
    // should appear and no entity should gain BoundEffects. Only ActiveHazards
    // mutates.
    let mut app = test_app_selecting();

    let pre_entity_count = app.world_mut().query::<Entity>().iter(app.world()).count();

    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(active.stacks(HazardKind::Decay), 1);

    let post_entity_count = app.world_mut().query::<Entity>().iter(app.world()).count();
    assert_eq!(
        post_entity_count, pre_entity_count,
        "dispatch must not spawn entities — entity count should be unchanged"
    );

    let breaker_count = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .count();
    assert_eq!(breaker_count, 0, "dispatch must not spawn Breaker");

    let bound_count = app
        .world_mut()
        .query::<&BoundEffects>()
        .iter(app.world())
        .count();
    assert_eq!(
        bound_count, 0,
        "dispatch must not install BoundEffects — it only increments stacks"
    );
}

// ── Fail-closed: missing registry definition does NOT increment stack ───────

/// Regression pin for backlog #15 G2: previously `dispatch_hazard_selection`
/// called `active.add_stack(msg.kind)` unconditionally and then skipped
/// `activate()` when the registry lacked a definition, leaving a silent
/// half-applied hazard (stack > 0, no runtime effect). After the fail-closed
/// fix, a `HazardSelected` for an unregistered kind must NOT increment
/// `ActiveHazards`.
#[test]
fn dispatch_with_missing_registry_definition_does_not_increment_stack() {
    // Build an app with an EMPTY HazardRegistry (no definitions seeded) so
    // `registry.get(Decay)` returns None and dispatch must fail closed.
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_hazard_selecting()
        .with_resource::<ActiveHazards>()
        .with_resource::<HazardRegistry>() // default = empty
        .with_message::<HazardSelected>()
        .with_system(
            Update,
            dispatch_hazard_selection.run_if(in_state(HazardSelectState::Selecting)),
        )
        .build();

    send_hazard(&mut app, HazardKind::Decay);
    app.update();

    let active = app.world().resource::<ActiveHazards>();
    assert_eq!(
        active.stacks(HazardKind::Decay),
        0,
        "missing registry definition → fail closed → stack NOT incremented \
         (silent half-apply bug fix)"
    );
    assert!(
        active.is_empty(),
        "ActiveHazards must remain empty when registry lookup returns None"
    );
}
