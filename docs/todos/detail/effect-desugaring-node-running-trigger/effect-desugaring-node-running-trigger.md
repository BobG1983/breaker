# Effect System Refactor: Typestate Builder, Spawn/During Triggers, Unified Vocabulary

## Summary
Comprehensive effect system refactor: unified RON/builder vocabulary, new wrappers (`When`/`During`/`Until`/`Spawned`), new targets (`EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts`), named trigger participants, `Stamp`/`Transfer` terminals, typestate builder with `Reversible` trait enforcement, unified damage/death pipeline. Clean room implementation in `src/new_effect/`, swap when complete.

## Design Documents

| File | Contents |
|------|----------|
| [api-reference.md](api-reference.md) | Full trigger/target/terminal tables, rename mapping, participant enums |
| [builder-design.md](builder-design.md) | Typestate machine, Rust types, builder API, RON format, validation rules |
| [storage-and-dispatch.md](storage-and-dispatch.md) | BoundEffects/StagedEffects/OnSpawnEffectRegistry shape, walk_effects, condition monitor, command extension dispatch model |
| [command-extensions.md](command-extensions.md) | EffectCommandsExt behavioral spec: fire_effect, reverse_effect, stamp_effect, transfer_effect, equip_chip, dispatch_fire/dispatch_reverse |
| [death-pipeline.md](death-pipeline.md) | DamageDealt\<T\>, KilledBy, GameEntity trait, KillYourself\<T\>/Destroyed\<T\>, bridge_destroyed, kill attribution chain |
| [examples.md](examples.md) | Builder + RON side-by-side for every pattern |
| [decisions.md](decisions.md) | All 22 resolved design decisions with rationale |
| [implementation-waves.md](implementation-waves.md) | Build order, parallelism, what each wave produces |
| [phase-6-swap-spec.md](phase-6-swap-spec.md) | Complete swap spec: every system, message, and file that changes outside src/effect/ |

## Research

| File | Contents |
|------|----------|
| [research/transfer-effects-flow.md](research/transfer-effects-flow.md) | Dispatch pipeline trace, insertion points, BoundEffects structure |
| [research/added-bolt-observer-feasibility.md](research/added-bolt-observer-feasibility.md) | Added<T> timing, component availability, spawn paths |

## Absorbed Todos

- **Todo #7** (Killed trigger / unified death messaging) — absorbed into death pipeline design
- **Todo #1** (Centralized entity despawn system) — absorbed into Wave 5d. `DespawnEntity` message replaces `PendingDespawn` component approach AND centralizes all `.despawn()`/`.try_despawn()` calls across the codebase. The message lives in `shared::messages`, the `process_despawn_requests` system runs in PostFixedUpdate after all trigger evaluation. Phase 6 sweep converts all domain despawn calls to write the message instead.

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
- **Flat Vec storage**: BoundEffects uses flat Vecs (Trigger contains f32 variants that can't impl Hash/Eq). Linear scan — counts are small.
- **Command extensions for dispatch**: walk_effects defers fire/stamp/transfer via EffectCommandsExt on Commands. Bridge systems are regular Bevy systems (not exclusive).
- **Dispatch order**: StagedEffects first, then BoundEffects (prevents arm+consume in same call)
- **During is first-class** (condition monitor, not desugared to synthetic triggers)
- **Until desugars to Once(trigger, Reverse(effect))**
- **Kill attribution propagates** through effect chains via DamageDealt<T> + KilledBy
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
  damage/         # DamageDealt<T>, GameEntity, KilledBy, apply_damage, DespawnEntity
  effects/        # SpeedBoost, Shockwave, Explode, etc. (fire/reverse impls)
```

## Scope

**In:**
- All design from absorbed todo #7 (Killed/Die/death pipeline, DamageDealt<T>, TriggerContext, domain handlers, bridge systems)
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
- Centralized `DespawnEntity` message + `process_despawn_requests` system (replaces PendingDespawn component + scattered `.despawn()` calls)
- RON migration (55 files — validated, see ron-migration/)
- Tests

**Out:**
- Changing inherit behavior (SpawnBolts inherit flag stays as-is)
- Co-op breaker spawning (multi-breaker support deferred)
- Content generation tooling (Phase 7 — builder becomes its API)
- `NodePlaying` condition (not defined in enum — add when needed. Only `NodeActive` is implemented.)
- During reversing on source entity despawn (orphaned effects stay applied until node teardown cleans everything — acceptable)
- Reorganizing effect types under unified `Spawn(...)` variant (SpawnBolts/SpawnPhantom/ChainBolt stay separate)

## Dependencies
- Depends on: Nothing specific
- Blocks: Future co-op / breaker clone mechanics, content generation tooling (Phase 7)
- Absorbs: todo #7 (Killed trigger / unified death messaging), todo #1 (centralized entity despawn)

## Resolved Implementation Gaps (2026-04-09)

25 gaps identified during clean-room implementability review. All resolved.

### Core type decisions

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **A1**: DamageDealt<T> vs DamageDealt\<T\> | **DamageDealt\<T\>** (generic per victim type) | One message queue per T. Type-safe, N specialized apply_damage\<T\> systems. Update death-pipeline.md to use DamageDealt\<T\> everywhere (remove DamageDealt<T> references). |
| **A2**: RandomEffect pool shape | **RandomEffect(Vec\<(f32, Box\<EffectType\>)\>)** | Flat EffectTypes only (no ValidTree nesting). Box needed to avoid infinite enum size. |
| **A3**: EntropyEngine pool type | **Same as RandomEffect** — Vec\<(f32, EffectType)\> | Both meta-effects use the same pool type. Consistent API. |
| **A5**: Entry point naming | **EffectDef::route()** | The `stamp()` reference in builder-design.md Rust Types section is an error. Route is definition-level, Stamp is a runtime terminal — they are different things. Fix the typo in builder-design.md. |
| **X3**: Vulnerable representation | **Vulnerable(f32)** bare | Matches SpeedBoost(f32) pattern. VulnerableConfig in the effect spec is misleading — update to match. |

### Dispatch architecture

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **B1**: World access for effect fire | **Command extensions** | Bridge systems are regular Bevy systems (not exclusive). walk_effects defers fire/stamp/transfer via EffectCommandsExt on Commands. Bevy applies commands at schedule flush points. Replicates current CommandExt pattern (clean-room behavioral spec, not code reference). |
| **B2/B3**: Cross-entity mutation + index invalidation | **Defer via Commands** | Cross-entity Stamp/Transfer deferred via command extensions. Same-entity StagedEffects drain happens inline. Once removal collected during walk, applied after walk completes (same entity, no cross-entity borrow issue). Reverse indices updated after all removals. |
| **D1/D2**: EffectType → fire dispatch | **Big match on EffectType** | dispatch_fire/dispatch_reverse functions match on every variant, call per-effect module functions. CircuitBreaker/EntropyEngine call dispatch_fire for sub-effects. Trait objects would make RON syntax ugly. |

### Death pipeline

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **C1**: detect_deaths (S,T) dispatch | **N specialized systems** per domain | detect_cell_deaths in cells/, detect_bolt_deaths in bolt/, etc. Each queries its domain's health component + marker. Killer type classified at runtime from KilledBy.dealer entity's components. |
| **C2**: No-killer type | **GameEntity trait** | `trait GameEntity: Component {}` impl'd on Bolt, Cell, Wall, Breaker. `KillYourself<T: GameEntity>` and `Destroyed<T: GameEntity>`. Skip the S generic — killer is Option\<Entity\>, type determined at runtime. Monomorphization only generates versions for types actually used. |
| **C3**: was_required_to_clear | **Query from entity** | Entity survives until DespawnEntity processes in PostFixedUpdate. track_node_completion queries Has\<RequiredToClear\> on the still-alive victim entity. No extra field on Destroyed\<T\>. |
| **C4**: Locked cell immunity | **apply_damage filter** | apply_damage for cells skips Locked entities — they can't take damage. **Critical ordering**: must run AFTER unlock system (check_lock_release) to avoid eating damage before a cell unlocks in the same frame. |
| **C5**: Deferred despawn pattern | **DespawnEntity message** | Message in shared/messages.rs. process_despawn_requests in PostFixedUpdate reads and try_despawns. Remove all PendingDespawn component references from docs. |

### Trigger dispatch

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **A4/B4**: NodeTimerThreshold matching | **Bridge scans BoundEffects** | Bridge tracks previous ratio in Local\<f32\>. Each frame, scans all entities' BoundEffects for NodeTimerThresholdOccurred(x) entries where prev \< x \<= current. Fires for each crossed threshold. |
| **B5/D6**: Chip equip integration | **Command extension** | EffectCommandsExt::equip_chip(entity, defs, source_id) and unequip_chip(entity, source_id). Chip domain calls via Commands. Effect domain provides the trait. |
| **D5**: System sets | **5 ordered sets** | EffectSystems { Damage, Death, Dispatch, Spawned, Despawn }. Ordering: Physics → Damage → Death → Dispatch → Spawned → Despawn. Damage/Death/Dispatch in FixedUpdate, Spawned/Despawn in PostFixedUpdate. |
| **D7**: Breaker-Cell collision | **Add it** | Define the messages and wire up the collision. Keep (Breaker, Cell) as a valid death pair. |

### Effect implementations

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **D4**: GravityWell velocity formula | **Direction + magnitude with falloff** | Use existing velocity helper function. Steer toward well center, preserve bolt speed. Falloff by distance. |
| **D3**: Stale DamageCell references in effect specs | **Update all specs** | shockwave.md, chain_lightning.md, etc. must reference DamageDealt\<Cell\> not DamageCell. |

### Low-severity / tooling

| Gap | Decision | Rationale |
|-----|----------|-----------|
| **A6**: Trait derives | ValidTree, EffectType, Trigger, Condition, all participant enums need Clone, Debug, PartialEq minimum. Config structs add Deserialize. Raw types add Serialize + Deserialize. |
| **A7**: Loader Sequence/Spawned | Add handling to load_tree and load_scoped_tree pseudocode in builder-design.md. |
| **X1**: drain_filter | **Available** — project uses nightly Rust. |
| **X2**: EventReader vs MessageReader | Use **MessageReader** everywhere (Bevy 0.18). Fix all pseudocode. |

## Status
`ready` — 22 original design decisions resolved. 25 implementation gaps identified and resolved (2026-04-09). Design documents need updates to reflect gap resolutions (see "Docs to update" below). 30 per-effect behavior specs, 10 per-trigger behavior specs, 4 sub-todos defined. Ready for implementation once doc updates are applied.

### Docs to update (from gap resolutions)
- **death-pipeline.md**: Replace all `DamageDealt<T>` with `DamageDealt<T>`. Remove S generic from KillYourself/Destroyed — use `T: GameEntity`. Add GameEntity trait definition.
- **builder-design.md**: Fix `EffectDef::stamp()` typo → `EffectDef::route()` in Rust Types section. Add RandomEffect(Vec<(f32, Box<EffectType>)>) to EffectType enum. Update EntropyConfig pool type. Fix Vulnerable to bare f32. Add Sequence/Spawned to loader pseudocode. Fix EventReader → MessageReader.
- **storage-and-dispatch.md**: Document command extension dispatch model (walk_effects defers via Commands, no EffectBuffer/flush_effects). Fix EventReader → MessageReader. Add NodeTimerThreshold bridge system spec.
- **implementation-waves.md**: Update Wave 4a walk_effects to use Commands. Add EffectSystems set definitions to Wave 8. Add breaker-cell collision messages. Fix DamageDealt<T> → DamageDealt<T> in Wave 1i.
- **phase-6-swap-spec.md**: ~~DamageMessage → DamageDealt<T>~~ DONE. ~~S generic removed~~ DONE. ~~PendingDespawn → DespawnEntity~~ DONE. Still needed: Add apply_damage ordering constraint (after unlock). Add breaker-cell collision wiring.
- **Effect specs** (shockwave.md, chain_lightning.md, explode.md, piercing_beam.md, pulse.md, tether_beam.md): Replace DamageCell with DamageDealt<Cell>.
- **Effect specs** (vulnerable.md): Remove VulnerableConfig, use bare f32.
- **Effect specs** (gravity_well.md): Specify direction + magnitude with falloff formula using velocity helper.
- **api-reference.md**: Fix EventReader → MessageReader in examples.
