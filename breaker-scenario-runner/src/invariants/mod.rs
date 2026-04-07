//! Invariant checker systems and `ViolationLog` resource.
//!
//! Invariant systems run in `FixedUpdate` after gameplay. They query game state
//! and append to a [`ViolationLog`] resource. They never panic — they collect
//! all violations for end-of-run reporting.

mod checkers;
/// First-failure screenshot tracking: detects new invariant violations,
/// formats screenshot output paths, and tracks which kinds have been captured.
pub mod screenshot;
mod types;

pub use checkers::*;
pub use screenshot::*;
pub use types::*;
