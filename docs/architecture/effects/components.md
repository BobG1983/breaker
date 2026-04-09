# Storage Components

## BoundEffects (Component)

Permanent effect trees. Populated by Route at chip equip time and Stamp at runtime. Never consumed — When entries re-arm after firing.

```rust
#[derive(Component, Default)]
struct BoundEffects {
    /// Trigger-keyed entries. Linear scan on trigger match.
    /// Flat Vec — Trigger contains f32 variants that can't impl Hash/Eq.
    triggers: Vec<BoundEntry<Trigger>>,

    /// Condition-keyed entries. Read by condition monitor systems.
    conditions: Vec<BoundEntry<Condition>>,

    /// Reverse index for chip unequip cleanup.
    /// Maps SourceId to entry indices for fast removal.
    sources: HashMap<SourceId, Vec<BoundKey>>,
}

struct BoundEntry<K> {
    key: K,
    source: SourceId,
    tree: ValidTree,
}

enum BoundKey {
    Trigger(usize),
    Condition(usize),
}
```

Both `BoundEffects` and `StagedEffects` are **always present** on any entity that participates in the effect system. Neither is optional. Trigger systems query `(Entity, &BoundEffects, &mut StagedEffects)` without `Option`.

### What goes in BoundEffects

| Origin | When | Example |
|---|---|---|
| Route at chip equip | Equip command | `Route(Bolt, When(Impacted(Cell), Fire(Shockwave(...))))` |
| Route at breaker spawn | Breaker spawn | `Route(Breaker, When(BoltLostOccurred, Fire(LoseLife)))` |
| Stamp terminal at runtime | Trigger fires | `On(ImpactTarget::Impactee, Stamp(When(Died, Fire(Explode(...)))))` |
| Until reversal | Until fires | `Once(Died, Reverse(SpeedBoost(1.5)))` |
| During nested When | Condition activates | Scope-registered When entries |

### What does NOT go in BoundEffects

- Transfer payloads → go to StagedEffects (one-shot)
- Armed inner trees from nested When → go to StagedEffects
- Fire terminals → execute immediately, not stored

**Never consumed by trigger evaluation** — entries persist and re-evaluate each time a matching trigger fires. When entries re-arm. Once entries self-remove after first match.

## StagedEffects (Component)

Armed inner trees waiting for a trigger match. Populated by Transfer at runtime and by nested When arming. Consumed when triggered.

```rust
#[derive(Component, Default)]
struct StagedEffects {
    /// Armed entries. Linear scan on trigger match.
    /// Consumed (removed) when the trigger fires.
    entries: Vec<StagedEntry>,
}

struct StagedEntry {
    trigger: Trigger,
    source: SourceId,
    tree: ValidTree,
}
```

**Always present, may be empty.** An empty StagedEffects means no armed chains are pending — the normal state between trigger evaluations.

### What goes in StagedEffects

| Origin | When | Consumed when |
|---|---|---|
| Nested When arming | Outer trigger fires | Inner trigger fires |
| Transfer terminal | Trigger fires | Transferred tree's trigger fires |
| Until desugaring | Until fires immediately | Reversal trigger fires |

## OnSpawnEffectRegistry (Resource)

Global registry of Spawned listeners. Populated by EveryBolt desugaring at chip equip time. Read by bridge systems on `Added<T>`.

```rust
#[derive(Resource, Default)]
struct OnSpawnEffectRegistry {
    /// EntityType → trees to stamp onto new entities of that type.
    /// HashMap is fine — EntityType impls Hash + Eq.
    entries: HashMap<EntityType, Vec<SpawnedEntry>>,
}

struct SpawnedEntry {
    source: SourceId,
    tree: ValidTree,
}
```

## Why Flat Vec, Not HashMap

`Trigger` contains `f32` variants (`TimeExpires`, `NodeTimerThresholdOccurred`) which don't implement `Hash` or `Eq`. Using `HashMap<Trigger, ...>` would require wrapping all f32s in `OrderedFloat<f32>`, polluting the entire type tree with a dependency.

Flat Vec with linear scan on `trigger == key` avoids this entirely. Performance is equivalent — chip effect counts are in single digits per entity.

## Command Extensions

Effect execution and cross-entity mutation are deferred through `EffectCommandsExt` on Bevy `Commands`. Bridge systems take `Commands` as a parameter and call extension methods (fire_effect, stamp_effect, transfer_effect, etc.). Each command implements `EntityCommand` and executes with `&mut World` when Bevy flushes the command queue.

See [Commands](commands.md) for the full EffectCommandsExt trait.
