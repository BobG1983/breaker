//! Procedural node sequence generation from difficulty curve.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::generate_node_sequence_system;
