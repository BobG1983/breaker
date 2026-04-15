# Prelude Expansion & Import Cleanup

## Summary
Simplify import blocks across `breaker-game` by consolidating verbose `use` paths, hoisting widely-used types into `crate::prelude`, and replacing repetitive import blocks with prelude glob imports.

## Context
Many files use fully qualified paths or large explicit import blocks where shorter paths or prelude globs would be cleaner. The prelude already exists at `src/prelude/` — the goal is to make better use of it rather than removing it.

Three phases of work:
1. **Simplify paths** — find places using fully qualified `crate::domain::subdomain::Type` inline and add `use` statements at the top of the file instead
2. **Hoist into prelude** — identify types imported in 3+ locations across the codebase and add them to `crate::prelude` re-exports
3. **Replace verbose imports with globs** — files with many imports (especially from the same domains) should use `use crate::prelude::*` instead of listing each type individually

## Scope
- In: All `.rs` files in `breaker-game`
- In: `src/prelude/` module — expand re-exports for widely-used types
- In: Replace verbose per-type import blocks with `use crate::prelude::*` where appropriate
- Out: `rantzsoft_*` crate preludes (leave as-is)
- Out: External-facing imports in `breaker-scenario-runner`
- Out: Changing any logic, signatures, or behavior — purely mechanical import refactoring

## Approach
1. **Audit fully qualified paths**: grep for inline `crate::` paths used in expressions/types (not `use` statements) and convert to `use` imports at file top
2. **Frequency analysis**: count how many files import each type; types at 3+ locations are candidates for prelude hoisting
3. **Expand prelude**: add high-frequency types to `src/prelude/` re-export modules, organized by domain
4. **Replace import blocks**: files importing 3+ prelude-available types switch to `use crate::prelude::*`
5. **Verify**: `cargo all-dtest` + `cargo all-dclippy` + `cargo fmt` after each batch

## Dependencies
- Depends on: nothing
- Blocks: nothing (but makes future cross-domain work easier by reducing import noise)

## Notes
- The prelude should stay organized by domain (bolt, breaker, cells, effect, shared, etc.) with one re-export module per domain
- Types that are truly domain-internal (used only within their own domain) should NOT be hoisted
- Test-only types stay out of the main prelude — test utils have their own import patterns
- Keep `rantzsoft_*` types out of the game prelude — they have their own preludes

## Status
`ready`
