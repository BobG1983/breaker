//! Bump visual feedback — eased upward pop animation on the breaker.

mod system;

#[cfg(test)]
mod tests;

pub use system::{animate_bump_visual, trigger_bump_visual};
#[cfg(test)]
use system::bump_offset;
