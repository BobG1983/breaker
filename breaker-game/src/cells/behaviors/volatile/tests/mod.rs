//! Groups D, E, F — integration tests for Volatile's end-to-end explosion at
//! death, chain reactions, and safety/idempotency claims.
//!
//! Split by group per `.claude/rules/file-splitting.md` (Strategy C — the
//! original `tests.rs` exceeded 800 lines). Helpers are shared via
//! `helpers.rs`.

mod helpers;

mod group_d;
mod group_e;
mod group_f;
