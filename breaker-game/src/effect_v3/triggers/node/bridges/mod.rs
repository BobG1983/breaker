//! Node trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{on_node_end_occurred, on_node_start_occurred, on_node_timer_threshold_occurred};
