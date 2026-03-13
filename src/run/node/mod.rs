//! Node subdomain — layout definitions, registry, active layout, timer, and completion tracking.

pub mod resources;
pub mod sets;
pub mod systems;

pub use resources::{
    ActiveNodeLayout, ClearRemainingCount, NodeLayout, NodeLayoutRegistry, NodeTimer,
};
pub use sets::NodeSystems;
