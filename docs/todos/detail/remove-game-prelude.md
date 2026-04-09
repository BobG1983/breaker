# Remove Game Crate Prelude Module

## Summary
Remove `crate::prelude` from `breaker-game`. Replace all `use crate::prelude::*` imports with explicit per-domain imports. Keep preludes in `rantzsoft_*` crates where they serve as stable public APIs.

## Context
The prelude was added to simplify cross-domain imports via `use crate::prelude::*`. In practice:
- Most files import 1-2 cross-domain types — the glob is overkill
- It hides actual dependencies, making refactoring harder
- Unused re-exports cause clippy warnings requiring constant maintenance
- After file splits, determining which prelude items a file needs is tedious
- The domains are well-organized enough that explicit imports are only marginally longer

For `rantzsoft_*` crates (external consumer APIs), preludes remain appropriate.

## Scope
- In: Remove `src/prelude/` module and all `use crate::prelude::*` statements in `breaker-game`
- In: Replace each glob import with explicit `use crate::<domain>::<type>` imports
- Out: `rantzsoft_*` crate preludes (keep them)
- Out: `use crate::prelude` in the scenario runner (it imports from the game crate's public API — replace with explicit `breaker::` imports)

## Approach
1. For each file using `use crate::prelude::*`:
   - Remove the glob import
   - Run `cargo dcheck` to see what's missing
   - Add explicit imports for each missing type
2. Once no files import from prelude, delete `src/prelude/`
3. Update `docs/architecture/standards.md` to remove prelude conventions section
4. Domain-by-domain to keep diffs reviewable

## Dependencies
- Depends on: nothing
- Blocks: nothing (but easier after test infra consolidation since fewer files to touch)

## Status
`ready`
