---
name: Transform usage in effect fire() functions
description: All previously flagged Transform usages in fire() functions are FIXED as of full-verification-fixes branch (2026-03-30); only chain_lightning arc entity Transform usage remains — confirmed correct
type: project
---

All production code previously flagged for incorrect Transform usage has been fixed in the `feature/full-verification-fixes` branch.

**Previously affected — ALL FIXED (as of 2026-03-30):**
- `effect/effects/gravity_well.rs` — FIXED: `fire()` uses `super::super::entity_position()` (Position2D only); `apply_gravity_pull` queries `&Position2D`; spawned entity carries `Position2D(position)`, not Transform
- `effect/effects/shockwave/effect.rs` — FIXED: `fire()` uses `super::super::entity_position()`; spawned entity carries `Position2D`, not Transform
- `effect/effects/explode/effect.rs` — FIXED: `fire()` uses `super::super::entity_position()`; spawned entity carries `Position2D`
- `effect/effects/pulse/effect.rs` — FIXED: emitter reads `&Position2D`; ring carries `Position2D`; `apply_pulse_damage` reads `&Position2D`
- `effect/effects/piercing_beam/effect.rs` — FIXED: `fire()` uses `super::super::entity_position()` (Position2D → Vec2::ZERO only); process system uses `GlobalPosition2D` for cell positions

**Confirmed correct (not a gap):**
- `effect/effects/chain_lightning/effect.rs` — arc_transforms query writes `Transform` on `ChainLightningArc` entities — CORRECT: arc entities are pure rendering objects, not physics entities; using Transform on them is right
- `effect/effects/attraction/effect.rs` — uses `GlobalPosition2D` correctly
- `effect/effects/second_wind/system.rs` — uses `Position2D` correctly

**Rule for new effects:** When reviewing any new effect, flag any Transform reads/writes in physics code. Correct pattern: read `GlobalPosition2D` or `Position2D`, spawn entities with `Position2D`, let `derive_transform` handle visual. Exception: pure rendering entities (like arc entities) may carry Transform.
