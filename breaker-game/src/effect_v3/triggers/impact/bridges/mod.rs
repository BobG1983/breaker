//! Impact trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{on_impact_occurred, on_impacted};
