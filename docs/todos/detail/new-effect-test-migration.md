# New Effect Domain — Test Infrastructure Migration

## Summary

After the effect system refactor (todo #2) ships and `new_effect/` is renamed to `effect/`, migrate its tests to use `TestAppBuilder` and domain `test_utils.rs` from todo #1.

## Context

The effect refactor builds `new_effect/` as a clean room. It will write its own test helpers during development. Once shipped and renamed to `effect/`, those helpers should be consolidated to match the `TestAppBuilder` pattern established by todo #1.

This is intentionally sequenced after both todos #1 and #2:
- Todo #1 establishes `TestAppBuilder` and migrates all non-effect domains
- Todo #2 builds `new_effect/` with whatever test helpers are expedient during development
- This todo aligns `effect/` with the rest of the codebase

## Scope

- In: Create `effect/test_utils.rs` with domain spawners, definition factories, and app builder shortcuts
- In: Replace any duplicate `tick()`, `spawn_in_world()`, or bespoke collector types with shared infrastructure
- In: Convert `test_app()` functions to use `TestAppBuilder`
- Out: Changing test logic or assertions — purely mechanical migration

## Dependencies

- Depends on: todo #1 (test infrastructure consolidation), todo #2 (effect system refactor)
- Blocks: nothing

## Status

`[NEEDS DETAIL]` — scope depends on what test helpers the effect refactor creates. Detail this after todo #2 ships.
