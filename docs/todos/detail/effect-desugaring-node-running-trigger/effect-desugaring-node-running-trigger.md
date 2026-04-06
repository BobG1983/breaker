# Effect System Refactor: Typestate Builder, Spawn/During Triggers, Unified Vocabulary

## Summary
Comprehensive effect system refactor: unified RON/builder vocabulary, new wrappers (`When`/`During`/`Until`/`Spawned`), new targets (`EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts`/`This`), named trigger participants, `transfer()` terminal, typestate builder with `Reversible` trait enforcement, and the unified death pipeline (absorbed from todo #7). RON and builder share the same names. Builder validates at construction time; RON goes through the builder at load time.

## Design Documents

| File | Contents |
|------|----------|
| [api-reference.md](api-reference.md) | Full trigger/target/terminal tables, rename mapping, reversibility catalog |
| [builder-design.md](builder-design.md) | Typestate machine, Rust types, builder examples, RON format, validation rules |
| [storage-and-dispatch.md](storage-and-dispatch.md) | BoundEffects/StagedEffects/SpawnedRegistry shape, dispatch walk, condition monitor, command extensions |
| [death-pipeline.md](death-pipeline.md) | Killed/Died/DeathOccurred triggers, KillYourself/Destroyed messages, DamageDealt<T>, domain handlers |
| [examples.md](examples.md) | Builder + RON side-by-side for every pattern |

## Research

| File | Contents |
|------|----------|
| [research/transfer-effects-flow.md](research/transfer-effects-flow.md) | Dispatch pipeline trace, insertion points, BoundEffects structure |
| [research/added-bolt-observer-feasibility.md](research/added-bolt-observer-feasibility.md) | Added<T> timing, component availability, spawn paths |

## The Problems (confirmed real)

1. **Late-spawned bolts miss AllBolts effects.** Bolts spawned mid-node by effects with `inherit: false` never get AllBolts effects.
2. **Duplicate stamping.** Second breaker spawn re-stamps all existing bolts.
3. **No kill attribution.** Can't express "when I kill a cell" — only "when a cell dies somewhere."
4. **Ambiguous targets.** `Bolt` as target means different things in different contexts. `This` on a local trigger is ambiguous.
5. **No scoped effects.** Can't express "speed boost for the duration of this node, reversed at end."
6. **No future-entity targeting.** Can't express "every bolt that will ever exist during this node."

## Key Design Decisions

### Unified vocabulary (RON = builder)
`When`, `During`, `Spawned`, `On`, `Fire`, `Transfer`, `This`. No divergence between data format and code API.

### Named trigger participants (not `Other`)
Each trigger defines named participants: `BumpTarget::Bolt`, `DeathTarget::Killer`, `ImpactTarget::Impactee`. Typestate enforces only valid participants for the current trigger. See [api-reference.md](api-reference.md) for full table.

### `This` = bound entity (distinct from participants)
`This` always means "the entity BoundEffects lives on." Named participants come from the trigger event. Same entity, different semantic source.

### `Occurred` suffix for globals
Local: `PerfectBumped` (past tense, "I was bumped"). Global: `PerfectBumpOccurred` (unambiguous, heavier name signals broader scope).

### `During` / `Until` reversibility depends on nesting
`During(X, On(target, Fire(effect)))` and `Until(X, On(target, Fire(effect)))` — effect must be `Reversible`. Wrapping a nested `When` relaxes the constraint — inner effects can be anything (reversal removes the trigger registration, not individual firings).

### `During` vs `Until`
`During(condition)` = state-scoped (fires on condition start, reverses on condition end). Takes a **condition**: `NodeActive`, `NodePlaying`.
`Until(trigger)` = event-scoped (fires immediately, reverses when trigger fires). Takes any **trigger**: `Died`, `BoltLostOccurred`, etc.

### Conditions: `NodeActive` + `NodePlaying`
`NodeActive` = node start through teardown (ignores pause). Most common.
`NodePlaying` = only while `NodeState::Playing` (respects pause, toggles on/off). Niche.

### `Spawned()` does NOT fire retroactively
`Spawned(Bolt)` = future only. `ActiveBolts` = existing only. `EveryBolt` = both (desugars to `ActiveBolts` + `Spawned(Bolt)`).

### `Route` is definition-level routing (required root)
Every entry in a chip/breaker/evolution `effects: []` list must be wrapped in `Route(target, tree)`. Route routes the inner tree to the target entity's **BoundEffects** (permanent, re-arming). Entity targets: `Bolt`, `Breaker`, `EveryBolt`, `ActiveBolts`, etc. No `This` — Route determines what `This` means for the subtree.

### `Stamp` and `Transfer` are runtime terminals
Both are reached via `On(participant, Stamp/Transfer(tree))`. They differ in permanence:
- **`Stamp(tree)`** → target's **BoundEffects** (permanent, re-arming). "Always explode on death" survives multiple lives.
- **`Transfer(tree)`** → target's **StagedEffects** (one-shot, consumed when triggered). "Explode on next death" fires once and is gone.

### `Fire` implicitly targets `This`
`Fire(effect)` always fires on the entity whose BoundEffects/StagedEffects contains the tree. `On(participant, ...)` is ONLY used to redirect to a non-This target (trigger participants). `On(This, ...)` never appears.

### Route vs Stamp vs Transfer = permanence spectrum
- **Route** → BoundEffects: permanent, definition-time routing. Required root of every `effects: []` entry.
- **Stamp** (terminal) → BoundEffects: permanent, runtime add via trigger. Re-arms.
- **Transfer** (terminal) → StagedEffects: one-shot, runtime add via trigger. Consumed on match.

### Existing command extensions reworked, not replaced
`EffectCommandsExt` (`fire_effect`, `transfer_effect`, `push_bound_effects`, `dispatch_initial_effects`) stays as the execution layer. Changes:
- `fire_effect` → add TriggerContext parameter (for damage attribution)
- Add `reverse_effect` command (calls `.reverse()` on a `Reversible` effect)
- `transfer_effect` → writes to StagedEffects (one-shot)
- Add `stamp_effect` command (writes to BoundEffects at runtime — permanent)
- `Do` → `Fire` in the tree enum (the command doesn't care about the name)
- `dispatch_initial_effects` → rework to use new tree format

### BoundEffects + StagedEffects (no new components)
- **BoundEffects**: permanent effect tree definitions. Never consumed. Populated by `Route` at load time and `Stamp` terminal at runtime.
- **StagedEffects**: armed inner trees waiting for a trigger match. Consumed when matched. Populated by `Transfer` terminal at runtime, and by During/Until reversal entries.
- **No ActiveEffects component** — During/Until stage `Reverse(effect)` entries into StagedEffects. When the end-trigger fires, the staged entry matches and calls `.reverse()`.
- `Reverse(EffectType)` is an internal-only terminal — never appears in RON or the builder. Generated by During/Until when they desugar.

### Parallel module: `src/new_effect/` → swap
The new effect system is built as `src/new_effect/` alongside the existing `src/effect/`. Both compile, neither references the other. `new_effect/` can freely use all game types (components, builders, messages) because it's in the same crate — no circular dep issues.

When complete: delete `src/effect/`, rename `new_effect` → `effect`, update imports, migrate RON files — all in one swap commit.

Separate crate was considered but rejected: `fire()` implementations need game components (`BoltSpeed`, `BreakerSpeed`) and builders (`Bolt::builder()`), creating circular deps that aren't worth solving with trait indirection.

Structure:
```
src/new_effect/
  builder/        # Route, Stamp, Transfer, Fire, EffectDef, EffectTree
  triggers/       # bump/, impact/, death/, bolt_lost/ (shared participant enums)
  tree/           # ValidTree, ValidDef, Raw types
  loader/         # RON -> builder -> ValidDef
  dispatch/       # walk_effects, bridge systems, condition monitor
  damage/         # DamageMessage, KilledBy, Hp, apply_damage, detect_deaths
  effects/        # SpeedBoost, Shockwave, Explode, etc. (fire/reverse impls)
```

### Effect types stay as-is
`SpawnBolts`, `Shockwave`, `ChainBolt`, etc. each keep their own module, `fire()`, and params. The refactor is triggers/targets/builder, not effect reorganization.

### `SpawnBolts` + `SpawnPhantom` gain optional definition override
`definition: Option<String>` — `None` inherits source bolt definition (current behavior), `Some("FastBolt")` overrides.

### Died fires on victim only, Killed fires on killer only
Same event, two triggers, opposite perspectives. Both have `::Victim` and `::Killer` participants for targeting.

## Scope

**In:**
- All design from absorbed todo #7 (Killed/Die/death pipeline, DamageDealt<T>, TriggerContext, domain handlers, bridge systems, RON migration)
- `Spawned(EntityType)` trigger (implicit target)
- `During(NodeActive)` scoped trigger with `Reversible` enforcement
- `EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts` targets (+ Cell/Wall/Breaker equivalents)
- Named trigger participants per trigger type
- `This` as bound-entity target
- `transfer()` / `transfer_to()` terminals
- Unified RON/builder vocabulary (`When`/`During`/`Spawned`/`On`/`Fire`/`This`)
- Typestate effect builder with `Reversible` marker trait
- `Raw → Builder → Valid` + `Valid → Raw → RON` round-trip
- 4 bridge systems for `Added<Bolt/Cell/Wall/Breaker>`
- Reversal system for `During`
- Source_id dedup for `ActiveBolts`
- Runtime recursion depth limit for spawn chains
- `SpawnBolts` + `SpawnPhantom` optional definition override
- Global trigger rename (`Occurred` suffix)
- RON migration (55 files — validated, see ron-migration/)
- Tests

**Post-refactor docs:**
- Update `docs/architecture/` with: how to add a new trigger (create participant enum, add bridge system, register), how to add a new effect (implement `Effect` trait, optionally `Reversible`, add to `EffectType` enum), how to add a new condition (for `During`)
- Document the type tree and where each type lives (trigger participant enums in `effect/triggers/<trigger>/`, effect types in `effect/effects/<effect>/`)

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

## Resolved Design Decisions

### 1. Dispatch mechanics
**HashMap-indexed storage.** BoundEffects and StagedEffects both use `HashMap<Trigger, Vec<(SourceId, ValidTree)>>`. When a trigger fires, look up the key, get matching trees, walk them. No separate index — the storage IS the index. For local triggers, fire on both participant entities if they have matching BoundEffects/StagedEffects entries.

### 2. TriggerContext
**Typed per-trigger structs.** Each trigger concept has its own context struct with named fields. `BumpContext { bolt, breaker, source }`, `ImpactContext { impactor, impactee, source }`, `DeathContext { victim, killer, source }`, `BoltLostContext { bolt, breaker, source }`. Wrapped in `enum TriggerContext { Bump(BumpContext), Impact(ImpactContext), Death(DeathContext), BoltLost(BoltLostContext), None }`.

### 3. During/Until reversal
**During is first-class, not desugared.** During stays as `During(condition, inner)` in BoundEffects. A condition-monitoring system watches for NodeState changes and fires/reverses During entries directly. No synthetic triggers (NodeActiveStarted/NodeActiveEnded don't exist). Condition cycling is handled by the monitor system.

**Until desugars to Once.** Until fires immediately, then inserts `Once(trigger, Reverse(effect))` into BoundEffects. One-shot reversal that self-removes after firing. Uses real triggers (Died, TimeExpires, etc.).

**Sequence in scoped context** produces paired reversals: `During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))` → on condition start both fire, on condition end both reverse via `Sequence([Reverse(SpeedBoost), Reverse(DamageBoost)])`.

### 4. Once self-removal
**Remove inline during dispatch.** Dispatch uses `retain()` on the Vec — Once entries return false (removed), When entries return true (kept). Practically may need collect-then-remove due to ownership/borrow constraints, but same-frame semantics.

### 5. EveryBolt desugaring
**Desugar at load time.** `Route(EveryBolt, tree)` expands to: (1) stamp tree onto all existing bolts via ActiveBolts query, (2) register tree in `SpawnedRegistry` resource for future bolts. SpawnedRegistry is a global `Resource<SpawnedRegistry>` holding `HashMap<EntityType, Vec<(SourceId, ValidTree)>>`.

### 6. Source tracking
**Chip definition name (String).** `type SourceId = String`. BoundEffects entries are `(SourceId, ValidTree)` pairs. Reverse index `HashMap<SourceId, Vec<Trigger>>` enables fast removal on chip unequip. SpawnedRegistry also tracks SourceId for cleanup.

### 7. Kill attribution — propagated through effect chains
**KilledBy propagates from TriggerContext.** `KilledBy { dealer: Option<Entity> }`. The dealer is the originating bolt entity, propagated through effect chains:
- Bolt hits cell → `dealer: Some(bolt)`
- Bolt's shockwave kills cell → `dealer: Some(bolt)` (shockwave inherits from spawning bolt)
- Bolt's chain lightning kills cell → `dealer: Some(bolt)` (arc inherits from source)
- Powder keg: bolt B kills cell → cell explodes → `dealer: Some(bolt_B)` (from DeathContext.killer, not the transferring bolt) → explosion kills cell C → `dealer: Some(bolt_B)`
- Environmental/timer hazard → `dealer: None` → Killed doesn't fire, Died + DeathOccurred still fire

`DeathContext { victim: Entity, killer: Option<Entity> }`. When killer is None, `Killed(Cell)` is skipped (no entity to fire on). `Died` always fires on victim. `DeathOccurred(Cell)` always fires globally.

TriggerContext flows through `fire_effect` so effects can read the killer entity and stamp it as KilledBy on whatever they spawn/damage.

**Unified damage message:** All damage sources (bolt collision, shockwave, chain lightning, explosion) send `DamageMessage { dealer: Option<Entity>, target: Entity, amount: f32 }`. One `apply_damage` system processes them all, decrements HP, and sets `KilledBy` only on the killing blow (HP crosses from positive to zero). Earlier hits that reduce HP but don't kill do not set KilledBy.

**Corner cases:**
- **Multi-source same frame**: Message processing order determines the killing blow. Deterministic (system ordering + message queue order).
- **Dealer despawns mid-chain**: Before firing Killed on the dealer, verify entity still exists. If despawned, skip Killed silently (known valid case — bolt lost while shockwave is still expanding). Died and DeathOccurred still fire.

### 8. Bridge systems for Spawned
**4 standard systems in PostFixedUpdate** (not Bevy Observers). One per entity type: `bridge_bolt_added`, `bridge_cell_added`, `bridge_wall_added`, `bridge_breaker_added`. Each queries `Added<Bolt/Cell/Wall/Breaker>`, reads SpawnedRegistry for matching entries, stamps/transfers trees onto the new entity's BoundEffects/StagedEffects.

### 9. Build phasing
**Bottom-up: types → builder → loader → dispatch → damage → swap.**
- Phase 1: Core types (enums, tree structs, participant enums, EffectType, RouteTarget)
- Phase 2: Builder (EffectDef::route(), EffectTree, typestate, validation)
- Phase 3: Loader (RON → Raw → builder → ValidDef, round-trip)
- Phase 4: Storage + Dispatch (BoundEffects HashMap, StagedEffects, SpawnedRegistry, walk_effects, condition monitor, bridge systems)
- Phase 5: Damage + Death pipeline (DamageMessage, apply_damage, KilledBy, detect_deaths, KillYourself<S,T>, Destroyed<S,T>, bridge_destroyed, PendingDespawn)
- Phase 6: Swap — delete src/effect/, rename new_effect → effect, rewire imports, migrate RON files, replace domain-specific damage/death messaging:
  - `DamageCell` → `DamageMessage`
  - `RequestCellDestroyed` → `KillYourself<Bolt, Cell>` (and other S,T pairs)
  - `CellDestroyedAt` → `Destroyed<Bolt, Cell>`
  - `RequestBoltDestroyed` → `KillYourself<(), Bolt>`
  - `bridge_cell_destroyed` → `bridge_destroyed::<Bolt, Cell>`
  - Direct HP mutation in effects → `DamageMessage`
  - Per-domain cleanup systems → unified `PendingDespawn`

### 10. Nested When in HashMap storage
**Arm into StagedEffects.** BoundEffects keys by outer trigger (PerfectBumped). When it fires, the inner tree `When(Impacted(Cell), Fire(Shockwave))` moves to StagedEffects under the `Impacted(Cell)` key. When Impacted fires, Shockwave executes and the entry is consumed. Next PerfectBumped re-arms from BoundEffects again.

### 11. Multiple effects from one trigger — Sequence node
**Sequence for ordered execution.** New tree node `Sequence([Fire(A), Fire(B)])` executes children in order. Use when one effect must apply before another (e.g., DamageBoost before Shockwave). Independent effects stay as separate Route entries — Sequence is only for when order matters.

### 12. Transfer/Stamp tree ownership
**Detach on transfer.** Once transferred/stamped onto another entity, the tree has no link back to the source. Unequipping the chip removes the bolt's BoundEffects entries (stops future transfers) but doesn't touch entities that already received trees. No cross-entity cleanup tracking.

### 13. Chip loading → Route processing
**Equip command processes Routes.** Same timing as today. The chip equip command reads each ValidDef, matches on RouteTarget, and stamps the tree into the target entity's BoundEffects with the chip's SourceId. EveryBolt desugars here: stamp existing + register in SpawnedRegistry.

### 14. During is first-class, not desugared
During stays as a first-class node in BoundEffects. A condition-monitoring system watches for NodeState changes and fires/reverses During entries directly. No synthetic triggers (NodeActiveStarted/NodeActiveEnded don't exist). Avoids creating triggers that nothing else uses and that would miss mid-equip state (even though mid-node equip is out of scope).

### 15. RON participant syntax — fully qualified
RON uses shared enum names: `On(BumpTarget::Bolt, ...)`, `On(ImpactTarget::Impactee, ...)`. RawParticipant wraps the shared enums: `enum RawParticipant { BumpTarget(BumpTarget), ImpactTarget(ImpactTarget), DeathTarget(DeathTarget), BoltLostTarget(BoltLostTarget) }`. No flat names, no ambiguity.

### 16. Sequence in scoped context — paired reversals
`During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))` → on condition start both fire, on condition end the condition monitor reverses both via `Sequence([Reverse(SpeedBoost), Reverse(DamageBoost)])`.

### 17. BoundEffects storage for During
During entries in BoundEffects are keyed by their condition (not a trigger). BoundEffects gains a second map: `conditions: HashMap<Condition, Vec<(SourceId, ValidTree)>>` alongside `triggers: HashMap<Trigger, Vec<(SourceId, ValidTree)>>`. The condition monitor reads `conditions`, trigger dispatch reads `triggers`.

### 18. Dispatch ordering — StagedEffects first
StagedEffects is walked BEFORE BoundEffects on each trigger dispatch. Prevents a single trigger from both arming and consuming a nested When in the same call. E.g., `When(PerfectBumped, When(PerfectBumped, Fire(...)))` — first bump arms, second bump consumes. If BoundEffects walked first, both would happen in one dispatch.

### 19. During + nested When lifecycle
During's inner When is registered into `BoundEffects.triggers` on condition start, with a scope source (`"ChipName:During(NodeActive)"`). On condition end, the scope source is used to remove the registration AND any armed StagedEffects entries from it. See [storage-and-dispatch.md](storage-and-dispatch.md) for full details.

### 20. Recursion depth limit
Depth counter on TriggerContext, incremented on each sub-dispatch. MAX_DISPATCH_DEPTH = 10. Prevents infinite spawn chains.

### 21. Trigger locality — bridge systems decide
No `locality()` method on Trigger. No centralized `dispatch_trigger`. Each trigger has its own Bevy bridge system that knows its participants and scope. Local bridges walk participant entities. Global bridges query all entities with effects. All bridge systems call a shared `walk_effects` helper for the tree-walking logic.

### 22. Stale participant references
Debug warning + skip. If `On(BumpTarget::Bolt, ...)` resolves to a despawned entity, log a debug warning and skip the fire. Helps catch bugs in development, normal gameplay occurrence in production.

## Status
`ready` — all 22 design decisions resolved. See design docs: [api-reference.md](api-reference.md), [builder-design.md](builder-design.md), [storage-and-dispatch.md](storage-and-dispatch.md), [death-pipeline.md](death-pipeline.md), [examples.md](examples.md), [ron-migration/](ron-migration/).
