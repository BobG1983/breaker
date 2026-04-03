//! Node subdomain — layout definitions, registry, active layout, timer, completion tracking, HUD.

pub mod definition;
pub(crate) mod highlights;
pub(crate) mod hud;
pub(crate) mod lifecycle;
pub mod messages;
mod plugin;
pub mod resources;
pub mod sets;
pub mod systems;
pub(crate) mod tracking;

pub use definition::NodeLayout;
pub use plugin::NodePlugin;
pub use resources::{
    ActiveNodeLayout, ClearRemainingCount, NodeLayoutRegistry, NodeTimer, ScenarioLayoutOverride,
};
pub use sets::NodeSystems;
