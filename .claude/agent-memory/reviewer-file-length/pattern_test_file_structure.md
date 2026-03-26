---
name: Test file structure patterns
description: How tests are organized in this codebase — already-extracted tests.rs files vs inline #[cfg(test)] blocks
type: project
---

## Pattern: Already-extracted test files (breaker-scenario-runner)

The scenario-runner crate uses `mod.rs` (pure production) + `tests.rs` (pure tests, no `#[cfg(test)]` wrapper at file level). Files like:
- `lifecycle/mod.rs` — production code, first line `#[cfg(test)] mod tests;`
- `lifecycle/tests.rs` — entire file is test code, no `#[cfg(test)]` wrapper needed (the `mod tests;` in mod.rs gates it)
- Same pattern in `types/`, `verdict/`, `runner/`, `input/`

**Why**: This is the project's intentional pattern for already-split modules. When checking prod/test line split, `grep #[cfg(test)]` returns nothing in `tests.rs` files — the whole file IS the test module.

## Pattern: Inline tests (breaker-game)

Game crate files keep tests inline: production code, then `#[cfg(test)] mod tests { ... }` at the bottom. This is the primary split target.

## Pattern: Section headers in tests

Tests in this codebase use `// ---...--- // system_name // ---...---` section headers to delimit logical test groups. These map directly to sub-split `group.rs` file names when Strategy C (sub-split) is needed.

**How to apply**: When identifying test groups for sub-splits, grep for `^// ===` or `^// ---` comment lines to find pre-existing group boundaries. These are reliable split points.
