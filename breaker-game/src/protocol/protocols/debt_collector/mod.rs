//! Debt Collector protocol — bump-stack damage multiplier.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/debt_collector.md`.
//!
//! Owns the `DebtCollectorConfig` resource (per-run tuning), the `DebtStack`
//! and `DebtCashOut` per-bolt components, the five runtime systems
//! (`debt_collector_on_bump`, `debt_collector_on_impact`,
//! `debt_collector_on_bolt_lost`, `debt_collector_attach_stack`,
//! `debt_collector_cleanup_node`), the `DEBT_COLLECTOR_SENTINEL` source-chip
//! tag, and the `activate` / `register` dispatch entry points.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
