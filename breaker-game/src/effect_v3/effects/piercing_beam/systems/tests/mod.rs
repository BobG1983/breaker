//! W7 behavioral tests for the `PiercingBeam` emitter request → consumer
//! pipeline. Mirrors `explode/systems/tests/` — see
//! `.claude/specs/w7-fireable-damage-refactor-tests.md` for the spec.

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
