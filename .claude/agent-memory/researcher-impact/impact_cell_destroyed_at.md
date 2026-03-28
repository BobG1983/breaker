---
name: impact_cell_destroyed_at
description: Full impact map for CellDestroyedAt — definition, registration, producers (none live), consumers, tests (2026-03-28)
type: project
---

Full reference map for `CellDestroyedAt`. As of 2026-03-28.

## Definition

`breaker-game/src/cells/messages.rs:19`
```rust
pub(crate) struct CellDestroyedAt {
    pub was_required_to_clear: bool,
}
```
Derives: `Message`, `Clone`, `Debug`

## Registration (add_message)

- `breaker-game/src/cells/plugin.rs:27` — `CellsPlugin::build` calls `app.add_message::<CellDestroyedAt>()`
- `breaker-game/src/run/node/plugin.rs:28` — `NodePlugin::build` calls `app.add_message::<crate::cells::messages::CellDestroyedAt>()`

## Producers (MessageWriter<CellDestroyedAt>)

**NONE IN LIVE CODE.** The doc comment in `messages.rs` says it is "sent by `bridge_cell_death`", but that system is a placeholder in `breaker-game/src/effect/triggers/death.rs` — no actual write occurs. `CellDestroyedAt` has NO live sender.

## Consumers (MessageReader<CellDestroyedAt>)

All in `FixedUpdate`, `PlayingState::Active`:

1. `breaker-game/src/cells/systems/check_lock_release/system.rs:17`
   — reads to check if adjacents were destroyed; removes `Locked` when all gone
   — system: `check_lock_release`

2. `breaker-game/src/run/node/systems/track_node_completion.rs:24`
   — reads `was_required_to_clear`, decrements `ClearRemainingCount`, sends `NodeCleared` at zero
   — system: `track_node_completion`

3. `breaker-game/src/run/systems/track_cells_destroyed.rs:13`
   — increments `RunStats::cells_destroyed` for each message
   — system: `track_cells_destroyed`

4. `breaker-game/src/run/highlights/systems/detect_mass_destruction.rs:28`
   — records timestamp per message, detects `MassDestruction` highlight
   — system: `detect_mass_destruction`

5. `breaker-game/src/run/highlights/systems/detect_combo_king.rs:26`
   — increments `cells_since_last_breaker_hit` tracker
   — system: `detect_combo_king`

## Tests Constructing CellDestroyedAt

All are test helpers / fixture builders (not production senders):

- `breaker-game/src/cells/messages.rs:78` — `cell_destroyed_at_debug_format` (debug assertion)
- `breaker-game/src/cells/messages.rs:87` — `cell_destroyed_at_non_required` (field check)
- `breaker-game/src/cells/systems/check_lock_release/tests.rs:197` — `lock_releases_when_all_adjacents_destroyed`
- `breaker-game/src/cells/systems/check_lock_release/tests.rs:241` — `lock_stays_locked_when_only_some_adjacents_destroyed`
- `breaker-game/src/cells/systems/check_lock_release/tests.rs:358` — `check_lock_release_reads_cell_destroyed_at`
- `breaker-game/src/run/node/systems/track_node_completion.rs:87` — `decrement_on_required_destroyed`
- `breaker-game/src/run/node/systems/track_node_completion.rs:97` — `ignore_non_required_destroyed`
- `breaker-game/src/run/node/systems/track_node_completion.rs:116` — `node_cleared_fires_when_remaining_hits_zero`
- `breaker-game/src/run/node/systems/track_node_completion.rs:186` — `track_node_completion_reads_cell_destroyed_at_and_decrements`
- `breaker-game/src/run/node/systems/track_node_completion.rs:200` — `track_node_completion_ignores_non_required_cell_destroyed_at`
- `breaker-game/src/run/node/systems/track_node_completion.rs:216` — `node_cleared_fires_on_cell_destroyed_at_reaching_zero`
- `breaker-game/src/run/systems/track_cells_destroyed.rs:88` — `track_cells_destroyed_reads_cell_destroyed_at`
- `breaker-game/src/run/systems/track_cells_destroyed.rs:108` — `increments_cells_destroyed_for_each_message`
- `breaker-game/src/run/highlights/systems/detect_mass_destruction.rs:152` — `detect_mass_destruction_reads_cell_destroyed_at`
- `breaker-game/src/run/highlights/systems/detect_mass_destruction.rs:206` — `mass_destruction_detected_with_10_cells_in_window`
- `breaker-game/src/run/highlights/systems/detect_mass_destruction.rs:268` — `old_timestamps_pruned_outside_window` (direct construction)
- `breaker-game/src/run/highlights/systems/detect_combo_king.rs:157` — `detect_combo_reads_cell_destroyed_at`
- `breaker-game/src/run/highlights/systems/detect_combo_king.rs:203` — `cell_destroyed_increments_cells_since_last_breaker_hit`

## Impact of Adding `position: Vec2` Field

Adding `pub position: Vec2` to `CellDestroyedAt` is a **struct literal breaking change**. Every construction site must supply the new field.

**Files requiring update:**
1. `breaker-game/src/cells/messages.rs` — 3 test constructions (lines 78, 87, + inline)
2. `breaker-game/src/cells/systems/check_lock_release/tests.rs` — multiple constructions
3. `breaker-game/src/run/node/systems/track_node_completion.rs` — multiple constructions
4. `breaker-game/src/run/systems/track_cells_destroyed.rs` — multiple constructions
5. `breaker-game/src/run/highlights/systems/detect_mass_destruction.rs` — multiple constructions
6. `breaker-game/src/run/highlights/systems/detect_combo_king.rs` — multiple constructions

**Consumer changes needed:**
None of the 5 consumers use `was_required_to_clear` or any field from the message for field-specific logic EXCEPT:
- `track_node_completion` reads `msg.was_required_to_clear` — no change needed there
- The others ignore all fields entirely (use `_msg`)

**No live producer exists**, so no producer update is needed — the sender will be written as part of implementing `bridge_cell_death`.

## Stale Documentation

`docs/architecture/messages.md:22` references `CellDestroyed` (old name) — should be updated to `CellDestroyedAt`.
