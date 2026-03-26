//! Typed per-effect observer events and dispatch helpers.
//!
//! Each event struct now lives in its corresponding effect handler file.
//! This module re-exports them for backward compatibility and contains
//! the dispatch helpers that convert `Effect` values into typed events.

mod events;

pub(crate) use events::*;

#[cfg(test)]
mod tests;
