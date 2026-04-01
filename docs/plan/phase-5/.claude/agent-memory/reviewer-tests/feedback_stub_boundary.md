---
name: Stub boundary — production logic in fire() and "real types" distinction
description: Two patterns for RED phase involving existing functions; the key distinction is whether the spec designates types as real or stub
type: feedback
---

## Pattern 1 — Violation (wave1c phantom FIFO)

For the wave1c FIFO spawn phantom feature, writer-tests correctly added the two new stub types (PhantomSpawnOrder, PhantomSpawnCounter) but did NOT stub out the existing `fire()` function body. The pre-existing production logic for max_active enforcement (the `while owned.len() >= max_active` loop and despawn) was left intact in the stub file.

This means several tests (behaviors 3–7, 9) passed against the stub rather than failing, violating the RED gate requirement.

**Why:** The spec explicitly said "Do NOT modify any production code — stubs only (empty PhantomSpawnOrder component, empty PhantomSpawnCounter resource, no logic in fire())". Writer-tests preserved existing logic rather than stripping it.

## Pattern 2 — Correct approach (wave1b gravity-well FIFO)

The spec used a "Shared Prerequisites" section that explicitly designated `GravityWellSpawnOrder` and `GravityWellSpawnCounter` as "real types, not TDD stubs" and said "Do NOT modify any production code — this is a test-only spec." Writer-tests correctly:
- Added the two real types to `effect.rs` (not stubbed)
- Left `fire()` completely unmodified (no FIFO logic)
- Tests fail RED because `fire()` never stamps `GravityWellSpawnOrder`
- Behavior 9 (max=0) regression guard tests PASS correctly against the existing guard clause

This is the correct pattern when a spec uses "Shared Prerequisites" with "real types": add the types, leave the function untouched, let tests fail naturally.

**How to apply:** Distinguish carefully between specs that say "stub the function" vs. specs that say "add real types, don't touch the function." When the spec uses a "Shared Prerequisites" section with "real types, not TDD stubs" language, do NOT strip `fire()` — the RED state comes from the missing integration, not a stripped stub. When the spec says "no logic in fire()", verify the function body was stripped.
