//! Transition infrastructure — effect-driven screen transitions.

pub mod effects;
pub(crate) mod messages;
pub(crate) mod orchestration;
pub mod registry;
pub mod resources;
pub mod traits;
pub mod types;

// Public re-exports
pub use registry::TransitionRegistry;
pub use resources::{ActiveTransition, EndingTransition, RunningTransition, StartingTransition};
pub use traits::{InTransition, OneShotTransition, OutTransition, Transition};
pub use types::TransitionType;
