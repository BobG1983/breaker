//! Debt Collector protocol — bump-stack damage multiplier.
//!
//! Design doc: `docs/design/protocols/debt_collector.md`.
//!
//! Owns the `DebtCollectorConfig` resource (per-run tuning), the `DebtStack`
//! and `DebtCashOut` per-bolt components, the five runtime systems
//! (`debt_collector_on_bump`, `debt_collector_on_impact`,
//! `debt_collector_on_bolt_lost`, `debt_collector_attach_stack`,
//! `debt_collector_cleanup_node`), the `DEBT_COLLECTOR_SENTINEL` source-chip
//! tag, and the `activate` / `wire` dispatch entry points.

pub mod system;

#[cfg(test)]
mod tests;

pub use system::{DebtCashOut, DebtStack};
pub(crate) use system::{activate, wire};
