//! Declarative state routing, screen transitions, and lifecycle messages
//! for Bevy 0.18 games.
//!
//! Provides:
//! - [`Route`] builder API for declaring state-to-state transitions
//! - [`RoutingTable<S>`] resource per state type
//! - Message-triggered and condition-triggered dispatch systems
//! - [`CleanupOnExit<S>`] component for state-scoped entity cleanup
//! - [`ChangeState<S>`] / [`StateChanged<S>`] lifecycle messages
//!
//! Zero game knowledge — this crate contains no game-specific types,
//! vocabulary, or configuration.

pub mod cleanup;
pub mod dispatch;
pub mod messages;
mod plugin;
pub mod route;
pub mod routing_table;

pub use cleanup::CleanupOnExit;
pub use messages::{ChangeState, StateChanged};
pub use plugin::RantzLifecyclePlugin;
pub use route::Route;
pub use routing_table::{RoutingTable, RoutingTableAppExt};
