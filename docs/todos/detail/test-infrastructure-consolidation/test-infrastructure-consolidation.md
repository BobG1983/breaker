# Test Infrastructure Consolidation

## Summary

Replace 101 ad-hoc `test_app()` functions, 73 duplicate `tick()` copies, and 47 duplicate `spawn_in_world()` copies with a composable `TestAppBuilder` and domain-level test utils modules.

See `docs/architecture/testing.md` for the target convention (to be updated after implementation).

## Design Documents

| File | Contents |
|------|----------|
| [builder-api.md](builder-api.md) | Full API reference with behavioral specs for each method |
| [shortcuts.md](shortcuts.md) | Domain test_utils, when to extract vs inline, common patterns as builder calls |
| [migration-map.md](migration-map.md) | Every test_app() in the codebase, categorized by pattern, with target builder call |

## The Duplication (measured 2026-04-09)

| Helper | Copies | Notes |
|--------|--------|-------|
| `test_app()` | 101 | Each unique but share 80%+ of their setup |
| `tick(app)` | 73 | All identical — 4-line function |
| `spawn_in_world()` | 47 | All identical — eliminated entirely via `World::commands()` + `World::flush()` in Bevy 0.18 |
| `spawn_bolt()` | 8 | Same builder call with same default values |
| `enter_playing()` | 5 | Same state transition chain |
| `default_bolt_definition()` | ~10 | Same `BoltDefinition` literal |

## Approach: TestAppBuilder + Domain test_utils

### The Builder

A typestate builder in `src/shared/test_utils.rs` that tracks whether the state hierarchy has been registered. One typestate dimension: `NoStates` → `WithStates`. This prevents the `in_state_node_playing()` panic (calling it without state hierarchy) at compile time.

Key capabilities:
- **Plugins**: `.with_physics()`
- **State**: `.with_state_hierarchy()` → `.in_state_node_playing()` / `.in_state_chip_selecting()`
- **Resources**: `.with_playfield()`, `.with_resource::<R>()`, `.insert_resource(val)`
- **Messages**: `.with_message::<M>()`, `.with_message_capture::<M>()` (generic collector)
- **Registries**: `.with_bolt_registry_entry()`, `.with_breaker_registry_entry()`, `.with_cell_registry_entry()`
- **Systems**: `.with_system(schedule, system)`

See [builder-api.md](builder-api.md) for the full API with behavioral specs.

### Generic Message Capture

`MessageCollector<M>` replaces all per-message collector structs. `.with_message_capture::<DamageCell>()` registers the message, the collector resource, and a collector system in `Last` schedule — all in one call. Tests read from `app.world().resource::<MessageCollector<DamageCell>>().0`.

### spawn_in_world Elimination

Bevy 0.18 provides `World::commands()` + `World::flush()` natively. No helper needed — the 47 copies are deleted entirely (47 → 0). Domain spawners in `test_utils.rs` encapsulate `commands()` + `flush()` internally. Direct component spawning uses `World::spawn()` which is immediate (no flush needed).

### Domain test_utils Modules

Each domain provides entity spawners and default definitions in `<domain>/test_utils.rs` (`#[cfg(test)] pub(crate)`). See [shortcuts.md](shortcuts.md) for the full list.

## Implementation Waves

### Wave 1: Shared Foundations

Create `src/shared/test_utils.rs`:
- `TestAppBuilder<S>` with full API
- `MessageCollector<M>` generic type
- `tick()` — the single implementation
- `spawn_in_world()` — simplified with `World::commands()`
- `in_state_node_playing()` / `in_state_chip_selecting()` state navigation

**Validation**: `cargo dtest` passes with no changes to any test file yet.

### Wave 2: Domain test_utils (parallelizable per domain)

For each domain (`bolt`, `breaker`, `cells`, `effect`, `chips`, `walls`, `state`):

1. Read all `helpers.rs` and `test_app()` definitions within the domain
2. Identify common entity spawners and definition factories
3. Write `<domain>/test_utils.rs` with consolidated versions
4. Update `<domain>/mod.rs` to include `#[cfg(test)] pub(crate) mod test_utils;`

**Do not migrate test files yet** — just create the modules and verify they compile.

### Wave 3: Migration (one domain at a time, independently verifiable)

For each domain:
1. Replace `tick()` definitions with `use crate::shared::test_utils::tick;`
2. Replace `spawn_in_world()` with import or inline `world.commands()` + `world.flush()`
3. Replace `spawn_<entity>()` and `default_<entity>_definition()` with imports from domain test_utils
4. Convert `test_app()` functions to use `TestAppBuilder`
5. Remove now-empty helper functions
6. Run `cargo dtest` after each domain

**Domain order** (by duplication density, easiest wins first):
1. `bolt/` — 8 spawn_bolt copies, most consistent patterns
2. `breaker/` — similar builder pattern to bolt
3. `cells/` — spawn_cell used by multiple domains
4. `walls/` — small, quick win
5. `state/` — large but straightforward patterns
6. `chips/` — smallest test surface
7. `effect/` — SKIP for now (clean-room `new_effect/` builds on the new convention from day one)

### Wave 4: Cross-Domain Cleanup

After all domains have test_utils:
- Tests that import from another domain's `helpers.rs` switch to `test_utils`
- Remove any remaining duplicate definitions
- Verify no `pub(super)` helper reimplements something available in `test_utils`

## Risk Assessment

**Low risk.** This is a mechanical refactor:
- No production code changes
- No test logic changes — same assertions, same setup, same values
- Each domain verifiable independently with `cargo dtest`
- Revertible at any granularity (per-file, per-domain)

**The one trap**: `test_app()` functions that look similar but have subtle differences (e.g., one registers a message and another doesn't). The migration must preserve each test_app's exact registration set. The [migration-map.md](migration-map.md) documents every test_app and its exact builder call.

## Acceptance Criteria

- [ ] `TestAppBuilder` exists in `shared/test_utils.rs` with full API
- [ ] `MessageCollector<M>` replaces all per-message collector structs
- [ ] `shared/test_utils.rs` has `tick()`, `spawn_in_world()`
- [ ] Each domain has `test_utils.rs` with its common spawners/definitions
- [ ] Zero duplicate `tick()` definitions (73 → 1)
- [ ] Zero `spawn_in_world()` definitions (47 → 0, eliminated via `World::commands()` + `World::flush()`)
- [ ] Each `spawn_<entity>()` exists once per domain
- [ ] Each `default_<entity>_definition()` exists once per domain
- [ ] `test_app()` functions either use `TestAppBuilder` or are thin wrappers around it
- [ ] All tests pass: `cargo all-dtest`
- [ ] All linting passes: `cargo all-dclippy`
- [ ] `docs/architecture/testing.md` updated to reflect `TestAppBuilder` pattern

## Relationship to Effect Refactor (Todo #3)

**Skip the `effect/` domain entirely.** It's being replaced by `new_effect/` (renamed to `effect/` when complete). The clean-room build will use `TestAppBuilder` from day one. A separate follow-up todo (#3 in the list) handles migrating `new_effect/`'s tests to the consolidated pattern after it ships.

Do NOT touch any file under `src/effect/` during this refactor.

## Dependencies

- Depends on: nothing
- Blocks: nothing (but simplifies all future test writing)
- Coordinates with: todo #2 (effect refactor) — skip `src/effect/`, it's being replaced
- Follow-up: todo #3 (new_effect test migration) — migrate `new_effect/` tests to `TestAppBuilder` after swap

## Status

`ready`
