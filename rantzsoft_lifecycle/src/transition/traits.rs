//! Marker traits for transition effects.
//!
//! These traits enforce compile-time guarantees about which effect types
//! can be used in each [`TransitionType`](super::types::TransitionType) variant.

use std::any::Any;

use bevy::ecs::world::World;

use super::resources::StartingTransition;

/// Base marker trait for all transition effects.
///
/// Requires `Any + Send + Sync + 'static` so that:
/// - `Any` enables `TypeId` extraction from trait objects
/// - `Send + Sync` allows storage in Bevy resources
/// - `'static` is required by Bevy's type system
///
/// A type that is `Any + Send + Sync + 'static` and implements `Transition`
/// can be used in `TransitionType`. This is a compile-time guarantee verified
/// by the type system.
///
/// The `#[doc(hidden)]` lifecycle methods have default implementations that
/// manage marker resources. They are called through vtable dispatch on trait
/// objects by the orchestration system.
pub trait Transition: Any + Send + Sync {
    /// Insert `StartingTransition<Self>` into the world.
    ///
    /// Called through vtable dispatch on trait objects by the orchestration
    /// system. The default implementation inserts the correct generic marker
    /// resource using the concrete type known at monomorphization time.
    #[doc(hidden)]
    fn insert_starting(&self, world: &mut World) {
        world.insert_resource(StartingTransition::<Self>::new());
    }
}

/// Marker for effects that play when revealing new content (e.g., fade-in).
///
/// Requires `Transition` as a supertrait. A type that does NOT implement
/// `Transition` will not compile as `InTransition` -- this is a compile-time
/// guarantee, not a runtime test.
pub trait InTransition: Transition {}

/// Marker for effects that play when hiding current content (e.g., fade-out).
///
/// Requires `Transition` as a supertrait. A type that does NOT implement
/// `Transition` will not compile as `OutTransition` -- this is a compile-time
/// guarantee, not a runtime test.
pub trait OutTransition: Transition {}

/// Marker for effects that play over both old and new content simultaneously.
///
/// Requires `Transition` as a supertrait. A type that does NOT implement
/// `Transition` will not compile as `OneShotTransition` -- this is a
/// compile-time guarantee, not a runtime test.
pub trait OneShotTransition: Transition {}
