# Storage & Dispatch

How BoundEffects, StagedEffects, and OnSpawnEffectRegistry are structured, and how dispatch walks them.

## BoundEffects (Component)

Permanent effect trees. Populated by Route at load time and Stamp at runtime. Never consumed — When entries re-arm after firing.

```rust
#[derive(Component, Default)]
struct BoundEffects {
    /// Trigger-keyed entries. When a trigger fires, linear scan for matching
    /// trigger, walk matching trees, execute or arm into StagedEffects.
    /// Flat Vec — counts are small (no chip has hundreds of triggers) and
    /// Trigger contains f32 variants that can't impl Hash/Eq.
    triggers: Vec<BoundEntry<Trigger>>,

    /// Condition-keyed entries. The condition monitor system reads these
    /// directly — not event-driven, checked on state change.
    conditions: Vec<BoundEntry<Condition>>,

    /// Reverse index for removal. Maps SourceId to entry indices.
    /// Used by chip unequip to find and remove all entries from a source.
    /// HashMap is fine here — SourceId is String, impls Hash+Eq.
    sources: HashMap<SourceId, Vec<BoundKey>>,
}

struct BoundEntry<K> {
    key: K,
    source: SourceId,
    tree: ValidTree,
}

enum BoundKey {
    Trigger(usize),    // index into triggers Vec
    Condition(usize),  // index into conditions Vec
}

type SourceId = String; // chip name, breaker name, etc.
```

> **Why flat Vec, not HashMap?** `Trigger` contains `f32` variants (`TimeExpires`, `NodeTimerThresholdOccurred`) which don't implement `Hash` or `Eq`. Using `HashMap<Trigger, ...>` would require wrapping all f32s in `OrderedFloat<f32>`, polluting the entire type tree with a dependency. Flat Vec with linear scan on `trigger == key` avoids this entirely. Performance is equivalent — chip effect counts are in single digits.

### What goes in BoundEffects

| Origin | When | Example |
|---|---|---|
| Route at chip equip | Equip command | `Route(Bolt, When(Impacted(Cell), Fire(Shockwave)))` |
| Route at breaker spawn | Breaker spawn | `Route(Breaker, When(BoltLostOccurred, Fire(LoseLife)))` |
| Stamp terminal at runtime | Trigger fires | `On(ImpactTarget::Impactee, Stamp(When(Died, Fire(Explode))))` |
| Until reversal | Until fires | `Once(Died, Reverse(SpeedBoost(1.5)))` — one-shot reversal |

### What does NOT go in BoundEffects

- Transfer payloads → go to StagedEffects (one-shot)
- Armed inner trees from nested When → go to StagedEffects
- Fire terminals → execute immediately, not stored

## StagedEffects (Component)

Armed inner trees waiting for a trigger match. Populated by Transfer at runtime and by nested When arming. Consumed when triggered.

```rust
#[derive(Component, Default)]
struct StagedEffects {
    /// Armed entries waiting for trigger match. Linear scan on trigger fire,
    /// execute all matching entries, remove them (consumed).
    /// Flat Vec — same rationale as BoundEffects (Trigger contains f32).
    entries: Vec<StagedEntry>,
}

struct StagedEntry {
    trigger: Trigger,
    source: SourceId,
    tree: ValidTree,
}
```

### What goes in StagedEffects

| Origin | When | Consumed when |
|---|---|---|
| Nested When arming | Outer trigger fires | Inner trigger fires |
| Transfer terminal | Trigger fires | Transferred tree's trigger fires |

### Examples

**Nested When arming:**
```
BoundEffects: PerfectBumped -> [When(Impacted(Cell), Fire(Shockwave))]

PerfectBumped fires:
  → arm inner into StagedEffects:
    StagedEffects: Impacted(Cell) -> [Fire(Shockwave)]

Impacted(Cell) fires:
  → execute Fire(Shockwave)
  → remove entry from StagedEffects (consumed)

Next PerfectBumped:
  → re-arms from BoundEffects again
```

**Transfer:**
```
BoundEffects (bolt): Impacted(Cell) -> [On(ImpactTarget::Impactee, Transfer(When(Died, Fire(Explode))))]

Bolt impacts cell:
  → Transfer fires: insert into cell's StagedEffects:
    StagedEffects (cell): Died -> [Fire(Explode)]

Cell dies:
  → execute Fire(Explode)
  → remove entry from StagedEffects (consumed)
  → bolt must impact again to re-transfer
```

## OnSpawnEffectRegistry (Resource)

Global registry of Spawned listeners. Populated by EveryBolt desugaring and explicit Spawned entries at equip time. Read by bridge systems on Added<T>.

```rust
#[derive(Resource, Default)]
struct OnSpawnEffectRegistry {
    /// EntityType -> trees to stamp/transfer onto new entities of that type.
    entries: HashMap<EntityType, Vec<SpawnedEntry>>,
}

struct SpawnedEntry {
    source: SourceId,
    tree: ValidTree,
}
```

### Bridge systems

4 standard systems in PostFixedUpdate (not Bevy Observers):

```rust
fn bridge_bolt_added(
    new_bolts: Query<Entity, Added<Bolt>>,
    registry: Res<OnSpawnEffectRegistry>,
    mut bound_query: Query<&mut BoundEffects>,
) {
    for new_bolt in &new_bolts {
        if let Some(entries) = registry.entries.get(&EntityType::Bolt) {
            for entry in entries {
                // Insert tree into new bolt's BoundEffects
                insert_tree(&mut bound_query, new_bolt, &entry.source, &entry.tree);
            }
        }
    }
}

// bridge_cell_added, bridge_wall_added, bridge_breaker_added — same pattern
```

## Dispatch

### Trigger dispatch

When a trigger fires (from a game message like BoltHitCell, BumpGraded, etc.), the dispatch system:

1. Build TriggerContext with participant entities
2. Determine locality: local triggers fire on participant entities only, global triggers fire on ALL entities with BoundEffects/StagedEffects
3. For each target entity:

**CRITICAL: Walk StagedEffects FIRST, then BoundEffects.** This prevents a single trigger from both arming and consuming a nested When in the same dispatch. Example: `When(PerfectBumped, When(PerfectBumped, Fire(SpeedBoost)))` — first PerfectBumped should arm the inner When into StagedEffects; the second PerfectBumped should consume it. If BoundEffects walked first, the arm and consume would happen in one dispatch call.

**Step A — Walk StagedEffects (consume):**
- Look up trigger key in `StagedEffects.entries`
- Execute all matching entries
- Remove all matched entries (consumed)
- Execution rules for inner trees:
  - `Fire(effect)` → execute effect on This
  - `Sequence([...])` → execute children in order
  - `When(trigger, inner)` → arm `inner` into StagedEffects under new trigger key (re-nesting)
  - `On(participant, terminal)` → resolve participant from TriggerContext:
    - `Fire(effect)` → execute on participant entity
    - `Stamp(tree)` → insert into participant's BoundEffects (permanent)
    - `Transfer(tree)` → insert into participant's StagedEffects (one-shot)

**Step B — Walk BoundEffects (arm or execute):**
- Look up trigger key in `BoundEffects.triggers`
- For each matched entry:
  - `When(trigger, inner)` → arm `inner` into StagedEffects under inner trigger key. **Keep entry** (re-arms next time).
  - `Once(trigger, inner)` → arm `inner` into StagedEffects. **Remove entry** (self-removes).
  - `Fire(effect)` → execute on This (bare Fire under trigger key).
  - `Sequence([...])` → execute children in order.
  - `On(participant, terminal)` → same resolution as StagedEffects above.

### Trigger locality — bridge systems decide

No `locality()` method on Trigger. The bridge system that fires the trigger already knows its scope:
- Local bridges (bump, impact, death) pass participant entities to `dispatch_trigger`
- Global bridges (BumpOccurred, DeathOccurred, etc.) query all entities with BoundEffects/StagedEffects
- `dispatch_trigger` takes a `&[Entity]` target list — the caller decides who

### Recursion depth limit

Effects can trigger spawns, which trigger bridge systems, which stamp effects, which could fire more spawns. A depth counter on TriggerContext prevents infinite loops:

```rust
struct TriggerContext {
    // ... participant fields ...
    depth: u32,
}

const MAX_DISPATCH_DEPTH: u32 = 10;

fn dispatch_trigger(trigger: Trigger, context: TriggerContext, world: &mut World) {
    if context.depth >= MAX_DISPATCH_DEPTH {
        warn!("Effect dispatch depth limit reached for {:?}", trigger);
        return;
    }
    // ... normal dispatch with context.depth + 1 for any sub-dispatches
}
```

### Stale participant references

If `On(BumpTarget::Bolt, Fire(SpeedBoost))` resolves a participant entity that has been despawned, log a debug warning and skip the fire. Helps catch bugs in development, normal gameplay occurrence in production.

### Condition monitor

One system per condition, each watching for its specific state change. During is first-class — not desugared into triggers.

```rust
// NodeActive: watches NodeState transitions
fn monitor_node_active(
    node_state: Res<State<NodeState>>,
    mut query: Query<(Entity, &mut BoundEffects, &mut StagedEffects)>,
) {
    // NodeActive = Playing or Paused (spans both)
    // Start: enter Playing from non-Playing non-Paused
    // End: node teardown
    for (entity, mut bound, mut staged) in &mut query {
        if let Some(entries) = bound.conditions.get(&Condition::NodeActive) {
            // activate_during / deactivate_during on transition
        }
    }
}

// ShieldActive: watches ShieldWall entity existence
fn monitor_shield_active(
    added: Query<Entity, Added<ShieldWall>>,
    removed: RemovedComponents<ShieldWall>,
    existing: Query<Entity, With<ShieldWall>>,
    mut query: Query<(Entity, &mut BoundEffects, &mut StagedEffects)>,
) {
    // Start: Added<ShieldWall> detected AND no shield was active before
    // End: ShieldWall removed AND no ShieldWall entities remain
    // activate/deactivate During(ShieldActive, ...) entries
}

// ComboActive: watches consecutive perfect bump counter
fn monitor_combo_active(
    bump_events: MessageReader<BumpGraded>,
    tracker: Res<HighlightTracker>,  // has consecutive_perfect_bumps
    mut query: Query<(Entity, &mut BoundEffects, &mut StagedEffects)>,
) {
    // For each ComboActive(n) condition in BoundEffects:
    // Start: consecutive_perfect_bumps crosses n upward
    // End: consecutive_perfect_bumps resets to 0 (non-perfect bump)
    // Must track per-n threshold state to detect crossings
}
```

During stays in `BoundEffects.conditions` permanently — condition cycling is handled by the monitors calling activate/deactivate on each transition. Each condition can cycle independently.

### During + nested When lifecycle

The most complex dispatch interaction. A During entry's inner tree can be:

**Direct Fire** (`During(NodeActive, Fire(SpeedBoost))`):
- Activate: call `fire_effect`, inline reversal on deactivate via `reverse_effect`
- Deactivate: call `reverse_effect` for each effect fired

**Direct Sequence** (`During(NodeActive, Sequence([Fire(SpeedBoost), Fire(DamageBoost)]))`):
- Activate: fire all children in order
- Deactivate: reverse all children (reverse order)

**Nested When** (`During(NodeActive, When(PerfectBumped, Fire(Explode)))`):
- Activate: register the inner When into `BoundEffects.triggers` under `PerfectBumped`, tagged with a scope source (e.g., `"Aftershock:During(NodeActive)"`)
- While active: normal trigger dispatch handles PerfectBumped → Fire(Explode)
- Deactivate: remove the registered When from `BoundEffects.triggers` by scope source. Also clean up any StagedEffects entries armed from it (same entity, same scope source). This prevents stale armed entries from firing after the scope ends.

The scope source is derived from the chip SourceId + the condition: `format!("{source}:During({condition:?})")`. This lets remove_source target just the During-registered entries without affecting other entries from the same chip.

### Until reversal tracking

Until desugars at stamp time:
1. Fire inner effects immediately
2. Insert `Once(until_trigger, Reverse(effect))` into BoundEffects

Normal dispatch handles the reversal — when the until trigger fires, Once matches, Reverse executes, Once self-removes. Clean.

For Until + nested When: same pattern as During — register inner When at stamp time, insert Once(trigger, unregister) for cleanup. But Until is simpler because it only fires once.

## Command Extensions

The existing `EffectCommandsExt` is rewritten in `new_effect/`:

| Command | Purpose | Destination |
|---|---|---|
| `fire_effect(entity, effect, context)` | Execute an effect on an entity | Immediate |
| `reverse_effect(entity, effect)` | Reverse a previously fired effect | Immediate |
| `stamp_effect(entity, source, tree)` | Add tree to entity's BoundEffects | BoundEffects |
| `transfer_effect(entity, source, tree)` | Add tree to entity's StagedEffects | StagedEffects |
| `remove_source(entity, source)` | Remove all entries from a source | BoundEffects + StagedEffects + OnSpawnEffectRegistry |

## Shared Tree Walker

There is no centralized dispatch function. Each trigger has its own Bevy bridge system that knows its participants and scope. What's shared is the **tree-walking helper** that every bridge system calls.

```rust
/// Walk an entity's effects for a trigger match.
/// Called by bridge systems — they decide which entities to walk.
///
/// CRITICAL: walks StagedEffects FIRST, then BoundEffects.
/// Prevents a single trigger from both arming and consuming
/// a nested When in the same dispatch call.
fn walk_effects(
    trigger: &Trigger,
    context: &TriggerContext,
    entity: Entity,
    bound: &BoundEffects,      // immutable — read only
    staged: &mut StagedEffects, // mutable for drain_filter on armed entries
    commands: &mut Commands,    // defer fire/stamp/transfer via EffectCommandsExt
) {
    if context.depth >= MAX_DISPATCH_DEPTH {
        warn!("Dispatch depth limit reached for {:?}", trigger);
        return;
    }

    // Step A: consume armed entries from StagedEffects (linear scan, drain matching)
    let matched: Vec<_> = staged.entries
        .extract_if(|e| e.trigger == *trigger)
        .collect();
    for entry in matched {
        execute_tree(&entry.tree, entity, context, bound, staged, commands);
    }

    // Step B: arm/execute from BoundEffects (linear scan, When re-arms, Once self-removes)
    let matching: Vec<_> = bound.triggers.iter().enumerate()
        .filter(|(_, e)| e.key == *trigger)
        .map(|(i, e)| (i, e.clone()))
        .collect();
    {
        let mut to_remove = Vec::new();
        for (i, entry) in &matching {
            match &entry.tree {
                ValidTree::When(_, inner) => {
                    // Arm inner into StagedEffects — entry stays (re-arms)
                    arm_into_staged(inner, &entry.source, staged);
                }
                ValidTree::Once(_, inner) => {
                    // Arm inner into StagedEffects — mark for removal
                    arm_into_staged(inner, &entry.source, staged);
                    to_remove.push(i);
                }
                other => {
                    // Bare Fire, Sequence, On — execute directly
                    execute_tree(other, entity, context, bound, staged, commands);
                }
            }
        }
        // Remove Once entries (reverse order to preserve indices)
        // ...
    }
}
```

## Command Extension Dispatch Model

walk_effects defers all effect execution and cross-entity mutation through `EffectCommandsExt` on Bevy `Commands`. Bridge systems are regular Bevy systems — they take `Query` + `Commands` parameters, no exclusive world access needed.

Same-entity StagedEffects mutations (drain matching entries) happen inline during the walk. Cross-entity operations (Stamp/Transfer onto another entity) and effect execution (Fire/Reverse) are deferred via command extensions. Bevy applies commands at schedule flush points, at which point the commands have exclusive world access.

See [command-extensions.md](command-extensions.md) for the full behavioral spec of each command extension.

This solves three problems:
- Bridge systems are regular Bevy systems (not exclusive)
- No aliased mutable borrows (Stamp/Transfer target other entities via deferred commands)
- Once removal collected during walk, applied after walk completes (same entity)

### Bridge system example

```rust
/// Bridge for PerfectBumped (local trigger — fires on both bolt and breaker)
fn bridge_perfect_bumped(
    mut bump_events: MessageReader<BumpGraded>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in bump_events.read() {
        if event.grade != BumpGrade::Perfect { continue; }

        let context = TriggerContext::Bump(BumpContext {
            bolt: event.bolt,
            breaker: event.breaker,
            depth: 0,
        });

        // Walk both participant entities
        for entity in [event.bolt, event.breaker] {
            if let Ok((entity, bound, mut staged)) = query.get_mut(entity) {
                walk_effects(&Trigger::PerfectBumped, &context, entity,
                    &bound, &mut staged, &mut commands);
            }
        }
    }
}

/// Bridge for PerfectBumpOccurred (global trigger — fires on ALL entities)
fn bridge_perfect_bump_occurred(
    mut bump_events: MessageReader<BumpGraded>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for event in bump_events.read() {
        if event.grade != BumpGrade::Perfect { continue; }

        let context = TriggerContext::Bump(BumpContext {
            bolt: event.bolt,
            breaker: event.breaker,
            depth: 0,
        });

        // Walk ALL entities with effects
        for (entity, bound, mut staged) in &mut query {
            walk_effects(&Trigger::PerfectBumpOccurred, &context, entity,
                &bound, &mut staged, &mut commands);
        }
    }
}
```

### NodeTimerThreshold bridge

```rust
fn bridge_node_timer_threshold(
    timer: Res<NodeTimer>,
    mut prev: Local<f32>,
    query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    let ratio = timer.ratio();
    for (entity, bound, mut staged) in &query {
        for entry in &bound.triggers {
            if let Trigger::NodeTimerThresholdOccurred(x) = entry.key {
                if *prev < x && x <= ratio {
                    walk_effects(&Trigger::NodeTimerThresholdOccurred(x), 
                        &TriggerContext::None { depth: 0 }, entity,
                        &bound, &mut staged, &mut commands);
                }
            }
        }
    }
    *prev = ratio;
}
```
Tracks previous ratio in `Local<f32>`. Scans entities' BoundEffects for threshold entries where `prev < threshold <= current_ratio`. Fires for each crossed threshold.
