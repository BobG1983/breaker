---
name: split-patterns
description: Recurring file split patterns and lessons from this codebase
type: project
---

## Recurring Split Patterns

### Strategy A (Test Extraction) -- dominant pattern
Most oversized files in this codebase follow the pattern: small production code + large inline test module.
Typical ratio: 50-150 lines prod, 300-1400 lines tests.

Key files that grow back:
- `cells/systems/dispatch_cell_effects.rs` -- had a prior incomplete split (directory existed with no mod.rs)
- `effect/triggers/evaluate.rs` -- has TWO `#[cfg(test)]` modules (tests + on_resolution_tests)
- `effect/core/types.rs` -- has `#[cfg(test)] impl` block before the test module

### Strategy B (Concern Separation) -- rare
- `lifecycle/systems.rs` is the primary candidate: 1241 lines of pure production code, 30+ public items
- Split by concern: input, plugin, menu bypass, frame control, debug setup, entity tagging, frame mutations, pending effects, perfect tracking
- Complex because every function shares a massive import block; each child file needs its own subset

### mod.rs Violations
- `breaker-scenario-runner/src/types/mod.rs` -- 560 lines of type definitions, not just wiring
- Fix: extract to `definitions.rs`, `mod.rs` becomes `mod definitions; pub use definitions::*; #[cfg(test)] mod tests;`

### Import Adjustment Rules (verified)
- `some_file.rs` tests using `use super::*;` -> `some_file/tests.rs` needs `use super::system::*;`
- `some_file.rs` tests using `use super::*;` -> `some_file/tests/group.rs` needs `use super::super::system::*;`
- For types files: `use super::types::*;` or `use super::definitions::*;`

### Files That Cannot Be Meaningfully Split
- `rantzsoft_physics2d/src/quadtree/tree.rs` (451 lines) -- single data structure, no tests
- `breaker-scenario-runner/src/runner/execution.rs` (439 lines) -- single concern, no tests
- `breaker-scenario-runner/src/runner/app.rs` (402 lines) -- single concern, tiny test section

### Pure Test Files Over Threshold
Many files in `tests/` directories are 400-800 lines. These are already extracted and under the 800-line Strategy C threshold. They don't need further splitting unless they exceed 800 lines.

`dispatch_breaker_effects/tests.rs` (812 lines) was split into a tests/ directory by c9964b7.
Remaining files in the 600-800 range are all in the reviewer-file-length/phase4_findings.md list.
