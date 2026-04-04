//! Marker resources for transition phase gating.
//!
//! Each phase of a transition (Starting, Running, Ending) has a generic
//! marker resource parameterized by the effect type. Effect systems use
//! `run_if(resource_exists::<StartingTransition<MyEffect>>)` to gate
//! execution.

use std::marker::PhantomData;

use bevy::prelude::*;

use super::traits::Transition;

/// Inserted when a transition effect should begin its start phase.
///
/// Removed by the orchestration system when `TransitionReady` is received.
///
/// Accepts `T: ?Sized` to allow creation from trait object vtable dispatch
/// (the `Transition::insert_starting` default method). At runtime, `T` is
/// always a concrete `Sized` type.
pub struct StartingTransition<T: ?Sized + Transition> {
    _marker: PhantomData<Box<T>>,
}

impl<T: ?Sized + Transition> Resource for StartingTransition<T> {}

impl<T: ?Sized + Transition> StartingTransition<T> {
    /// Create a new `StartingTransition` marker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized + Transition> Default for StartingTransition<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Inserted when a transition effect should run its main phase.
///
/// Removed by the orchestration system when `TransitionRunComplete` is received.
pub struct RunningTransition<T: ?Sized + Transition> {
    _marker: PhantomData<Box<T>>,
}

impl<T: ?Sized + Transition> Resource for RunningTransition<T> {}

impl<T: ?Sized + Transition> RunningTransition<T> {
    /// Create a new `RunningTransition` marker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized + Transition> Default for RunningTransition<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Inserted when a transition effect should run its ending phase.
///
/// Removed by the orchestration system when `TransitionOver` is received.
pub struct EndingTransition<T: ?Sized + Transition> {
    _marker: PhantomData<Box<T>>,
}

impl<T: ?Sized + Transition> Resource for EndingTransition<T> {}

impl<T: ?Sized + Transition> EndingTransition<T> {
    /// Create a new `EndingTransition` marker.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: ?Sized + Transition> Default for EndingTransition<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Inserted when any transition is in progress.
///
/// Game code can use `resource_exists::<ActiveTransition>()` to gate
/// systems that should not run during transitions (e.g., pause screen).
///
/// Removed by the orchestration system when the transition lifecycle
/// completes.
#[derive(Resource)]
pub struct ActiveTransition;

/// Type-erased callback that operates on a Bevy [`World`](bevy::ecs::world::World).
pub(crate) type WorldCallback = Option<Box<dyn FnOnce(&mut bevy::ecs::world::World) + Send + Sync>>;

/// Stores the state change action to apply at the right moment during a
/// transition.
///
/// Type-erased: the closures capture the concrete state types. This allows
/// `orchestrate_transitions` (which doesn't know `S`) to call them.
/// Cleaned up at the end of the transition lifecycle alongside
/// `ActiveTransition`.
#[derive(Resource)]
pub(crate) struct PendingTransition {
    /// The current phase of the transition.
    pub(crate) phase: TransitionPhase,
    /// The `TypeId` of the currently active effect (for registry lookup
    /// during phase advances).
    pub(crate) current_effect_type_id: std::any::TypeId,
    /// Closure that applies `NextState<S>::set` and sends `StateChanged<S>`.
    /// Called once. `None` means the state change was already applied (`In`,
    /// `OneShot`, or `OutIn` after Out phase).
    pub(crate) apply_state_change: WorldCallback,
    /// Closure that sends `TransitionEnd<S>`. Called once at final
    /// completion.
    pub(crate) send_transition_end: WorldCallback,
    /// For `OutIn`: data about the In phase effect.
    pub(crate) out_in_in_effect: Option<OutInData>,
    /// For `OutIn`: which sub-phase we're in (Out or In).
    pub(crate) out_in_state: Option<OutInState>,
    /// Whether to unpause `Time<Virtual>` when the transition completes.
    pub(crate) unpause_at_end: bool,
}

/// Current phase within a transition lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransitionPhase {
    /// Effect start system is expected to fire.
    Starting,
    /// Effect run system is expected to fire.
    Running,
    /// Effect end system is expected to fire.
    Ending,
}

/// Data for the In phase of an `OutIn` transition.
pub(crate) struct OutInData {
    /// The `TypeId` of the In effect.
    pub(crate) in_effect_type_id: std::any::TypeId,
    /// The In effect trait object, stored so the orchestrator can call
    /// `insert_starting` on it when transitioning from Out to In phase.
    pub(crate) in_effect: std::sync::Arc<dyn super::traits::InTransition>,
}

/// Tracks whether we're in the Out or In phase of an `OutIn` transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OutInState {
    /// Currently running the Out effect.
    Out,
    /// Currently running the In effect.
    In,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transition::traits::{InTransition, OutTransition, Transition};

    struct TestEffectOut;
    impl Transition for TestEffectOut {}
    impl OutTransition for TestEffectOut {}

    struct TestEffectIn;
    impl Transition for TestEffectIn {}
    impl InTransition for TestEffectIn {}

    // --- Section C behavior 1: StartingTransition can be inserted and removed ---

    #[test]
    fn starting_transition_can_be_inserted_as_resource() {
        let mut world = World::new();
        world.insert_resource(StartingTransition::<TestEffectOut>::new());
        assert!(world.contains_resource::<StartingTransition<TestEffectOut>>());
    }

    #[test]
    fn starting_transition_different_type_parameter_is_independent() {
        let mut world = World::new();
        world.insert_resource(StartingTransition::<TestEffectOut>::new());
        assert!(!world.contains_resource::<StartingTransition<TestEffectIn>>());
    }

    // --- Section C behavior 2: RunningTransition can be inserted and removed ---

    #[test]
    fn running_transition_can_be_inserted_as_resource() {
        let mut world = World::new();
        world.insert_resource(RunningTransition::<TestEffectOut>::new());
        assert!(world.contains_resource::<RunningTransition<TestEffectOut>>());
    }

    #[test]
    fn running_transition_removal_causes_contains_to_return_false() {
        let mut world = World::new();
        world.insert_resource(RunningTransition::<TestEffectOut>::new());
        world.remove_resource::<RunningTransition<TestEffectOut>>();
        assert!(!world.contains_resource::<RunningTransition<TestEffectOut>>());
    }

    // --- Section C behavior 3: EndingTransition can be inserted and removed ---

    #[test]
    fn ending_transition_can_be_inserted_as_resource() {
        let mut world = World::new();
        world.insert_resource(EndingTransition::<TestEffectOut>::new());
        assert!(world.contains_resource::<EndingTransition<TestEffectOut>>());
    }

    #[test]
    fn ending_transition_removal_causes_contains_to_return_false() {
        let mut world = World::new();
        world.insert_resource(EndingTransition::<TestEffectOut>::new());
        world.remove_resource::<EndingTransition<TestEffectOut>>();
        assert!(!world.contains_resource::<EndingTransition<TestEffectOut>>());
    }

    // --- Section C behavior 4: ActiveTransition can be inserted and checked ---

    #[test]
    fn active_transition_can_be_inserted_and_checked() {
        let mut world = World::new();
        world.insert_resource(ActiveTransition);
        assert!(world.contains_resource::<ActiveTransition>());
    }

    #[test]
    fn active_transition_removal_causes_contains_to_return_false() {
        let mut world = World::new();
        world.insert_resource(ActiveTransition);
        world.remove_resource::<ActiveTransition>();
        assert!(!world.contains_resource::<ActiveTransition>());
    }

    // --- Section C behavior 5: Marker resources for different types coexist ---

    #[test]
    fn marker_resources_for_different_types_coexist_independently() {
        let mut world = World::new();
        world.insert_resource(StartingTransition::<TestEffectOut>::new());
        world.insert_resource(RunningTransition::<TestEffectIn>::new());
        assert!(world.contains_resource::<StartingTransition<TestEffectOut>>());
        assert!(world.contains_resource::<RunningTransition<TestEffectIn>>());
    }

    #[test]
    fn removing_one_marker_does_not_affect_another() {
        let mut world = World::new();
        world.insert_resource(StartingTransition::<TestEffectOut>::new());
        world.insert_resource(RunningTransition::<TestEffectIn>::new());

        world.remove_resource::<StartingTransition<TestEffectOut>>();

        assert!(!world.contains_resource::<StartingTransition<TestEffectOut>>());
        assert!(world.contains_resource::<RunningTransition<TestEffectIn>>());
    }
}
