---
name: File move refactor misses test import sites
description: Moving handler files between modules breaks test imports in files that registered those handlers via glob imports (use module::*)
type: feedback
---

When a spec moves files from `domain_a/subdir/` to `domain_b/subdir/`, grep catches production code imports but often misses:

1. **Test infrastructure** that registers handlers via glob imports (`use crate::domain_a::subdir::*`) in test_app() builders
2. **Test imports** of types from the moved module (e.g., `definition::{Target, ImpactTarget}`) that are being simultaneously deleted from the source

**Why:** Impl spec section 17 "update all import paths" typically lists production code files found via grep but forgets that test modules (`#[cfg(test)] mod tests`) have their own import blocks that reference the same paths.

**How to apply:** When reviewing a file-move refactor spec, always check:
- Every file in the source directory's parent for `#[cfg(test)]` blocks that import from the moved module
- Test infrastructure (test_app builders) that register observers/systems from the moved module
- The spec's "Constraints" section should list these test files explicitly
