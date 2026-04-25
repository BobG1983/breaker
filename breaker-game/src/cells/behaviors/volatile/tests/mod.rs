//! Integration tests for Volatile's end-to-end explosion at death, chain
//! reactions, and safety/idempotency claims.
//!
//! Split by behavior per `.claude/rules/file-splitting.md` (Strategy C — the
//! original `tests.rs` exceeded 800 lines). Helpers are shared via
//! `helpers.rs`.

mod helpers;

mod chain_reactions;
mod explosion_at_death;
mod safety_idempotency;
