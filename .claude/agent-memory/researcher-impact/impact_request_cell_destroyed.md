---
name: impact_request_cell_destroyed
description: Full reference map for RequestCellDestroyed message (2026-03-28)
type: project
---

# Impact Map: RequestCellDestroyed

Mapped 2026-03-28. Adding `position: Vec2` and `was_required_to_clear: bool` fields.

## Definition
- `breaker-game/src/cells/messages.rs:10` — struct definition, `#[derive(Message, Clone, Debug)]`, single field `cell: Entity`

## Message Registration (configures)
- `breaker-game/src/cells/plugin.rs:26` — `app.add_message::<RequestCellDestroyed>()`

## Producers (sends)
- `breaker-game/src/cells/systems/handle_cell_hit/system.rs:49` — `request_destroyed_writer.write(RequestCellDestroyed { cell: msg.cell })` in `handle_cell_hit`

## Consumers (receives)
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:11` — `MessageReader<RequestCellDestroyed>` in `cleanup_destroyed_cells`; iterates `msg.cell` to despawn
- `breaker-game/src/effect/triggers/death.rs:6-11` — placeholder `bridge_death` (Wave 8 stub — does NOT yet read RequestCellDestroyed; comment says "message reading wired in Wave 8")

## Documentation (documents)
- `breaker-game/docs/architecture/plugins.md:98` — lists `RequestCellDestroyed` as read by effect bridges in the Cross-Domain Read Access section
- `breaker-game/docs/architecture/plugins.md:215` — describes `bridge_cell_death` reading `RequestCellDestroyed` while entity is still alive (target design, not yet implemented)
- `breaker-game/docs/architecture/messages.md:22` — stale: still lists `CellDestroyed` not `RequestCellDestroyed`; mentions bridge_cell_death evaluating via RequestCellDestroyed parenthetically

## Tests (construct)

### messages.rs tests
- `breaker-game/src/cells/messages.rs:68` — `request_cell_destroyed_debug_format` constructs `RequestCellDestroyed { cell: Entity::PLACEHOLDER }`

### cleanup_destroyed_cells.rs tests
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:70` — `cleanup_destroyed_cells_despawns_cell` constructs `RequestCellDestroyed { cell }`
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:28-38` — test helper `enqueue_request` uses `MessageWriter<RequestCellDestroyed>`; `SendRequestCellDestroyed` resource wraps `Option<RequestCellDestroyed>`
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:44` — `test_app()` calls `add_message::<RequestCellDestroyed>()`
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:81-94` — `cleanup_destroyed_cells_noop_without_message` (no RequestCellDestroyed constructed)

### handle_cell_hit/tests.rs tests (all in `src/cells/systems/handle_cell_hit/tests.rs`)
- line 16 — `CapturedDestroyed(Vec<RequestCellDestroyed>)` resource
- line 31-37 — `capture_destroyed` system uses `MessageReader<RequestCellDestroyed>`
- line 45 — `test_app()` calls `add_message::<RequestCellDestroyed>()`
- line 157-168 — `damage_cell_10_destroys_10hp_cell` asserts on `captured.0[0].cell`
- line 171-197 — `damage_cell_overkill_15_on_10hp_cell_destroys` checks `captured.0.len()`
- line 256-291 — `damage_cell_zero_does_not_change_health` asserts no RequestCellDestroyed sent
- line 323-353 — `destroyed_non_required_cell_sends_request_cell_destroyed` asserts `captured.0[0].cell`
- line 357-392 — `double_damage_cell_same_cell_only_one_request_cell_destroyed` dedup test
- line 399-420 — `CapturedRequestCellDestroyed` resource and `test_app_two_phase()` with `add_message::<RequestCellDestroyed>()`
- line 402-408 — `capture_request_cell_destroyed` helper uses `MessageReader<crate::cells::messages::RequestCellDestroyed>`
- line 416-418 — second `test_app_two_phase()` adds message
- line 422-458 — `handle_cell_hit_writes_request_cell_destroyed_instead_of_despawning` asserts `captured.0[0].cell`
- line 460-494 — `handle_cell_hit_dedup_produces_one_request_cell_destroyed`
- line 496-522 — `handle_cell_hit_non_required_cell_produces_request_cell_destroyed`
- line 597-625 — `damage_cell_for_despawned_entity_is_silently_skipped` asserts no RequestCellDestroyed

### check_lock_release/tests.rs tests (all in `src/cells/systems/check_lock_release/tests.rs`)
- line 60-73 — `hit_app()` registers `add_message::<RequestCellDestroyed>()`
- line 258-268 — `CapturedRequestCellDestroyed` resource, `capture_request_cell_destroyed` reader
- line 270-300 — `unlocked_cell_takes_damage_and_sends_request_destroyed` asserts `captured.0[0].cell`

## Field Change Impact for adding `position: Vec2` and `was_required_to_clear: bool`

Files that MUST be updated (struct construction):
1. `breaker-game/src/cells/messages.rs:68` — test constructs with `Entity::PLACEHOLDER` only
2. `breaker-game/src/cells/systems/handle_cell_hit/system.rs:49` — only producer, must supply both new fields
3. `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:70` — test constructs the struct
4. `breaker-game/src/cells/systems/handle_cell_hit/tests.rs` (lines 157, 349, 442, 449, etc.) — tests that check `captured.0[0].cell` may need to also verify new fields, or just compile (struct construction in test helpers at line 70 and line 278)

Files that read fields (must verify no breakage):
- `breaker-game/src/cells/systems/cleanup_destroyed_cells.rs:15` — only accesses `msg.cell`, `position` and `was_required_to_clear` are ignored (safe)
- All test `capture_*` helpers only push the cloned message — safe

Note: `was_required_to_clear` on `RequestCellDestroyed` is separate from the same field on `CellDestroyedAt`. The `bridge_cell_death` system (not yet implemented) will be the one that reads both from `RequestCellDestroyed` and writes `CellDestroyedAt` — so the `was_required_to_clear` field needs to be on `RequestCellDestroyed` for that bridge to use it.

## Scenario Runner
No references to RequestCellDestroyed in breaker-scenario-runner/src/.

## RON Data / Scenarios
No references to RequestCellDestroyed in any .ron files.
