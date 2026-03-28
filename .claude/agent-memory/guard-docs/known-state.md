---
name: known-state
description: Verified doc/code alignment state after effect system rewrite review (2026-03-28)
type: project
---

Confirmed correct after feature/effect-system-rewrite review on 2026-03-28.

**Why:** The effect system was rebuilt from scratch (commit 35c10d1). The old observer/typed-event architecture was replaced with direct `EffectKind::fire()/reverse()` free functions. Several docs lagged behind.

**How to apply:** When reviewing effect-domain docs in future sessions, start from this baseline to avoid re-reporting already-fixed drift.

## Confirmed Correct (as of 2026-03-28)

- `docs/architecture/messages.md` — Collision messages now use `BoltImpactCell`, `BoltImpactWall`, `BreakerImpactCell`, `BreakerImpactWall`, `CellImpactWall`. `DamageCell.source_chip` (not `source_bolt`). Observer Events section replaced with Effect Dispatch section describing `EffectCommandsExt`.
- `docs/architecture/effects/core_types.md` — `EffectKind` enum now includes `Explode`, `QuickStop`, `TetherBeam`. `SecondWind` is a unit variant (no `invuln_secs` field). `EntropyEngine` uses `max_effects: u32` (not `threshold: u32`).
- `docs/architecture/effects/reversal.md` — Passive buffs table no longer lists non-existent `ChainHit`/`BreakerSpeed` variants. New effects added. Fire-and-forget category added.
- `docs/architecture/effects/node_types.md` — `Once` example uses `Do(SecondWind)` (unit variant).
- `docs/architecture/layout.md` — Effect domain layout now reflects actual `core/types.rs` structure and per-trigger-type files in `triggers/`.
- `docs/architecture/plugins.md` — Cross-domain reads updated to new message names. Effect Domain section rewritten for the new architecture.
- `docs/design/chip-catalog.md` — No TiltControl section, no MultiBolt reference. SpawnBolts is the correct effect name. This is correct.

## Intentionally Forward-Looking (do NOT flag as drift)

- `docs/design/chip-catalog.md` — Chip RON files do not exist yet (Phase 7 content). The catalog is design spec, not committed code. The trigger notation (`When(PerfectBumped)`, `When(OnBump)`) is authoring shorthand — not required to match `Trigger` enum variant names literally.
- `docs/design/effects/ramping_damage.md` — `damage_per_trigger` is correct per code.
- Evolution chips (Entropy Engine, Nova Lance, etc.) — Not yet implemented in code. Design spec only.
- `docs/plan/index.md` — The Spatial/Physics Extraction phase is marked "Done" in the plan. Correct.

## Architecture Confirmed (effect system post-rewrite)

- Effect dispatch: `EffectKind::fire(entity, world)` / `reverse(entity, world)` via `EffectCommandsExt` on `Commands`
- No typed observer events. No `fire_typed_event`/`fire_passive_event`. No `ActiveEffects`/`ArmedEffects`/`EffectChains` resources.
- Chain stores: `BoundEffects` (permanent, component) + `StagedEffects` (one-shot, component)
- Effect file pattern: `fire()` + `reverse()` + `register()` free functions per module
