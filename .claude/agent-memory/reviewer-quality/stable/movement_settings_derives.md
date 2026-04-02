---
name: MovementSettings missing derives
description: MovementSettings in core.rs has no derives; BumpSettings only has Clone — inconsistent with DashSettings (Clone, Copy)
type: project
---

In `breaker-game/src/breaker/builder/core.rs`:
- `MovementSettings` — no derives at all
- `BumpSettings` — `#[derive(Clone)]` only
- `BumpFeedbackSettings` — `#[derive(Clone, Copy)]` 
- `DashSettings`, `DashParams`, `BrakeParams`, `SettleParams` — all `#[derive(Clone, Copy)]`

All fields in all these structs are `EaseFunction` (Copy) and `f32` (Copy), so all are Copy-eligible.

**Why:** Appears to be an oversight — no deliberate decision to omit Copy.
**How to apply:** Flag as an idiom inconsistency when reviewing this file.
