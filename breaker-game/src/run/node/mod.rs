//! Node subdomain — layout definitions, registry, active layout, timer, and completion tracking.

pub mod messages;
mod plugin;
pub mod resources;
pub mod sets;
pub mod systems;

pub use plugin::NodePlugin;
pub use resources::{
    ActiveNodeLayout, ClearRemainingCount, NodeLayout, NodeLayoutRegistry, NodeTimer,
    ScenarioLayoutOverride,
};
pub use sets::NodeSystems;
