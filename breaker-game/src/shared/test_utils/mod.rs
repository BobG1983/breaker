//! Shared test infrastructure: `TestAppBuilder`, `MessageCollector`, and `tick()`.
//!
//! This module provides composable building blocks for Bevy ECS integration tests.
//! All types are `#[cfg(test)]` and `pub(crate)` — available to any domain's tests.

pub(crate) mod builder;
pub(crate) mod collector;
pub(crate) mod effect_v3_infra;
pub(crate) mod input;
pub(crate) mod pregate_drain;
pub(crate) mod schedule_inspect;
pub(crate) mod tick_helper;

pub(crate) use builder::{TestAppBuilder, add_breaker_transition_systems};
pub(crate) use collector::{MessageCollector, attach_message_capture};
pub(crate) use effect_v3_infra::register_effect_v3_test_infrastructure;
pub(crate) use input::press_key;
pub(crate) use schedule_inspect::system_in_set;
pub(crate) use tick_helper::tick;

#[cfg(test)]
mod tests;
