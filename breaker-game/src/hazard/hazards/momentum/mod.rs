//! Momentum hazard — non-lethal damage bolsters cell HP; cells split at 2x starting HP.
//!
//! Design doc: `docs/todos/detail/mod-system-design/hazards/momentum.md`.
//!
//! Scaffold-level stubs — production logic lives in `system.rs`. Tests live in
//! `tests/` and are written to fail against the stubs.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
