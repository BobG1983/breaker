//! Bump system — input, timing grades, velocity modifiers.

mod system;

pub use system::perfect_bump_dash_cancel;
pub(crate) use system::{forward_grade, grade_bump, retroactive_grade, update_bump};

#[cfg(test)]
mod tests;
