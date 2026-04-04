//! Route builder — typestate API for declaring state-to-state transitions.
//!
//! Four independent axes:
//! - **Destination**: `.to(S)` (static) or `.to_dynamic(fn)` (runtime)
//! - **Transition**: `.with_transition(T)` (static) or `.with_dynamic_transition(fn)` (runtime)
//! - **Trigger**: message-triggered (default) or `.when(fn)` (polling)
//!
//! Invalid combinations (e.g. calling `.to()` twice, `.with_transition()` twice)
//! are prevented at compile time via phantom type parameters.

use std::marker::PhantomData;

use bevy::{ecs::world::World, prelude::States};

use crate::transition::types::{TransitionKind, TransitionType};

// ── Typestate markers ────────────────────────────────────────

/// Destination not yet set (required).
pub struct NoDest;
/// Static destination set via `.to(S)`.
pub struct StaticDest;
/// Dynamic destination set via `.to_dynamic(fn)`.
pub struct DynamicDest;

/// No transition effect (default — instant state change).
pub struct NoTransition;
/// Static transition effect set via `.with_transition(T)`.
pub struct StaticTransition;
/// Dynamic transition effect set via `.with_dynamic_transition(fn)`.
pub struct DynamicTransition;

/// Message-triggered route (default).
pub struct MessageTrigger;
/// Condition-triggered route via `.when(fn)`.
pub struct ConditionTrigger;

// ── Internal storage ─────────────────────────────────────────

/// How the destination is resolved.
#[doc(hidden)]
pub enum DestinationKind<S> {
    /// Not yet set.
    None,
    /// Fixed destination variant.
    Static(S),
    /// Computed at dispatch time.
    Dynamic(Box<dyn Fn(&World) -> S + Send + Sync>),
}

/// How the route is triggered.
#[doc(hidden)]
pub enum TriggerKind {
    /// Triggered by `ChangeState<S>` message.
    Message,
    /// Triggered by polling a condition each frame.
    Condition(Box<dyn Fn(&World) -> bool + Send + Sync>),
}

// ── RouteBuilder ─────────────────────────────────────────────

/// Fluent builder for a single route. Consumed by
/// [`RoutingTable::add`](crate::RoutingTable::add).
pub struct RouteBuilder<S: States, Dest, Trans, Trigger> {
    pub(crate) from: S,
    pub(crate) destination: DestinationKind<S>,
    pub(crate) transition: TransitionKind,
    pub(crate) trigger: TriggerKind,
    _phantom: PhantomData<(Dest, Trans, Trigger)>,
}

// ── Entry point ──────────────────────────────────────────────

impl Route {
    /// Start building a route from the given state variant.
    pub const fn from<S: States>(
        state: S,
    ) -> RouteBuilder<S, NoDest, NoTransition, MessageTrigger> {
        RouteBuilder {
            from: state,
            destination: DestinationKind::None,
            transition: TransitionKind::None,
            trigger: TriggerKind::Message,
            _phantom: PhantomData,
        }
    }
}

/// Namespace for the [`Route::from`] entry point.
pub struct Route;

// ── Destination methods (only on NoDest) ─────────────────────

impl<S: States, Trans, Trigger> RouteBuilder<S, NoDest, Trans, Trigger> {
    /// Set a fixed destination state.
    pub fn to(self, dest: S) -> RouteBuilder<S, StaticDest, Trans, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: DestinationKind::Static(dest),
            transition: self.transition,
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Set a dynamic destination resolved at dispatch time.
    pub fn to_dynamic(
        self,
        f: impl Fn(&World) -> S + Send + Sync + 'static,
    ) -> RouteBuilder<S, DynamicDest, Trans, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: DestinationKind::Dynamic(Box::new(f)),
            transition: self.transition,
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }
}

// ── Transition methods (only on NoTransition) ────────────────

impl<S: States, Dest, Trigger> RouteBuilder<S, Dest, NoTransition, Trigger> {
    /// Set a static transition effect for this route.
    pub fn with_transition(
        self,
        transition: TransitionType,
    ) -> RouteBuilder<S, Dest, StaticTransition, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: self.destination,
            transition: TransitionKind::Static(transition),
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Set a dynamic transition effect resolved at dispatch time.
    pub fn with_dynamic_transition(
        self,
        f: impl Fn(&World) -> TransitionType + Send + Sync + 'static,
    ) -> RouteBuilder<S, Dest, DynamicTransition, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: self.destination,
            transition: TransitionKind::Dynamic(Box::new(f)),
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }
}

// ── Trigger methods (only on MessageTrigger) ─────────────────

impl<S: States, Dest, Trans> RouteBuilder<S, Dest, Trans, MessageTrigger> {
    /// Make this route condition-triggered instead of message-triggered.
    ///
    /// The condition is polled each frame in `Update`. When it returns
    /// `true`, the route fires once.
    pub fn when(
        self,
        f: impl Fn(&World) -> bool + Send + Sync + 'static,
    ) -> RouteBuilder<S, Dest, Trans, ConditionTrigger> {
        RouteBuilder {
            from: self.from,
            destination: self.destination,
            transition: self.transition,
            trigger: TriggerKind::Condition(Box::new(f)),
            _phantom: PhantomData,
        }
    }
}

// ── Finalized route (for RoutingTable consumption) ───────────

/// A finalized route ready for insertion into a [`RoutingTable`](crate::RoutingTable).
///
/// Not intended for direct construction — use the [`Route`] builder.
#[doc(hidden)]
pub struct FinalizedRoute<S: States> {
    /// The source state variant.
    pub from: S,
    /// How to determine the destination.
    pub destination: DestinationKind<S>,
    /// How the transition effect is configured.
    pub transition: TransitionKind,
    /// How the route is triggered.
    pub trigger: TriggerKind,
}

/// Trait for converting a `RouteBuilder` with a destination into a `FinalizedRoute`.
///
/// Implemented automatically for `RouteBuilder` types that have a destination set.
/// Used as a bound on [`RoutingTable::add`](crate::RoutingTable::add).
pub trait IntoFinalizedRoute<S: States> {
    /// Consume the builder and produce a finalized route.
    fn finalize(self) -> FinalizedRoute<S>;
}

impl<S: States, Trans, Trigger> IntoFinalizedRoute<S>
    for RouteBuilder<S, StaticDest, Trans, Trigger>
{
    fn finalize(self) -> FinalizedRoute<S> {
        FinalizedRoute {
            from: self.from,
            destination: self.destination,
            transition: self.transition,
            trigger: self.trigger,
        }
    }
}

impl<S: States, Trans, Trigger> IntoFinalizedRoute<S>
    for RouteBuilder<S, DynamicDest, Trans, Trigger>
{
    fn finalize(self) -> FinalizedRoute<S> {
        FinalizedRoute {
            from: self.from,
            destination: self.destination,
            transition: self.transition,
            trigger: self.trigger,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::transition::traits::{InTransition, OutTransition, Transition};

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
    // This is verified by the type system — `with_transition` returns
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
        // Order shouldn't matter — when() and with_transition() are on different axes
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
}
