//! Iron Curtain protocol — bolt-lost damage wave.
//!
//! Design doc: `docs/design/protocols/iron_curtain.md`.
//!
//! Owns the `IronCurtainConfig` resource (per-run tuning), the single runtime
//! system (`iron_curtain_on_bolt_lost`), the `IRON_CURTAIN_SENTINEL`
//! source-chip tag stamped on emitted `DamageDealt<Cell>` messages, and the
//! `activate` / `register` dispatch entry points.

pub(crate) mod system;

#[cfg(test)]
mod tests;

pub(crate) use system::{activate, register};
