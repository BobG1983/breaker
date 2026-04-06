# Effect Desugaring: Persistent Effect Rules + Auto-Stamping

## Summary
Replace the current "stamp AllBolts effects once at dispatch time" approach with persistent per-type effect stores (`AllBoltsEffects`, `AllCellsEffects`, etc.) that auto-stamp new entities via `Added<T>` observers and stamp existing entities at write time.

## The Problem (confirmed real)
1. **Late-spawned bolts miss AllBolts effects.** A chip says `AllBolts: SpeedBoost`. Currently desugared to per-entity `BoundEffects` at dispatch time. Bolts spawned mid-node by SpawnBolts/MirrorProtocol (with `inherit: false`) miss it entirely.
2. **Duplicate stamping.** If a second breaker spawns, it re-dispatches and double-stamps effects on existing bolts.

## Proposed Solution: Per-Type Effect Store Resources

### Resources (one per target type)

```rust
/// Persistent store of effects that apply to all bolt entities during this node.
#[derive(Resource, Default)]
pub struct AllBoltsEffects(pub Vec<StoredEffect>);

#[derive(Resource, Default)]
pub struct AllCellsEffects(pub Vec<StoredEffect>);

#[derive(Resource, Default)]
pub struct AllWallsEffects(pub Vec<StoredEffect>);

#[derive(Resource, Default)]
pub struct AllBreakersEffects(pub Vec<StoredEffect>);

pub struct StoredEffect {
    pub effect: EffectEntry,
    pub source_id: EffectSourceId,  // for dedup
}
```

### Two-Way Stamping

**At dispatch time** (in `transfer_effects` command or equivalent):
When the dispatcher encounters `Target::AllBolts`:
1. Push the effect into `AllBoltsEffects` resource
2. Query all existing `With<Bolt>` entities and stamp them immediately
3. Dedup: skip entities that already have this source_id

**On entity spawn** (via `Added<Bolt>` observer/system):
When a new bolt appears mid-node:
1. Read `AllBoltsEffects` resource
2. Stamp all stored effects onto the new bolt
3. No dedup needed — the entity is brand new, it has no effects yet

Same pattern for cells, walls, breakers.

### Cleanup
All four resources are cleared (or removed) on node exit. Effects don't carry between nodes.

### Why This Works

| Scenario | What happens |
|----------|-------------|
| Bolt exists at node start, chip has AllBolts: SpeedBoost | Dispatch stamps SpeedBoost + stores in AllBoltsEffects |
| SpawnBolts creates bolt mid-node (inherit: false) | Added<Bolt> fires, reads AllBoltsEffects, stamps SpeedBoost |
| MirrorProtocol creates bolt mid-node (inherit: true) | Inherit copies parent effects. Added<Bolt> also stamps AllBoltsEffects rules. Dedup prevents doubles. |
| Second breaker spawns with same definition | Dispatch tries to push same effect to AllBoltsEffects — source_id dedup rejects it. Existing entities already have it. |

### Inherit Remains Unchanged
`inherit: true` still propagates parent→child bolt effects. The AllBoltsEffects store is complementary:
- **inherit**: "bolt B gets bolt A's effects" (parent relationship)
- **AllBoltsEffects**: "all bolts get these effects" (node-wide rule)
- A bolt with `inherit: true` gets both. Dedup via source_id on the entity prevents doubles.

## Scope
- In: 4 new resources (AllBoltsEffects, AllCellsEffects, AllWallsEffects, AllBreakersEffects), dispatch changes for All* targets, `Added<Bolt/Cell/Wall/Breaker>` observer systems, per-entity dedup tracking, cleanup on node exit, AllBreakers target type
- Out: Changing inherit behavior, co-op breaker spawning mechanics, removing per-entity BoundEffects (they still exist for non-All targets like Self/Source)

## Dependencies
- Depends on: Nothing specific
- Blocks: Future co-op / breaker clone mechanics

## Open Questions
- **Source_id structure**: definition name + pre-desugaring effect index is the plan. Need to verify pre-desugaring indexes are accessible in `transfer_effects`. If not, consider hash of effect config or assign IDs at RON load time.
- **Per-entity dedup tracking**: a `ReceivedEffectSources(HashSet<EffectSourceId>)` component on each bolt/cell/wall? Or track on the resource side as `(Entity, SourceId)` pairs? Per-entity component is cleaner.
- **Added<T> observer timing**: must fire after the entity has all its components. Bevy observers fire after command application, so this should be safe — verify.
- **Where exactly in transfer_effects**: need to read the dispatch/transfer code to find the right insertion point.

## Status
`[NEEDS DETAIL]` — source_id stability and transfer_effects insertion point need code investigation. Core design is decided.
