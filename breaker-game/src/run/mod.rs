//! Run domain plugin — run state, seeded node sequencing, timer, difficulty scaling.

pub mod definition;
pub mod messages;
pub mod node;
mod plugin;
pub mod resources;
pub mod systems;

pub use node::{NodeLayout, NodeLayoutRegistry};
pub use plugin::RunPlugin;
pub use resources::{HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats};
