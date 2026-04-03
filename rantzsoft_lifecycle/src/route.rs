//! Route builder — typestate API for declaring state-to-state transitions.
//!
//! Three independent axes:
//! - **Destination**: `.to(S)` (static) or `.to_dynamic(fn)` (runtime)
//! - **Trigger**: message-triggered (default) or `.when(fn)` (polling)
//!
//! Invalid combinations (e.g. calling `.to()` twice) are prevented at
//! compile time via phantom type parameters.

use std::marker::PhantomData;

use bevy::{ecs::world::World, prelude::States};

// ── Typestate markers ────────────────────────────────────────

/// Destination not yet set (required).
pub struct NoDest;
/// Static destination set via `.to(S)`.
pub struct StaticDest;
/// Dynamic destination set via `.to_dynamic(fn)`.
pub struct DynamicDest;

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
pub struct RouteBuilder<S: States, Dest, Trigger> {
    pub(crate) from: S,
    pub(crate) destination: DestinationKind<S>,
    pub(crate) trigger: TriggerKind,
    _phantom: PhantomData<(Dest, Trigger)>,
}

// ── Entry point ──────────────────────────────────────────────

impl Route {
    /// Start building a route from the given state variant.
    pub const fn from<S: States>(state: S) -> RouteBuilder<S, NoDest, MessageTrigger> {
        RouteBuilder {
            from: state,
            destination: DestinationKind::None,
            trigger: TriggerKind::Message,
            _phantom: PhantomData,
        }
    }
}

/// Namespace for the [`Route::from`] entry point.
pub struct Route;

// ── Destination methods (only on NoDest) ─────────────────────

impl<S: States, Trigger> RouteBuilder<S, NoDest, Trigger> {
    /// Set a fixed destination state.
    pub fn to(self, dest: S) -> RouteBuilder<S, StaticDest, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: DestinationKind::Static(dest),
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }

    /// Set a dynamic destination resolved at dispatch time.
    pub fn to_dynamic(
        self,
        f: impl Fn(&World) -> S + Send + Sync + 'static,
    ) -> RouteBuilder<S, DynamicDest, Trigger> {
        RouteBuilder {
            from: self.from,
            destination: DestinationKind::Dynamic(Box::new(f)),
            trigger: self.trigger,
            _phantom: PhantomData,
        }
    }
}

// ── Trigger methods (only on MessageTrigger) ─────────────────

impl<S: States, Dest> RouteBuilder<S, Dest, MessageTrigger> {
    /// Make this route condition-triggered instead of message-triggered.
    ///
    /// The condition is polled each frame in `Update`. When it returns
    /// `true`, the route fires once.
    pub fn when(
        self,
        f: impl Fn(&World) -> bool + Send + Sync + 'static,
    ) -> RouteBuilder<S, Dest, ConditionTrigger> {
        RouteBuilder {
            from: self.from,
            destination: self.destination,
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

impl<S: States, Trigger> IntoFinalizedRoute<S> for RouteBuilder<S, StaticDest, Trigger> {
    fn finalize(self) -> FinalizedRoute<S> {
        FinalizedRoute {
            from: self.from,
            destination: self.destination,
            trigger: self.trigger,
        }
    }
}

impl<S: States, Trigger> IntoFinalizedRoute<S> for RouteBuilder<S, DynamicDest, Trigger> {
    fn finalize(self) -> FinalizedRoute<S> {
        FinalizedRoute {
            from: self.from,
            destination: self.destination,
            trigger: self.trigger,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
    enum TestState {
        #[default]
        A,
        B,
        C,
    }

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
}
