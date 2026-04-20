//! Burnout protocol — heat-gauge rhythm mechanic.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/burnout.md`.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
