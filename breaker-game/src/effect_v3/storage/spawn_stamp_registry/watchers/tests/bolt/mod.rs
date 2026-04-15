//! Behaviors 1-11 — canonical coverage of the bolt watcher.
//!
//! The bolt watcher is the reference implementation. Cells/walls/breakers
//! share the same code path and are exercised in `cells_walls_breakers.rs`.

mod empty_and_filtering;
mod idempotency;
mod insert_and_append;
mod multi_entity;
