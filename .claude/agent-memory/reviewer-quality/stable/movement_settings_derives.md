---
name: MovementSettings derives — resolved
description: MovementSettings and BumpSettings now both have Clone, Copy — inconsistency was fixed in feature/breaker-builder-pattern
type: project
---

As of feature/breaker-builder-pattern, all settings structs in `breaker/builder/core.rs` consistently derive `Clone, Copy`:
- `MovementSettings` — `#[derive(Clone, Copy)]`
- `DashSettings`, `DashParams`, `BrakeParams`, `SettleParams` — `#[derive(Clone, Copy)]`
- `BumpSettings` — `#[derive(Clone, Copy)]`
- `BumpFeedbackSettings` — `#[derive(Clone, Copy)]`

**Why:** Previous inconsistency was an oversight; resolved in the builder refactor.
**How to apply:** No longer flag this as an issue.
