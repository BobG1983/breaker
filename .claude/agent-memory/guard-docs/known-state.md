---
name: known-state
description: Confirmed doc/code alignment state; covers effect system rewrite (2026-03-28) and stat-effects phase (feature/stat-effects, merged to develop)
type: project
---

## Confirmed Correct (as of effect system rewrite, 2026-03-28)

- `docs/architecture/messages.md` — Collision messages use `BoltImpactCell`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `CellImpactWall`. `DamageCell.source_chip` (not `source_bolt`). Observer Events section replaced with Effect Dispatch section.
- `docs/architecture/effects/core_types.md` — `EffectKind` enum includes `Explode`, `QuickStop`, `TetherBeam`. `SecondWind` is unit variant. `EntropyEngine` uses `max_effects: u32`.
- `docs/architecture/effects/reversal.md` — Passive buffs table uses correct variants. Fire-and-forget category added.
- `docs/architecture/effects/node_types.md` — `Once` example uses `Do(SecondWind)` (unit variant).
- `docs/architecture/layout.md` — Effect domain layout reflects `core/types.rs` + per-trigger-type files in `triggers/`.
- `docs/design/chip-catalog.md` — SpawnBolts correct. No TiltControl/MultiBolt.

## Confirmed Correct (as of stat-effects merge, 2026-03-28+)

- `docs/architecture/plugins.md` — Cross-Domain Read Access: bolt reads `PiercingRemaining` (bolt domain) + `EffectivePiercing`/`EffectiveDamageMultiplier` (effect domain); breaker reads `EffectiveSpeedMultiplier`/`EffectiveSizeMultiplier` (effect domain); cells receives pre-computed damage via `DamageCell` message (no direct Effective* read). `EffectSystems` entry lists both `Bridge` and `Recalculate`. Effect domain `sets.rs` line updated.
- `docs/architecture/ordering.md` — `EffectSystems::Recalculate`, `BoltSystems::CellCollision`, `BreakerSystems::UpdateState` in Defined Sets table. FixedUpdate chain shows Recalculate above Move/PrepareVelocity.
- `docs/architecture/data.md` — Active/Effective Component Pattern section added.
- `docs/plan/index.md` — Stat Effects entry added to Current section.

## Key Architectural Fact: DamageCell pre-bakes multiplier

`handle_cell_hit` (cells domain) does NOT read `EffectiveDamageMultiplier` directly. `bolt_cell_collision` reads it and applies it when computing `effective_damage = BASE_BOLT_DAMAGE * multiplier` — that pre-computed value goes into the `DamageCell.damage` field. Cells are decoupled from the effect stat model.

**Why:** The bolt domain owns collision, so it applies the multiplier at collision time. The cells domain only needs to know how much damage to apply.

**How to apply:** Do not flag cells reading Effective* types as missing — it's correct that cells doesn't read them.

## Intentionally Forward-Looking (do NOT flag as drift)

- `docs/design/chip-catalog.md` — Chip RON files do not exist yet (Phase 7 content). Design spec only.
- `docs/design/effects/ramping_damage.md` — `damage_per_trigger` is correct per code.
- Evolution chips (Entropy Engine, Nova Lance, etc.) — Not yet implemented in code. Design spec only.
- `docs/plan/index.md` — Spatial/Physics Extraction and Stat Effects are both correctly marked Done.

## chips/components.rs — Intentional Stub

The file contains only a doc comment explaining legacy stat components were removed. Do not flag as a missing/empty file. Chip stat components (DamageBoost, BoltSpeedBoost, BreakerSpeedBoost, BumpForceBoost, Piercing) were removed; state is now managed by effect domain Active*/Effective* pairs.

**Why:** stat-effects phase migration removed all flat chip stat components.
**How to apply:** When reviewing chips domain, expect components.rs to be a stub with doc comment only.

## Architecture Confirmed (effect system post-rewrite)

- Effect dispatch: `EffectKind::fire(entity, world)` / `reverse(entity, world)` via `EffectCommandsExt` on `Commands`
- No typed observer events. No `ActiveEffects`/`ArmedEffects`/`EffectChains` resources.
- Chain stores: `BoundEffects` (permanent) + `StagedEffects` (one-shot)
- Effect file pattern: `fire()` + `reverse()` + `register()` free functions per module
- Stat model: `Active*` stacks (effect domain) → `Effective*` scalars (computed by Recalculate) → consumers
- `PiercingRemaining` is bolt domain (gameplay state), not an effect stat. `EffectivePiercing` is the cap.
- `EffectSystems::Recalculate` ordering: `.after(EffectSystems::Bridge)`, run_if `in_state(PlayingState::Active)`
