//! Tests for the Renewal hazard retrofit to the unified heal pipeline.
//!
//! Groups mirror the Cascade retrofit's layout:
//! - A: `duration_secs` formula (pure unit)
//! - B: `renewal_attach_timers` idempotence
//! - C: `renewal_tick` — timer decrements, no expiry
//! - D: `renewal_tick` — timer expiry emits `HealDealt<Cell>`
//! - E: per-message invariants (`cap`, `source`, `healer`, `amount`)
//! - F: `wire` — scheduling, run-condition gates
//! - G: end-to-end pipeline with `apply_heal::<Cell>`
//! - H: `activate` / config lifecycle
//! - I: retrofit-specific regression guards

mod helpers;

mod activate;
mod attach;
mod expiry;
mod formulas;
mod integration;
mod message;
mod regression;
mod tick;
mod wire;
