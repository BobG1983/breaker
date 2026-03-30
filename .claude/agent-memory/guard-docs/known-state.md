---
name: known-state
description: Confirmed doc/code alignment state; covers effect system rewrite (2026-03-28) and stat-effects phase (feature/stat-effects, merged to develop)
type: project
---

## Confirmed Correct (as of runtime-effects branch, 2026-03-28)

- `docs/design/effects/explode.md` — "Not yet implemented" removed; explode is fully implemented
- `docs/architecture/messages.md` — DamageCell sender list now includes all effect senders (shockwave, explode, pulse, chain_lightning, piercing_beam, tether_beam). SpawnChainBolt removed (never existed). SpawnAdditionalBolt removed entirely from both code and docs.
- `docs/architecture/ordering.md` — spawn_additional_bolt and spawn_chain_bolt entries removed (neither system exists; effects spawn directly via &mut World)
- `docs/design/terminology/core.md` — ChainBolt entry corrected (now references ChainBoltMarker, ChainBoltAnchor, ChainBoltConstraint, DistanceConstraint; removed SpawnChainBolt/spawn_chain_bolt/break_chain_on_bolt_lost which never existed)
- `docs/plan/index.md` — Runtime Effects entry added to Current section (In Progress)
- `docs/architecture/effects/core_types.md` — EffectKind enum is complete and current for all 25 effect modules
- `docs/design/effects/` — all 25 design docs match implemented behavior

## SpawnAdditionalBolt — REMOVED (feature/runtime-effects)

`SpawnAdditionalBolt` was removed from `bolt/messages.rs` and from `docs/architecture/messages.md`.
`spawn_bolts::fire()` and `chain_bolt::fire()` spawn directly via `&mut World`. Do NOT flag its
absence as missing functionality — direct World spawning is the established pattern.

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

## Architecture Confirmed (source-chip-shield-absorption, 2026-03-29)

- Effect dispatch: `EffectKind::fire(entity, source_chip: &str, world)` / `reverse(entity, source_chip: &str, world)` — source_chip added to ALL fire/reverse signatures
- `EffectCommandsExt` methods: `fire_effect(entity, effect, source_chip: String)`, `reverse_effect(entity, effect, source_chip: String)`, `transfer_effect(entity, chip_name: String, children, permanent)`, `push_bound_effects(entity, effects: Vec<(String, EffectNode)>)`
- `PushBoundEffects` — custom `Command` struct in `effect/commands.rs`; inserts `BoundEffects`+`StagedEffects` if absent, appends entries. Used by `dispatch_cell_effects` and `dispatch_breaker_effects`.
- `CellEffectsDispatched` — marker component in `cells/components/types.rs`; prevents double-dispatch by `dispatch_cell_effects`
- `dispatch_cell_effects` — cells system; `OnEnter(GameState::Playing)` after `NodeSystems::Spawn`; skips cells with `CellEffectsDispatched`
- `dispatch_breaker_effects` — breaker system; `OnEnter(GameState::Playing)` chained after `init_breaker`, both after `BreakerSystems::InitParams` and `NodeSystems::Spawn`
- `dispatch_wall_effects` — wall system; `OnEnter(GameState::Playing)` chained after `spawn_walls`; currently a no-op stub (walls have no RON-defined effects)
- `ChainArcCountReasonable` — new `InvariantKind` variant; checks combined `ChainLightningChain` + `ChainLightningArc` entity count against `invariant_params.max_chain_arc_count` (default 50)
- `SpawnExtraChainArcs(usize)` — new `MutationKind` variant; spawns N chain + N arc entities for self-test
- InvariantKind total: 22 variants (was 16 documented — `ChipOfferExpected`, `SecondWindWallAtMostOne`, `ShieldChargesConsistent`, `PulseRingAccumulation`, `EffectiveSpeedConsistent`, `ChainArcCountReasonable` were undocumented)
- MutationKind total: 15 variants (was 5 documented — added in multiple prior sessions)
- `EffectSourceChip(Option<String>)` — component on AoE/spawn effect entities; carries chip attribution from dispatch to damage-application tick
- `chip_attribution(source_chip: &str) -> Option<String>` — helper: empty → None, non-empty → Some
- fire() method split: `fire` → `fire_aoe_and_spawn` → `fire_utility_and_spawn` (3 methods)
- reverse() method split: `reverse` → `reverse_aoe_and_spawn` (2 methods)
- `EffectKind::Shield { stacks: u32 }` — only field is stacks (NO base_duration, NO duration_per_level)
- `EffectKind::Attraction { attraction_type, force, max_force: Option<f32> }` — named fields (NOT tuple)
- `EffectKind::ChainLightning { arcs, range, damage_mult, arc_speed }` — arc_speed field (default 200.0 via serde)
- `EffectKind::Pulse { base_range, range_per_level, stacks, speed, interval }` — interval field (default 0.5 via serde)
- `BoltSystems::WallCollision` — defined in bolt/sets.rs, tags bolt_wall_collision, runs after CellCollision
- ShieldActive cross-domain writes: bolt domain writes ShieldActive on breaker entity (bolt_lost); cells domain writes ShieldActive on cell entities (handle_cell_hit). Both are accepted architectural exceptions.
- No typed observer events. No `ActiveEffects`/`ArmedEffects`/`EffectChains` resources.
- Chain stores: `BoundEffects` (permanent) + `StagedEffects` (one-shot)
- Effect file pattern: `fire(entity, ..params.., source_chip: &str, world)` + `reverse(...)` + `register()` free functions per module
- Stat model: `Active*` stacks (effect domain) → `Effective*` scalars (computed by Recalculate) → consumers
- `PiercingRemaining` is bolt domain (gameplay state), not an effect stat. `EffectivePiercing` is the cap.
- `EffectSystems::Recalculate` ordering: `.after(EffectSystems::Bridge)`, run_if `in_state(PlayingState::Active)`
