//! Bump system — input, timing grades, velocity modifiers.

mod system;

pub use system::perfect_bump_dash_cancel;
// Re-export for tests — child modules can see private `use` items.
#[cfg(test)]
use system::{forward_grade, retroactive_grade};
pub(crate) use system::{grade_bump, update_bump};

#[cfg(test)]
mod tests;
