//! Group E — Wave A integration tests for `DamageBoostStack` source filter.
//!
//! End-to-end pipeline coverage: a dealer with a populated
//! `DamageBoostStack` (filtered or filterless) emits `DamageDealt<TestT>`
//! messages with various `source` values, ticks the app, and asserts on
//! the victim's resulting `Hp`.
//!
//! Also smoke-checks the `entry_applies` re-export from the crate root.

mod helpers;

mod entry_applies_smoke;
mod one_shots;
mod persistent;
