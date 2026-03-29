---
name: known-state
description: Confirmed doc/code alignment state; covers effect system rewrite (2026-03-28) and stat-effects phase (feature/stat-effects, merged to develop)
type: project
---

## Confirmed Correct (as of runtime-effects branch, 2026-03-28)

- `docs/design/effects/explode.md` — "Not yet implemented" removed; explode is fully implemented
- `docs/architecture/messages.md` — DamageCell sender list now includes all effect senders (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam). SpawnChainBolt removed (never existed). SpawnAdditionalBolt moved to Registered section (registered but no active producer/consumer).
- `docs/architecture/ordering.md` — spawn_additional_bolt and spawn_chain_bolt entries removed (neither system exists; effects spawn directly via &mut World)
- `docs/design/terminology/core.md` — ChainBolt entry corrected (now references ChainBoltMarker, ChainBoltAnchor, ChainBoltConstraint, DistanceConstraint; removed SpawnChainBolt/spawn_chain_bolt/break_chain_on_bolt_lost which never existed)
- `docs/plan/index.md` — Runtime Effects entry added to Current section (In Progress)
- `docs/architecture/effects/core_types.md` — EffectKind enum is complete and current for all 25 effect modules
- `docs/design/effects/` — all 25 design docs match implemented behavior

## SpawnAdditionalBolt — Intentional Dead Registration

Registered in BoltPlugin but no producer or consumer exists. `spawn_bolts::fire()` and `chain_bolt::fire()` spawn directly via `&mut World`. This is likely a placeholder or legacy from a pre-direct-spawn design. Recorded in messages.md Registered section. Do NOT flag as missing code — it may be intentionally unused until future cross-domain spawn coordination is needed.

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
