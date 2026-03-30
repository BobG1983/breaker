---
name: Transform usage in effect fire() functions
description: Several effect fire() functions read/write Transform directly instead of Position2D, violating the canonical coordinate rule in physics.md
type: project
---

Multiple effect `fire()` functions and runtime systems read `Transform` to get entity position, and some spawn entities with `Transform` directly. Per `docs/architecture/physics.md`: "Position2D is the canonical position for all game entities. Transform is derived... game systems must never write Transform directly."

**Affected production code (as of 2026-03-30):**
- `effect/effects/gravity_well.rs` — reads `Transform` in fire() (line 33) and `apply_gravity_pull` system (lines 88-89); writes `Transform` on spawned entity (line 63)
- `effect/effects/shockwave/effect.rs` — reads `Transform` in fire() (line 62); writes `Transform` on spawned entity (line 77); reads `Transform` in `apply_shockwave_damage` (line 43)
- `effect/effects/explode/effect.rs` — reads `Transform` in fire() (line 34); writes `Transform` on spawned entity (line 47); reads `Transform` in `process_explode_requests` (line 64)
- `effect/effects/pulse/effect.rs` — reads `Transform` in runtime system; writes `Transform` on spawned pulse ring (line 124)
- `effect/effects/chain_lightning/effect.rs` — writes `Transform` on spawned arc entity (line 211) — but fire() itself uses `GlobalPosition2D` correctly

**Not affected (correct usage):**
- `effect/effects/chain_lightning/effect.rs` fire() — uses `GlobalPosition2D`
- `effect/effects/attraction/effect.rs` — uses `GlobalPosition2D`
- `effect/effects/spawn_extra_bolt()` — uses `Position2D`
- `effect/effects/second_wind/system.rs` — uses `Position2D`

**Why this matters:** In FixedUpdate, Position2D holds the current physics tick's position. Transform holds the derived value from the previous visual frame's AfterFixedMainLoop pass. Reading Transform in FixedUpdate gets a stale position. Writing Transform directly is overwritten by derive_transform in AfterFixedMainLoop.

**Impact:** For fire() functions running at apply_deferred time, the position difference is usually one frame of movement (small for most entities). For runtime systems like `apply_shockwave_damage` running each FixedUpdate tick, the stale position means collision checks use last-frame positions. Effect entities spawned with Transform but no Position2D skip the spatial pipeline entirely.

**How to apply:** When reviewing new effects, flag any Transform reads/writes. The correct pattern is to read GlobalPosition2D or Position2D, spawn entities with Position2D, and let derive_transform handle the visual.
