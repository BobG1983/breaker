---
name: Production validate() logic in stub crosses RED/GREEN boundary
description: When a struct's validate() method is updated with full production logic by writer-tests (not just a no-op), all tests for that behavior — both positive (Ok) and negative (Err) — pass at RED
type: feedback
---

When a spec renames a field and changes validation rules (e.g., `guardian_hp` → `guardian_hp_fraction` with range `(0.0, 1.0]`), writer-tests may implement the full validate() logic as part of the struct stub. This causes all 8+ tests for that validation behavior to pass at RED, breaking the RED gate.

**Why:** The writer-tests agent sees validate() as a structural method on the struct (not "production logic"), but it IS production logic — even if it's short. Any non-trivial branching/range check in a stub crosses the RED/GREEN boundary.

**How to apply:** Flag any `validate()` implementation that contains `if` branches, range checks, or `return Err(...)` as BLOCKING production logic in a stub. The stub validate() should always return `Ok(())` (or `todo!()`). The tests for reject cases (e.g., `guarded_behavior_validate_rejects_zero_fraction`) will then correctly fail at RED because Ok != Err.
