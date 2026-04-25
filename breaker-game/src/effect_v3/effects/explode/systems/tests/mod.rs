//! W7 behavioral tests for the Explode emitter request → consumer pipeline.
//!
//! Behaviors 1, 5 (request-only / source propagation), and 2, 3, 4, 6, 7, 8,
//! 10, 11 (pipeline + scheduling + robustness) are all wired here, grouped by
//! concern. See `.claude/specs/w7-fireable-damage-refactor-tests.md`.

mod helpers;

mod boost_once;
mod consumer_emits_raw;
mod despawned_dealer;
mod end_to_end_no_boost;
mod fire_writes_request_only;
mod invulnerable_pass_through;
mod multi_request_per_tick;
mod no_cells;
mod scheduling;
mod source_propagation;
