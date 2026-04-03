//! Node subdomain resources — registry, active layout, timer, and completion tracking.

pub(crate) mod definitions;

#[cfg(test)]
mod tests;

pub use definitions::{
    ActiveNodeLayout, ClearRemainingCount, NodeLayoutRegistry, NodeTimer, ScenarioLayoutOverride,
};
