//! Run domain plugin — run state, seeded node sequencing, timer, difficulty scaling.

pub(crate) mod components;
pub mod definition;
pub(crate) mod highlights;
pub mod messages;
pub mod node;
mod plugin;
pub mod resources;
pub mod systems;

pub use definition::HighlightConfig;
pub use node::{NodeLayout, NodeLayoutRegistry};
pub use plugin::RunPlugin;
pub use resources::{
    HighlightCategory, HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats,
};
