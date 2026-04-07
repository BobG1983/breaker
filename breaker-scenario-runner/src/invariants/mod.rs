//! Invariant checker systems and `ViolationLog` resource.
//!
//! Invariant systems run in `FixedUpdate` after gameplay. They query game state
//! and append to a [`ViolationLog`] resource. They never panic — they collect
//! all violations for end-of-run reporting.

mod checkers;
pub mod screenshot;
mod types;

pub use checkers::*;
pub use screenshot::*;
pub use types::*;
