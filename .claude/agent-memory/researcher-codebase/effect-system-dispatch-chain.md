---
name: Effect system dispatch chain
description: How BoundEffects/StagedEffects are walked, bridge pattern, Until desugaring, chip dispatch — current system and new system differences
type: project
---

## Current system (`src/effect/`)

**Storage**: `BoundEffects(Vec<(String, EffectNode)>)` (permanent, never consumed) and
`StagedEffects(Vec<(String, EffectNode)>)` (one-shot, consumed on trigger match). Always
inserted as a pair.

**Bridge pattern**: each trigger type has one system in `src/effect/triggers/`. Bridge reads
a game message, builds `TriggerContext { bolt, breaker, cell, wall }`, calls
`evaluate_bound_effects` then `evaluate_staged_effects` (Bound first — this is a known issue
fixed in the new system).

**`evaluate_bound_effects`**: walks `BoundEffects`, matches `When` nodes, queues `Do` children
via `commands.fire_effect`, stages non-`Do` children into `StagedEffects`.

**`evaluate_staged_effects`**: walks `StagedEffects`, consumes matching entries, handles
`Reverse` nodes (fires `reverse_effect` + `RemoveChainsCommand`), handles `On` nodes via
`ResolveOnCommand`.

**Until desugaring**: `desugar_until` in FixedUpdate during Playing. Extracts Until nodes,
fires their Do children, installs chains into BoundEffects, pushes `When { trigger, Reverse }` 
into StagedEffects.

**Chip dispatch**: `dispatch_chip_effects` reads `ChipSelected`. Breaker-target effects
dispatch immediately; all other targets wrap in `When(NodeStart, On(target, permanent: true, ...))` 
on the breaker (deferred dispatch).

## New system (`src/new_effect/` — todo #2)

Key changes:
- `BoundEffects` and `StagedEffects` become `HashMap`-keyed (by `Trigger`/`Condition`)
- `sources: HashMap<SourceId, Vec<BoundKey>>` reverse index enables `remove_source`
- **Walk order inverted**: StagedEffects FIRST, then BoundEffects (prevents same-trigger arm+consume)
- `During(Condition, ...)` is first-class (not desugared); condition monitors activate/deactivate it
- `Route` replaces `RootEffect::On` at definition level; sets `This` for subtree
- `Stamp` (permanent to BoundEffects) and `Transfer` (one-shot to StagedEffects) are explicit terminals
- `SpawnedRegistry` resource handles `EveryBolt`/`EveryCells` routing to future spawned entities
- New trigger `Killed(KillTarget)` fires on the killer entity
- Depth limit: `TriggerContext.depth: u32`, max 10

## Protocol integration surface

Effect-tree protocols (Deadline, Ricochet, Anchor, Kickstart) use `Route(...)` definitions
identical in shape to chips. SourceId convention: `"protocol:<name>"`.

Custom-system protocols (Debt Collector, Echo Strike, Siphon, Fission, etc.) need
`ActiveProtocols(HashSet<ProtocolKind>)` resource + per-protocol systems with run_if guard.

**Why**: Protocol effects should target the new system, not the old one.
**How to apply**: When writing protocol specs or implementation, use new system API (Route,
Stamp, Transfer, During, walk_effects, remove_source). Do not write protocol code against
`src/effect/` BoundEffects/StagedEffects Vec-based API.
