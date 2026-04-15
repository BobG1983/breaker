//! Walk effects — tree traversal for bound and staged effects.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(in crate::effect_v3) use system::{evaluate_tree, walk_bound_effects, walk_staged_effects};
