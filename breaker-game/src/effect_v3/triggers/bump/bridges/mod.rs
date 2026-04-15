//! Bump trigger bridges.
pub(crate) mod system;

#[cfg(test)]
mod tests;

pub use system::{
    on_bump_occurred, on_bump_whiff_occurred, on_bumped, on_early_bump_occurred, on_early_bumped,
    on_late_bump_occurred, on_late_bumped, on_no_bump_occurred, on_perfect_bump_occurred,
    on_perfect_bumped,
};
