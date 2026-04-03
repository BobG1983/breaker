# Tidy Up Imports

## Summary
Replace `super::super::` chains with `crate::` paths, then consolidate verbose `use` statements into glob imports where appropriate.

## Context
The codebase has accumulated `super::super::` import chains (especially in test modules after file splits) that are harder to read and more fragile than `crate::` absolute paths. There are also opportunities to consolidate multiple individual imports from the same module into glob imports (`use module::*` or `use module::{A, B, C}`).

The user specified the order: do `crate::` replacement first, then look for glob opportunities. This matters because replacing `super::` chains may surface new grouping opportunities.

## Scope
- In: All `.rs` files in `breaker-game/`, `breaker-scenario-runner/`, `rantzsoft_spatial2d/`, `rantzsoft_physics2d/`, `rantzsoft_defaults/`
- In: Replace `super::super::` (2+ levels) with `crate::` absolute paths
- In: Consolidate multiple `use` items from the same module into grouped imports or globs
- Out: Single `super::` (one level up is fine and idiomatic)
- Out: Changing any logic, signatures, or visibility — imports only

## Dependencies
- Depends on: nothing — purely mechanical refactor
- Blocks: nothing

## Notes
- Do `crate::` replacement pass first, then glob consolidation pass second
- Be careful with `#[cfg(test)]` modules — `super::` in test modules referring to the parent production module is idiomatic and should stay as single `super::` (but `super::super::` should still become `crate::`)
- Run clippy + tests after each pass to catch any breakage

## Status
`ready`
