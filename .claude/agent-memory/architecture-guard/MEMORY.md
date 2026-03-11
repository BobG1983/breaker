# Architecture Guard Memory

## Project State
- Phase 0 scaffolding complete, reviewed 2026-03-10
- Bevy 0.18.1, bevy_egui 0.39, edition 2024
- Single crate, plugin-per-domain, message-driven decoupling

## Key Patterns Confirmed
- Messages defined in sending domain's `messages.rs`, registered via `app.add_message::<T>()` in owning plugin
- `shared.rs` has passive types only: GameState, PlayingState, cleanup markers, playfield constants
- `game.rs` is the ONLY file that imports all plugin structs
- `screen/` owns state registration (init_state, add_sub_state) and cleanup systems
- Debug plugin gated behind `#[cfg(feature = "dev")]` inside `build()`, struct always compiled
- All 8 messages from architecture table are implemented and registered
- lib.rs visibility correct: pub for app/game/shared, pub(crate) for all domain modules
- proptest dev-dependency is present in Cargo.toml

## Open Issues (Phase 0)
- V1: ARCHITECTURE.md file tree shows assets/ under src/ but actual location is project root (doc bug)
- N2/N3: "upgrades" module and "UpgradeSelected" message use generic term; TERMINOLOGY.md forbids it but ARCHITECTURE.md uses it. Docs contradict each other.
- N5: Redundant run_if(resource_exists::<DebugOverlays>) in debug/mod.rs
- I3: Dev-feature path of DebugPlugin not testable headlessly

## Message Inventory
See [message-inventory.md](message-inventory.md) for full table.

## Test Pattern
- Every plugin has a `plugin_builds` headless test (except DebugPlugin which requires render context)
- `app.rs` headless test disables DebugPlugin via `.disable::<DebugPlugin>()`
- Tests are in-module `#[cfg(test)]` blocks
- DebugPlugin has `plugin_builds_headless` test gated behind `#[cfg(not(feature = "dev"))]`
