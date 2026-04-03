//! Run domain — run state, seeded node sequencing, timer, difficulty scaling.

pub mod chip_select;
pub(crate) mod components;
pub mod definition;
pub(crate) mod loading;
pub mod messages;
pub mod node;
mod plugin;
pub mod resources;
pub(crate) mod run_end;
pub(crate) mod systems;

pub use definition::HighlightConfig;
pub use node::{NodeLayout, NodeLayoutRegistry};
pub use plugin::RunPlugin;
pub use resources::{
    HighlightCategory, HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats,
};
