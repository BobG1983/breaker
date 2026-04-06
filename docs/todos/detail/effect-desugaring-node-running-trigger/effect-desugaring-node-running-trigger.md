# Effect System Refactor: Typestate Builder, Spawn/During Triggers, Unified Vocabulary

## Summary
Comprehensive effect system refactor: unified RON/builder vocabulary, new wrappers (`When`/`During`/`Until`/`Spawned`), new targets (`EveryBolt`/`ActiveBolts`/`PrimaryBolts`/`ExtraBolts`/`This`), named trigger participants, `transfer()` terminal, typestate builder with `Reversible` trait enforcement, and the unified death pipeline (absorbed from todo #7). RON and builder share the same names. Builder validates at construction time; RON goes through the builder at load time.

## Design Documents

| File | Contents |
|------|----------|
| [api-reference.md](api-reference.md) | Full trigger/target/terminal tables, rename mapping, reversibility catalog |
| [builder-design.md](builder-design.md) | Typestate machine, Rust types, builder examples, RON format, validation rules |
| [death-pipeline.md](death-pipeline.md) | Killed/Died/DeathOccurred triggers, KillYourself/Destroyed messages, DamageDealt<T>, domain handlers |

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
Each trigger defines named participants: `PerfectBumped::Bolt`, `Died::Killer`, `Impacted::Target`. Typestate enforces only valid participants for the current trigger. See [api-reference.md](api-reference.md) for full table.

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

### Separate crate: `breaker-effects`
The entire new effect system is built as a separate `breaker-effects/` crate at workspace root. Developed in isolation with full tests before touching `breaker-game`. The swap: delete `breaker-game/src/effect/`, add `breaker-effects` dependency, rewire. RON migration happens at swap time.

Structure:
```
breaker-effects/
  src/
    builder/        # Route, Stamp, Transfer, Fire, EffectDef, EffectTree
    triggers/       # bump/, impact/, death/, bolt_lost/ (shared participant enums)
    tree/           # ValidTree, ValidDef, ValidTerminal, Raw types
    loader/         # RON -> builder -> ValidDef
    dispatch/       # fire_effect, stamp_effect, transfer_effect, reverse_effect
```

`breaker-game` depends on `breaker-effects`. `breaker-effects` depends on `breaker-game` types (messages, components) — or a shared types crate if circular deps arise.

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
- RON migration (~17 files)
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

## Status
`[NEEDS DETAIL]` — design is mostly complete. Remaining: (1) full participant catalog needs verification against bridge system implementations, (2) typestate builder Rust internals (PhantomData threading, associated types for participants), (3) exact `Raw` struct definitions for RON round-trip. See individual design docs for detailed status.
