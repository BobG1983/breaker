# Storage Components

The effect system stores tree state on entities via components, and global spawn-stamp state via a resource. All five storage types live under `effect_v3/storage/`.

## BoundEffects

Permanent effect trees on an entity. Re-arms on every matching trigger; never consumed except by explicit removal (`commands.remove_effect`, `Tree::Once` self-removal, condition disarm).

```rust
#[derive(Component, Clone, Default)]
pub struct BoundEffects(pub Vec<(String, Tree)>);
```

It is a flat tuple vec — `(source_name, tree)` pairs. No internal indexing by trigger or condition. Lookup is linear, which is fine because chip counts per entity are in single digits and the comparison is structural enum equality.

### Where entries come from

| Origin | When | Naming convention |
|---|---|---|
| `commands.stamp_effect` from chip dispatch | `ChipSelected` is read | `chip_name` |
| `commands.stamp_effect` from spawn-stamp watcher | `Added<Bolt>` etc. fires and registry has matching entries | original chip name |
| `evaluate_when` arming a nested gate | Outer When trigger matches and inner is a gate | same name as outer |
| `evaluate_during` Shape A install | During wraps a When/On (idempotent insert) | `format!("{source}#installed[0]")` |
| `evaluate_until` Shape B install | Until wraps a During | same `#installed[0]` convention |

The `#installed[0]` and `#armed[0]` suffixes are sentinels read by `is_armed_source` (in `conditions/armed_source.rs`) so that walkers and pollers can distinguish "user-authored" entries from "internally installed" ones during cleanup. The `[0]` is reserved for future multi-arm shapes.

### What does not go in

- `Tree::Fire` is a terminal — it executes immediately and is never stored.
- `Terminal::Route(Staged, ...)` lands in `StagedEffects`, not here.
- `Tree::Until` originally written by an author lives in `BoundEffects` until the first walk evaluates it — `evaluate_until` may then install a `#installed[0]` companion entry (Shape B) but the original is left in place and walked again next tick.

## StagedEffects

One-shot effect trees on an entity. Consumed when the trigger matches.

```rust
#[derive(Component, Clone, Default)]
pub struct StagedEffects(pub Vec<(String, Tree)>);
```

Same shape as `BoundEffects` but with consumption semantics. Walked by `walk_staged_effects` (in `walking/walk_effects/system.rs`), which iterates entries, evaluates any whose top-level gate matches the active trigger, and then calls `commands.remove_staged_effect` to consume the matched entry by `(source, tree)` tuple identity.

### Why entry-specific consumption

`walk_staged_effects` removes an entry by exact `(source, Tree)` tuple match, not by source name. This is load-bearing for nested gate arming during the same trigger event:

> *"if the inner staged tree is `Tree::When(A, Tree::When(A, ...))`, evaluating it will call `commands.stage_effect(entity, source, inner_when)` to arm its inner gate. The subsequent `commands.remove_staged_effect(entity, source, outer_tree)` removes the ORIGINAL outer staged entry by tuple match — the freshly-armed inner (different `Tree` value) is preserved. Deeper chains therefore peel layer by layer across ticks rather than wiping."*

Without exact-tuple identity, a same-source fresh stage queued during evaluation would be erased by a name-sweep removal. With exact-tuple identity, the original outer is consumed and the freshly-armed inner survives until the next matching trigger.

### Walker call order

Bridge systems must call `walk_staged_effects` **before** `walk_bound_effects` for the same `(entity, trigger, context)` tuple. Reason: a `When` in `BoundEffects` may arm a fresh inner gate by staging it under the same source. Walking bound first would then walk the freshly-armed inner against the *same* trigger event that armed it, breaking the documented "new trigger event required" semantics. Walking staged first ensures the inner is only ever evaluated by the *next* matching trigger.

This ordering is enforced inside each bridge system's body, not by scheduling.

## ArmedFiredParticipants

Owner-side bookkeeping for the Shape D armed-On disarm path. Populated by `commands.track_armed_fire` and drained by `evaluate_conditions` reversal.

```rust
#[derive(Component, Default, Debug)]
pub struct ArmedFiredParticipants(pub HashMap<String, Vec<Entity>>);

impl ArmedFiredParticipants {
    pub fn track(&mut self, armed_source: String, participant: Entity);
    pub fn drain(&mut self, armed_source: &str) -> Vec<Entity>;
}
```

When an armed `On(participant, Fire(reversible))` fires through `evaluate_on`, the walker queues a `TrackArmedFireCommand` on the **owner** (the entity carrying the `BoundEffects` containing the armed entry). The command appends `(armed_source → participant)` to the owner's `ArmedFiredParticipants`.

When the condition deactivates, `evaluate_conditions` drains the entry for `armed_source` and queues a `commands.reverse_effect(participant, reversible, armed_source)` for each tracked participant. The participant vec intentionally allows duplicates — N fires produce N reverses, matching `commands.reverse_effect`'s single-instance semantics.

## SpawnStampRegistry

Global registry of `Spawn`-rooted trees, keyed by `EntityKind`. Populated by chip dispatch; read by per-kind watcher systems.

```rust
#[derive(Resource, Default)]
pub struct SpawnStampRegistry {
    pub entries: Vec<(EntityKind, String, Tree)>,
}
```

A flat vec of `(entity_kind, name, tree)` triples. Watcher systems iterate the vec and call `commands.stamp_effect` for every entry whose `entity_kind` exactly matches the watched kind.

The watchers live in `effect_v3/storage/spawn_stamp_registry/watchers/`:

- `stamp_spawned_bolts` — `Query<Entity, Added<Bolt>>`
- `stamp_spawned_cells` — `Query<Entity, Added<Cell>>`
- `stamp_spawned_walls` — `Query<Entity, Added<Wall>>`
- `stamp_spawned_breakers` — `Query<Entity, Added<Breaker>>`

All four watchers are registered into `EffectV3Systems::Bridge` in `EffectV3Plugin::build`.

`EntityKind::Any` is intentionally not handled by the watchers — wildcarding belongs at the trigger-matching layer, not at spawn-time stamping. A chip authored as `Spawn(Any, ...)` would be silently ignored by all four watchers.

## EffectStack&lt;C&gt;

Generic stack component for stack-based passive effects (`SpeedBoostConfig`, `PiercingConfig`, `DamageBoostConfig`, etc.). Lives in `effect_v3/stacking/effect_stack/`.

`EffectStack<C>` holds a vec of `(source, config)` pairs for effects that stack additively or multiplicatively. Each `Fireable::fire` pushes onto the stack; `Reversible::reverse_all_by_source` retains-not-matching to remove every entry from a given source.

Recalculation systems (each effect has its own — `apply_speed_boosts`, `apply_damage_boosts`, etc.) read the stack and compute the effective value. This is the standard pattern for any passive that participates in arithmetic stacking — the stack is the source of truth, the recalculator is idempotent.

Singleton effects (`Pulse`, `Shield`, `SecondWind`, `GravityWell`, etc.) do **not** use `EffectStack` — they use a per-entity component or spawn a child entity, and `reverse_all_by_source` defaults to a single `reverse()` call.

## Optionality

Both `BoundEffects` and `StagedEffects` are inserted on-demand at first effect dispatch — `dispatch_chip_effects` calls `commands.entity(entity).insert_if_new(BoundEffects::default())` and the staged equivalent before stamping. They are not part of any spawn bundle. Querying systems use `Option<&BoundEffects>` only when interacting with entities that may or may not have ever received an effect; once present, the components are never removed (only their internal vecs are mutated).

`ArmedFiredParticipants` and `EffectStack<C>` are inserted lazily on first need — their respective producers check for the component and insert it if absent.
