//! Bump system — input, timing grades, velocity modifiers.

mod system;

pub use system::perfect_bump_dash_cancel;
pub(crate) use system::{grade_bump, update_bump};

#[cfg(test)]
mod tests;
