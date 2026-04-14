//! Integration tests for the `SpawnStampRegistry` watcher systems.
//!
//! Behaviors are grouped by concern:
//! - [`bolt`]: canonical per-behavior coverage via the bolt watcher (behaviors 1-11).
//! - [`cells_walls_breakers`]: per-kind sanity checks (behaviors 12-14).
//! - [`cross_kind`]: all four watchers registered together (behaviors 15-16).
//! - [`ordering`]: Bridge → Tick set ordering (behavior 17).

mod helpers;

mod bolt;
mod cells_walls_breakers;
mod cross_kind;
mod ordering;
