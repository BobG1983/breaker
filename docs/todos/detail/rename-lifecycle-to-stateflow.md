# Rename rantzsoft_lifecycle → rantzsoft_stateflow

## Summary
Rename the `rantzsoft_lifecycle` crate to `rantzsoft_stateflow` — the current name is too generic for a crate that specifically handles state routing and screen transitions.

## Context
The crate provides two things: declarative state routing (`Route`, `RoutingTable`, `ChangeState`/`StateChanged` messages, dispatch systems) and visual screen transitions (fade, dissolve, iris, pixelate, slide, wipe). "lifecycle" doesn't convey either of those. "stateflow" captures both the state routing and transition orchestration.

## Scope
- In: rename directory, Cargo.toml `[package]` name, all workspace `Cargo.toml` references, all `use rantzsoft_lifecycle` imports, plugin name (`RantzLifecyclePlugin` → `RantzStateflowPlugin`), doc references
- Out: no behavior changes, no API changes

## Dependencies
- Depends on: nothing
- Blocks: nothing

## Notes
Good `/quickfix` candidate — pure mechanical rename, no logic changes.

## Status
`ready`
