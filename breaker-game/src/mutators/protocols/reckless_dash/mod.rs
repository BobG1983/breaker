//! Reckless Dash protocol — risky-zone damage boost with bolt-loss penalty
//! doubling on dash transitions.
//!
//! Design doc: `docs/design/protocols/reckless_dash.md`.
//!
//! Owns the `RecklessDashConfig` resource (per-run tuning), the
//! `RiskyDamageBoost` per-bolt component, the `OriginalBoltLossBehavior`
//! per-breaker overlay component, the four runtime systems
//! (`reckless_dash_on_bump`, `reckless_dash_amplify_damage`,
//! `reckless_dash_on_dash_transition`, `reckless_dash_cleanup_node`), the
//! `RECKLESS_DASH_SENTINEL` source-chip tag, and the `activate` / `wire`
//! dispatch entry points.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::RiskyDamageBoost;
pub(crate) use system::{activate, wire};
