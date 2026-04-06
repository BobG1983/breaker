# Effect Desugaring: Spawn Trigger + SpawnedEntity Target

## Summary
Add a `Spawn(EntityType)` trigger, `SpawnedEntity` target, and `PrimaryBolt`/`ExtraBolt` target variants. `Spawn(Bolt)` fires for future bolts only (via `Added<Bolt>`). `AllBolts` remains for stamping existing bolts. No new resources or auto-stamp systems — the trigger system handles everything.

## The Problem (confirmed real)
1. **Late-spawned bolts miss AllBolts effects.** A chip says `AllBolts: SpeedBoost`. Currently desugared to per-entity `BoundEffects` at dispatch time. Bolts spawned mid-node by SpawnBolts/MirrorProtocol (with `inherit: false`) miss it entirely.
2. **Duplicate stamping.** If a second breaker spawns, it re-dispatches and double-stamps effects on existing bolts.

## Solution

### Semantics

| Target / Trigger | What it does |
|-----------------|-------------|
| `AllBolts` | Stamp all existing bolts RIGHT NOW (point-in-time snapshot, current behavior) |
| `Spawn(Bolt)` + `SpawnedEntity` | Stamp each bolt AS IT APPEARS in the future (via `Added<Bolt>` bridge) |
| `PrimaryBolt` | Resolves to entities with `PrimaryBolt` marker only |
| `ExtraBolt` | Resolves to entities with `ExtraBolt` marker only |
| Both together | `AllBolts` for existing + `Spawn(Bolt)` for future = full coverage |

A chip that wants "every bolt, current and future, gets SpeedBoost" would express:
```ron
// Stamp existing bolts at node start
When(NodeStart, On(AllBolts, Do(SpeedBoost)))
// Stamp future bolts as they spawn
When(Spawn(Bolt), On(SpawnedEntity, Do(SpeedBoost)))
```

Or if we want syntactic sugar, a new `AllBoltsAlways` (or similar) could desugar to both. But the explicit two-trigger form is clear and avoids hidden magic.

### `SpawnedEntity` — only valid inside `Spawn()` triggers

`SpawnedEntity` resolves to the specific entity that triggered the `Spawn` event. It is ONLY valid inside a `Spawn(...)` trigger context.

**Validation**: runtime, at RON load or dispatch time. If `SpawnedEntity` appears outside a `Spawn(...)` trigger, reject with a clear error. Compile-time enforcement would require splitting the effect tree type system — not worth the refactor.

### `Spawn(Bolt)` does NOT fire retroactively

If a `Spawn(Bolt)` trigger is registered late (after bolts already exist), it does NOT stamp existing bolts. Only `AllBolts` does that. This keeps the semantics clean:
- `AllBolts` = "existing bolts now"
- `Spawn(Bolt)` = "future bolts as they appear"

### Spawn ordering guarantee

`setup_run` controls spawn order: breaker spawns first → effects dispatched → bolts/cells/walls spawn. By the time `Added<Bolt>` fires for the primary bolt, the breaker's `When(Spawn(Bolt), ...)` trigger is already in its `BoundEffects`.

### New Types

```rust
/// Trigger variant: fires when an entity of the given type is added.
enum Trigger {
    // ... existing variants ...
    Spawn(EntityType),
}

enum EntityType {
    Bolt,
    Cell,
    Wall,
    Breaker,
}

/// Target variants
enum Target {
    // ... existing variants ...
    SpawnedEntity,   // the entity that caused the Spawn trigger (runtime-validated)
    PrimaryBolt,     // entities with PrimaryBolt marker
    ExtraBolts,      // entities with ExtraBolt marker
    // AllBolts remains unchanged — both PrimaryBolt + ExtraBolt
}
```

### Dedup

Each bolt spawns once → `Added<Bolt>` fires once → trigger fires once. No dedup component needed for `Spawn(Bolt)`.

For `AllBolts` (existing behavior), the existing duplicate-stamping problem (issue #2) still needs a fix. Options:
- Source_id dedup on `push_bound_effects` (chip_name + effect_index — pre-desugaring index available at dispatch time per research)
- Or: only dispatch AllBolts effects once per chip (track "already dispatched" per chip in a resource)

## Implementation Plan

### Step 1: Add Spawn trigger variant + EntityType enum
In trigger type definitions.

### Step 2: Add SpawnedEntity + PrimaryBolt + ExtraBolts target variants
In target type definitions. Add runtime validation: SpawnedEntity outside Spawn trigger → error at RON load.

### Step 3: Add bridge systems
- `bridge_bolt_spawn`: query `Added<Bolt>`, evaluate BoundEffects on all entities for `When(Spawn(Bolt), ...)` matches, fire with SpawnedEntity → the new bolt
- Same for `bridge_cell_spawn`, `bridge_wall_spawn`, `bridge_breaker_spawn`
- Register in FixedUpdate, after entity spawn systems

### Step 4: Add target resolution for PrimaryBolt / ExtraBolts
Wherever `AllBolts` is resolved (the entity query), add resolution for:
- `PrimaryBolt` → `query_filtered::<Entity, With<PrimaryBolt>>()`
- `ExtraBolts` → `query_filtered::<Entity, With<ExtraBolt>>()`

### Step 5: Fix duplicate stamping for AllBolts
Add source_id dedup to `push_bound_effects`: (chip_name, pre-desugaring effect_index). Skip if entity already has this source_id. Pre-desugaring index is available at dispatch time (confirmed by research).

### Step 6: Update RON definitions
Where chips/breakers currently use `AllBolts` and need future coverage, add `Spawn(Bolt)` trigger alongside. Audit existing definitions to determine which need both.

### Step 7: Tests
- Late-spawned bolt gets `Spawn(Bolt)` effect
- Existing bolts get `AllBolts` effect but NOT `Spawn(Bolt)` retroactively
- `SpawnedEntity` outside `Spawn` trigger → error
- `PrimaryBolt` resolves to primary only
- `ExtraBolts` resolves to extras only
- Dedup: second breaker dispatch doesn't double-stamp AllBolts

## What We DON'T Need
- ~~AllBoltsEffects / AllCellsEffects resources~~
- ~~Auto-stamp systems~~
- ~~ReceivedEffectSources dedup component~~

## Research (still valid)
- [research/transfer-effects-flow.md](research/transfer-effects-flow.md) — dispatch pipeline, insertion points, BoundEffects structure, source_id availability
- [research/added-bolt-observer-feasibility.md](research/added-bolt-observer-feasibility.md) — Added<T> timing, component availability, spawn paths

## Scope
- In: `Spawn(EntityType)` trigger, `SpawnedEntity` target (runtime-validated), `PrimaryBolt`/`ExtraBolts` targets, 4 bridge systems, source_id dedup for AllBolts, RON definition audit, tests
- Out: Changing inherit behavior, co-op breaker spawning, compile-time SpawnedEntity validation, AllBoltsAlways sugar

## Dependencies
- Depends on: Nothing specific
- Blocks: Future co-op / breaker clone mechanics

## Status
`ready` — design is clean. Research confirms Added<Bolt> timing and dispatch insertion points.
