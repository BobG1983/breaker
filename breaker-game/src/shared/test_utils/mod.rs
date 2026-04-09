//! Shared test infrastructure: `TestAppBuilder`, `MessageCollector`, and `tick()`.
//!
//! This module provides composable building blocks for Bevy ECS integration tests.
//! All types are `#[cfg(test)]` and `pub(crate)` — available to any domain's tests.

pub(crate) mod builder;
pub(crate) mod collector;
pub(crate) mod tick_helper;

pub(crate) use builder::TestAppBuilder;
pub(crate) use collector::MessageCollector;
pub(crate) use tick_helper::tick;

#[cfg(test)]
mod tests;
