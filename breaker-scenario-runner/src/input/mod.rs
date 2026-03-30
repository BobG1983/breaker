//! Input strategies — [`ChaosDriver`], [`ScriptedInput`], [`HybridInput`], and [`InputDriver`].
//!
//! All strategies are pure Rust — no Bevy types. They produce `Vec<GameAction>`
//! for a given frame number. The Bevy integration (injecting into `InputActions`)
//! lives in the lifecycle module.

pub(crate) mod drivers;

#[cfg(test)]
mod tests;

pub use drivers::{ChaosDriver, HybridInput, InputDriver, PerfectDriver, ScriptedInput};
