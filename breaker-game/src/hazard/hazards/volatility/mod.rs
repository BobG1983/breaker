//! Volatility hazard — interval-based cell recovery.
//!
//! Cells that are left untouched regrow HP in discrete interval ticks. Each
//! interval emits a `HealDealt<Cell>` (capped via `HealCap::Max` against a
//! lifted `Hp.max = 2× starting`) so growth rides the unified heal pipeline.
//! Any `DamageDealt<Cell>` resets the cell's timer to zero — a cell "touched"
//! within the interval stays at its current HP. Effective interval shrinks
//! with hazard stacks (floored at 1.0s). Authoritative design doc:
//! `docs/todos/detail/mod-system-design/hazards/volatility.md`.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
