# Effect System Refactor: Typestate Builder, Spawn/During Triggers, Unified Vocabulary

## Summary
Comprehensive effect system refactor: unified RON/builder vocabulary, new wrappers (`When`/`During`/`Until`/`Spawned`), new targets (`EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts`), named trigger participants, `Stamp`/`Transfer` terminals, typestate builder with `Reversible` trait enforcement, unified damage/death pipeline. Clean room implementation in `src/new_effect/`, swap when complete.

## Design Documents

| File | Contents |
|------|----------|
| [api-reference.md](api-reference.md) | Full trigger/target/terminal tables, rename mapping, participant enums |
| [builder-design.md](builder-design.md) | Typestate machine, Rust types, builder API, RON format, validation rules |
| [storage-and-dispatch.md](storage-and-dispatch.md) | BoundEffects/StagedEffects/SpawnedRegistry shape, walk_effects, condition monitor, command extensions |
| [death-pipeline.md](death-pipeline.md) | DamageMessage, KilledBy, KillYourself/Destroyed, bridge_destroyed, kill attribution chain |
| [examples.md](examples.md) | Builder + RON side-by-side for every pattern |
| [decisions.md](decisions.md) | All 22 resolved design decisions with rationale |
| [implementation-waves.md](implementation-waves.md) | Build order, parallelism, what each wave produces |
| [phase-6-swap-spec.md](phase-6-swap-spec.md) | Complete swap spec: every system, message, and file that changes outside src/effect/ |

## Research

| File | Contents |
|------|----------|
| [research/transfer-effects-flow.md](research/transfer-effects-flow.md) | Dispatch pipeline trace, insertion points, BoundEffects structure |
| [research/added-bolt-observer-feasibility.md](research/added-bolt-observer-feasibility.md) | Added<T> timing, component availability, spawn paths |

## The Problems (confirmed real)

1. **Late-spawned bolts miss AllBolts effects.** Bolts spawned mid-node by effects with `inherit: false` never get AllBolts effects.
2. **Duplicate stamping.** Second breaker spawn re-stamps all existing bolts.
3. **No kill attribution.** Can't express "when I kill a cell" — only "when a cell dies somewhere."
4. **Ambiguous targets.** `Bolt` as target means different things in different contexts.
5. **No scoped effects.** Can't express "speed boost for the duration of this node, reversed at end."
6. **No future-entity targeting.** Can't express "every bolt that will ever exist during this node."

## Key Design Decisions (summary)

Full details with rationale in [decisions.md](decisions.md).

- **Unified vocabulary**: Route/When/During/Until/Spawned/On/Fire/Stamp/Transfer/Sequence
- **Route** = definition-level routing to BoundEffects (permanent). Required root of every `effects: []` entry.
- **Stamp** = runtime terminal to BoundEffects (permanent). **Transfer** = runtime terminal to StagedEffects (one-shot).
- **Fire** implicitly targets This. **On** only for non-This participant redirects.
- **Shared participant enums**: BumpTarget, ImpactTarget, DeathTarget, BoltLostTarget (grouped by concept, not per-trigger)
- **HashMap-indexed storage**: BoundEffects keys by trigger + condition. StagedEffects keys by trigger.
- **Dispatch order**: StagedEffects first, then BoundEffects (prevents arm+consume in same call)
- **During is first-class** (condition monitor, not desugared to synthetic triggers)
- **Until desugars to Once(trigger, Reverse(effect))**
- **Kill attribution propagates** through effect chains via DamageMessage + KilledBy
- **Bridge systems are the dispatch** — each trigger has its own Bevy system calling shared `walk_effects`
- **Sequence** for ordered multi-fire (only when ordering matters)
- **Detach on transfer** — no cross-entity cleanup tracking

## Implementation Strategy

**Clean room** in `src/new_effect/`. The old `src/effect/` is not used as reference material — the design docs above are the source of truth. Both modules compile side-by-side. When `new_effect/` is complete and tested, Phase 6 swaps them.

See [implementation-waves.md](implementation-waves.md) for the full build order with parallelism analysis.

```
src/new_effect/
  builder/        # Route, Stamp, Transfer, Fire, EffectDef, EffectTree
  triggers/       # bump/, impact/, death/, bolt_lost/ (shared participant enums)
  tree/           # ValidTree, ValidDef, Raw types
  loader/         # RON -> Raw -> builder -> ValidDef
  dispatch/       # walk_effects, bridge systems, condition monitor
  damage/         # DamageMessage, KilledBy, apply_damage, detect_deaths, PendingDespawn
  effects/        # SpeedBoost, Shockwave, Explode, etc. (fire/reverse impls)
```

## Scope

**In:**
- All design from absorbed todo #7 (Killed/Die/death pipeline, DamageMessage, TriggerContext, domain handlers, bridge systems)
- Unified damage message + kill attribution chain
- `Spawned(EntityType)` trigger (implicit target)
- `During(NodeActive)` with `Reversible` enforcement (first-class, condition monitor)
- `EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts` targets (+ Cell/Wall/Breaker equivalents)
- Named trigger participants (BumpTarget, ImpactTarget, DeathTarget, BoltLostTarget)
- `Route` definition routing, `Stamp`/`Transfer` runtime terminals
- `Sequence` ordered multi-fire
- Unified RON/builder vocabulary
- Typestate effect builder with `Reversible` marker trait
- `Raw -> Builder -> Valid` + `Valid -> Raw -> RON` round-trip
- 4 bridge systems for `Added<Bolt/Cell/Wall/Breaker>`
- SpawnedRegistry resource for EveryBolt desugaring
- Source tracking (SourceId) for chip unequip cleanup
- Runtime recursion depth limit (MAX_DISPATCH_DEPTH = 10)
- PendingDespawn unified entity cleanup
- RON migration (55 files — validated, see ron-migration/)
- Tests

**Out:**
- Changing inherit behavior
- Co-op breaker spawning
- Content generation tooling (Phase 7 — builder becomes its API)
- During reacting to pause
- During reversing on source entity despawn
- Reorganizing effect types under unified `Spawn(...)` variant

## Dependencies
- Depends on: Nothing specific
- Blocks: Future co-op / breaker clone mechanics, content generation tooling (Phase 7)
- Absorbs: todo #7 (Killed trigger / unified death messaging)

## Status
`ready` — 22 design decisions resolved, 8 design documents complete, implementation waves defined with parallelism analysis. See [decisions.md](decisions.md) for all resolved questions, [implementation-waves.md](implementation-waves.md) for build order.
