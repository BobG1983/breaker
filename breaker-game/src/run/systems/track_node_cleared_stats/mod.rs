//! System to record node-level stats and detect highlights on node clear.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::track_node_cleared_stats;
