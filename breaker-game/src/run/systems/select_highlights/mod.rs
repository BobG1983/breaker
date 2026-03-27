//! Highlight scoring and diversity-penalized selection.
//!
//! Pure functions (not Bevy systems) for scoring highlights and selecting
//! the most impressive/diverse subset for run-end display.

mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::select_highlights;
