# Test Infrastructure Consolidation

## Summary

One-time migration to eliminate duplicated test helpers across the game crate. Creates domain-level `test_utils.rs` modules and a shared `shared/test_utils.rs`, then migrates existing test suites to import from them instead of reimplementing identical functions.

See `docs/architecture/testing.md` for the target convention and design rationale.

## The Duplication (measured 2026-04-09)

| Helper | Copies | Notes |
|--------|--------|-------|
| `tick(app)` | 73 | Identical 4-line function in 73 files |
| `spawn_in_world()` | 47 | Near-identical entity spawner |
| `test_app()` | 154 | Each unique, but many share 80%+ of their setup |
| `spawn_bolt()` | 8 | Same builder call with same default values |
| `enter_playing()` | 5 | Same state transition chain |
| `default_bolt_definition()` | ~10 | Same BoltDefinition literal |

Total estimated duplicated test code: **~2,000 lines** across the crate.

## Approach: Domain-by-Domain, Bottom-Up

This is a pure refactor — no test logic changes, no new tests, no production code changes. Each domain can be migrated independently.

### Wave 1: Shared Foundations

Create `shared/test_utils.rs` with the cross-cutting helpers:
- `tick(app)` — the single implementation
- `with_state_hierarchy(app)` — registers all state types
- `enter_playing(app)` — transitions to NodeState::Playing

**Validation**: `cargo dtest` passes with no changes to any test file yet. The module exists but nothing imports it.

### Wave 2: Domain test_utils (one per domain, parallelizable)

For each domain (`bolt`, `breaker`, `cells`, `effect`, `chips`, `walls`, `state`):

1. Read all `helpers.rs` and `test_app()` definitions within the domain
2. Identify common patterns (app builders, entity spawners, definition factories)
3. Write `<domain>/test_utils.rs` with the consolidated versions
4. Update `<domain>/mod.rs` to include `#[cfg(test)] pub(crate) mod test_utils;`

**Do not migrate test files yet** — just create the modules and verify they compile.

### Wave 3: Migration (one domain at a time, each independently verifiable)

For each domain:
1. Replace `tick()` definitions with `use crate::shared::test_utils::tick;`
2. Replace `spawn_<entity>()` and `default_<entity>_definition()` with imports from domain test_utils
3. Simplify `test_app()` functions to compose domain builders
4. Remove now-empty `helpers.rs` functions (keep the file if suite-specific helpers remain)
5. Run `cargo dtest` after each domain

**Order suggestion** (by duplication density, easiest wins first):
1. `bolt/` — 8 spawn_bolt copies, most consistent patterns
2. `breaker/` — similar builder pattern to bolt
3. `cells/` — spawn_cell used by multiple domains
4. `walls/` — small, quick win
5. `effect/` — the biggest domain, do last with the most experience
6. `state/` — may have unique needs
7. `chips/` — smallest test surface

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

**The one trap**: test_app() functions that look similar but have subtle differences (e.g., one registers a message and another doesn't). The migration must preserve each test_app's exact registration set. When consolidating into domain builders, use extension functions for optional capabilities rather than adding everything to the base builder.

## Acceptance Criteria

- [ ] `shared/test_utils.rs` exists with `tick()`, `with_state_hierarchy()`, `enter_playing()`
- [ ] Each domain has `test_utils.rs` with its common builders/spawners
- [ ] Zero duplicate `tick()` definitions (from 73 → 1)
- [ ] Zero duplicate `enter_playing()` definitions (from 5 → 1)
- [ ] Each `spawn_<entity>()` exists once per domain (from N → 1)
- [ ] Each `default_<entity>_definition()` exists once per domain
- [ ] All tests pass: `cargo all-dtest`
- [ ] All linting passes: `cargo all-dclippy`
- [ ] `docs/architecture/testing.md` accurately describes the final state

## When to Do This

**Before the effect refactor (#1).** The effect domain has the most test infrastructure (44K lines, `effect_test_app()` is a god node with 61 edges). If we consolidate first, the clean-room `new_effect/` builds on the new convention from day one and doesn't recreate the problem.

Alternatively, **during the effect refactor** — wave 2-3 for the `effect/` domain happen naturally as part of the clean-room build, and the other domains migrate as a follow-up.

Either way, don't do this *after* the effect refactor — that would mean migrating the new effect tests retroactively.
