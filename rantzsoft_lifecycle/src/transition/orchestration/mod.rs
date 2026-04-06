pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{begin_transition, orchestrate_transitions};
