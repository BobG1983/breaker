use std::sync::Arc;

use bevy::prelude::States;

use super::data::*;
use crate::transition::{
    traits::{InTransition, OutTransition, Transition},
    types::{TransitionKind, TransitionType},
};

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum TestState {
    #[default]
    A,
    B,
    C,
}

struct TestEffectOut;
impl Transition for TestEffectOut {}
impl OutTransition for TestEffectOut {}

struct TestEffectIn;
impl Transition for TestEffectIn {}
impl InTransition for TestEffectIn {}

// --- Existing tests (preserved) ---

#[test]
fn static_route_compiles() {
    let _route = Route::from(TestState::A).to(TestState::B);
}

#[test]
fn dynamic_route_compiles() {
    let _route = Route::from(TestState::A).to_dynamic(|_world| TestState::B);
}

#[test]
fn condition_triggered_route_compiles() {
    let _route = Route::from(TestState::A)
        .to(TestState::B)
        .when(|_world| true);
}

#[test]
fn static_route_stores_from_and_destination() {
    let builder = Route::from(TestState::A).to(TestState::B);
    let route = builder.finalize();
    assert_eq!(route.from, TestState::A);
    assert!(
        matches!(route.destination, DestinationKind::Static(TestState::B)),
        "expected static destination B"
    );
}

#[test]
fn dynamic_route_resolves_destination() {
    let builder = Route::from(TestState::A).to_dynamic(|_world| TestState::C);
    let route = builder.finalize();
    assert_eq!(route.from, TestState::A);
    assert!(matches!(route.destination, DestinationKind::Dynamic(_)));
}

#[test]
fn message_trigger_is_default() {
    let builder = Route::from(TestState::A).to(TestState::B);
    let route = builder.finalize();
    assert!(matches!(route.trigger, TriggerKind::Message));
}

#[test]
fn when_sets_condition_trigger() {
    let builder = Route::from(TestState::A).to(TestState::B).when(|_| false);
    let route = builder.finalize();
    assert!(matches!(route.trigger, TriggerKind::Condition(_)));
}

// --- Section F: Transition extension tests ---

// Behavior 1: with_transition sets static transition on route

#[test]
fn with_transition_sets_static_transition() {
    let route = Route::from(TestState::A)
        .to(TestState::B)
        .with_transition(TransitionType::Out(Arc::new(TestEffectOut)))
        .finalize();
    assert!(
        matches!(
            route.transition,
            TransitionKind::Static(TransitionType::Out(_))
        ),
        "expected TransitionKind::Static(TransitionType::Out(_))"
    );
}

#[test]
fn route_without_transition_has_kind_none() {
    let route = Route::from(TestState::A).to(TestState::B).finalize();
    assert!(
        matches!(route.transition, TransitionKind::None),
        "expected TransitionKind::None for route without with_transition()"
    );
}

// Behavior 2: with_dynamic_transition sets dynamic transition

#[test]
fn with_dynamic_transition_sets_dynamic_transition() {
    let route = Route::from(TestState::A)
        .to(TestState::B)
        .with_dynamic_transition(|_world| TransitionType::Out(Arc::new(TestEffectOut)))
        .finalize();
    assert!(
        matches!(route.transition, TransitionKind::Dynamic(_)),
        "expected TransitionKind::Dynamic(_)"
    );
}

// Behavior 3: with_transition is only available on NoTransition typestate
// Compile-time guarantee: calling .with_transition() twice does NOT compile.
// This is verified by the type system -- `with_transition` returns
// `RouteBuilder<S, _, StaticTransition, _>`, and `with_transition` is only
// implemented on `RouteBuilder<S, _, NoTransition, _>`.

// Behavior 4: with_transition composes with when()

#[test]
fn with_transition_composes_with_when() {
    let route = Route::from(TestState::A)
        .to(TestState::B)
        .with_transition(TransitionType::Out(Arc::new(TestEffectOut)))
        .when(|_| true)
        .finalize();
    assert!(
        matches!(
            route.transition,
            TransitionKind::Static(TransitionType::Out(_))
        ),
        "expected static transition"
    );
    assert!(
        matches!(route.trigger, TriggerKind::Condition(_)),
        "expected condition trigger"
    );
}

#[test]
fn when_then_with_transition_composes_equally() {
    // Order shouldn't matter -- when() and with_transition() are on different axes
    let route = Route::from(TestState::A)
        .to(TestState::B)
        .when(|_| true)
        .with_transition(TransitionType::Out(Arc::new(TestEffectOut)))
        .finalize();
    assert!(
        matches!(
            route.transition,
            TransitionKind::Static(TransitionType::Out(_))
        ),
        "expected static transition"
    );
    assert!(
        matches!(route.trigger, TriggerKind::Condition(_)),
        "expected condition trigger"
    );
}

// Behavior 5: with_transition works with dynamic destination

#[test]
fn with_transition_works_with_dynamic_destination() {
    let route = Route::from(TestState::A)
        .to_dynamic(|_| TestState::B)
        .with_transition(TransitionType::In(Arc::new(TestEffectIn)))
        .finalize();
    assert!(
        matches!(route.destination, DestinationKind::Dynamic(_)),
        "expected dynamic destination"
    );
    assert!(
        matches!(
            route.transition,
            TransitionKind::Static(TransitionType::In(_))
        ),
        "expected static In transition"
    );
}
