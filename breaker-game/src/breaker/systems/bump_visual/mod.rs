//! Bump visual feedback — eased upward pop animation on the breaker.

mod system;

#[cfg(test)]
mod tests;

#[cfg(test)]
use system::bump_offset;
pub use system::{animate_bump_visual, trigger_bump_visual};
