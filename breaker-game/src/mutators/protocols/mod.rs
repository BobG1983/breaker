//! Protocol sub-domain — run-long positive upgrades that change how the
//! player plays. Selected at chip-select time alongside chips.
//!
//! Each per-protocol module under this path owns one custom-system
//! protocol's config resource, activation logic, and runtime system(s).
//! Effect-tree protocols (Deadline, Ricochet, Anchor, Kickstart) have no
//! module here — their behaviour comes from the effect tree stamped by
//! `dispatch_protocol_selection`.

pub mod definition;
pub(crate) mod messages;
pub mod resources;
pub(crate) mod systems;

pub(crate) mod fan_out;

pub(crate) mod afterimage;
pub mod burnout;
pub(crate) mod conductor;
pub mod debt_collector;
pub mod echo_strike;
pub mod fission;
pub mod greed;
pub(crate) mod iron_curtain;
pub mod reckless_dash;
pub mod siphon;
pub(crate) mod tier_regression;

#[cfg(test)]
pub(crate) mod test_utils;

pub use fan_out::activate_from_registry;
pub(crate) use fan_out::{activate, wire};
