---
name: impact_tilt_control
description: Full reference map for TiltControl — where it appears and where it does NOT exist in code (2026-03-28)
type: project
---

TiltControl was a planned EffectKind variant that was never implemented in the current codebase.

**Why:** The effect system was rebuilt from scratch (commit 35c10d1 `refactor(effect): rebuild effect domain from scratch`). TiltControl was designed in Phase 4b as an `AugmentEffect` variant but never made it into the rebuilt EffectKind enum.

**How to apply:** When asked to remove TiltControl from EffectKind — it is already absent. The only action needed is to update docs.

## Where TiltControl EXISTS

- `docs/plan/done/phase-4/phase-4b-chip-effects.md` — line 39 (in old `AugmentEffect` enum design sketch) and line 93 (implementation table). These are historical design docs, not code.
- `docs/design/chip-catalog.md` — lines 110-119. The "Tilt Control" chip section describes three rarity variants using `Do(TiltControl(0.1/0.2/0.35))`. This is a forward-looking design doc treating TiltControl as a planned but unimplemented effect.

## Where TiltControl does NOT EXIST

- `breaker-game/src/effect/core/types.rs` — `EffectKind` enum has no `TiltControl` variant
- `breaker-game/src/effect/effects/` — no `tilt_control.rs` module
- `breaker-game/src/effect/effects/mod.rs` — no tilt_control registration
- `breaker-game/assets/chips/templates/` — no `tilt_control.chip.ron` file
- `breaker-scenario-runner/src/` — no invariants or scenario types reference TiltControl
- All `.rs` source files — zero production code references
- All `.scenario.ron` files — no scenario uses TiltControl
