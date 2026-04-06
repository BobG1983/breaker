# Research: Transfer Effects Flow — End-to-End Dispatch Pipeline

Bevy version: **0.18** (Cargo.toml workspace root, no explicit version field — read from breaker-game/Cargo.toml).

---

## 1. Where `Target::AllBolts` Gets Resolved

`Target::AllBolts` is resolved in two different sites, depending on the dispatch path.

### Path A — Chip dispatch (`dispatch_chip_effects`)

File: `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs`

In `dispatch_chip_effects`, only `Target::Breaker` gets direct dispatch. Every other target — including `AllBolts` — enters the **deferred path**:

```
if *target == Target::Breaker {
    // direct dispatch now
} else {
    // wrap and push to Breaker's BoundEffects
    let wrapped = EffectNode::When {
        trigger: Trigger::NodeStart,
        then: vec![EffectNode::On {
            target: *target,   // AllBolts preserved here
            permanent: true,
            then: then.clone(),
        }],
    };
    for breaker_entity in targets.breakers.iter() {
        commands.push_bound_effects(breaker_entity, vec![(chip_name.clone(), wrapped.clone())]);
    }
}
```

`AllBolts` is NOT resolved to entity IDs at chip dispatch time. It is wrapped verbatim in the tree and stored in the Breaker's `BoundEffects`.

Resolution happens **later**, inside `walk_staged_node` in the evaluate system when the `NodeStart` trigger fires:

- `evaluate_bound_effects` is called on the Breaker entity with `Trigger::NodeStart`
- The `When { trigger: NodeStart, then: [On { target: AllBolts, ... }] }` node matches
- The `On` child is pushed to `StagedEffects`
- `evaluate_staged_effects` then processes the `On` node from staged, calling `commands.queue(ResolveOnCommand { target: AllBolts, ... })`
- `ResolveOnCommand::apply` calls `resolve_all(Target::AllBolts, world)`, which runs `world.query_filtered::<Entity, With<Bolt>>()` to get every bolt entity at that moment

File: `breaker-game/src/effect/commands/ext.rs` — `ResolveOnCommand::apply` and `resolve_all`.

### Path B — Breaker definition effects (`dispatch_initial_effects`)

File: `breaker-game/src/effect/commands/ext.rs` — `DispatchInitialEffects::apply`

Same deferred pattern. `AllBolts`/`AllCells`/`AllWalls` are wrapped in `When(NodeStart, On(target, permanent: true, ...))` and pushed to the first primary breaker's `BoundEffects`. Resolution happens via the same `NodeStart` bridge path.

`Target::Bolt` (singular) dispatches to `PrimaryBolt` entities immediately. `Target::Cell` and `Target::Wall` are silently skipped (no entities exist at init time).

### Path C — Bolt definition effects (`dispatch_bolt_effects`)

File: `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs`

`AllBolts` is treated identically to `Bolt` — resolved immediately to all `With<Bolt>` entities at the time of `Added<BoltDefinitionRef>`. **No deferred path.** Non-Do children are pushed directly to `push_bound_effects` on all matching entities.

### Path D — Cell definition effects (`dispatch_cell_effects`)

File: `breaker-game/src/state/run/node/systems/dispatch_cell_effects/system.rs`

`AllBolts` resolves immediately to `bolt_query.iter().collect()`. Same for `AllCells` and `AllWalls`. No deferred path. Runs `OnEnter(NodeState::Loading)` after `NodeSystems::Spawn`.

### Summary: Where `AllBolts` is finally resolved to entity IDs

| Dispatch path | Resolution timing | Resolution mechanism |
|---|---|---|
| Chip effects | Deferred — at `NodeStart` trigger | `ResolveOnCommand` → `resolve_all` → `query_filtered::<Entity, With<Bolt>>` |
| Breaker definition effects | Deferred — at `NodeStart` trigger | Same |
| Bolt definition effects | Immediate — on bolt spawn | `bolt_query.iter().collect()` in `dispatch_bolt_effects` |
| Cell definition effects | Immediate — on cell spawn | `bolt_query.iter().collect()` in `dispatch_cell_effects` |

---

## 2. What `transfer_effects` Is

There is no function or system named `transfer_effects`. The plan doc uses this as a placeholder name. The actual mechanism is the `TransferCommand` struct.

### `TransferCommand`

File: `breaker-game/src/effect/commands/ext.rs`

```rust
pub(crate) struct TransferCommand {
    pub(crate) entity: Entity,
    pub(crate) chip_name: String,
    pub(crate) children: Vec<EffectNode>,
    pub(crate) permanent: bool,
    pub(crate) context: TriggerContext,
}
```

Called via `commands.transfer_effect(entity, chip_name, children, permanent, context)`.

**What it does (in `TransferCommand::apply`):**

1. Splits `children` into three buckets:
   - `Do(effect)` nodes → `do_effects` vec
   - `On { target, permanent, then }` nodes → `on_children` vec
   - Everything else (When, Once, Until, Reverse) → `other_children` vec
2. Ensures `BoundEffects` and `StagedEffects` exist on the entity
3. For each item in `other_children`:
   - If `permanent: true` → pushed to `BoundEffects`
   - If `permanent: false` → pushed to `StagedEffects`
4. For each `Do(effect)` → calls `effect.fire(entity, &chip_name, world)` immediately (same frame, synchronous)
5. For each `On { target, permanent, then }` → queues `ResolveOnCommand` to resolve the inner target

**Callers of `transfer_effect` / `TransferCommand`:**

- `dispatch_chip_effects` → `dispatch_children` → for non-Do, non-On nodes, calls `commands.transfer_effect(entity, chip_name, vec![node], true, TriggerContext::default())`
- `DispatchInitialEffects::apply` → `TransferCommand::apply` (for Breaker and Bolt targets at init time)
- `walk_staged_node` in the evaluate system does NOT call `transfer_effect` directly — it calls `commands.queue(ResolveOnCommand)` which internally chains to `TransferCommand`

### `PushBoundEffects`

A simpler command: just appends pre-formed `(String, EffectNode)` entries to `BoundEffects`, ensuring both effect components exist. Used by chip/bolt/cell dispatch paths when the entries are already formatted.

---

## 3. `dispatch_breaker_effects` / `dispatch_chip_effects`

### Breaker definition effects

There is no `dispatch_breaker_effects` system. Breaker-defined effects are dispatched inside the breaker builder's `.spawn()` terminal method:

File: `breaker-game/src/breaker/builder/core/terminal.rs`

```rust
pub fn spawn(self, commands: &mut Commands) -> Entity {
    let effects = self.optional.effects.clone();
    let entity = commands.spawn(self.build()).id();
    if let Some(effects) = effects.filter(|e| !e.is_empty()) {
        commands.dispatch_initial_effects(effects, None);
    }
    entity
}
```

All four terminal states (Rendered/Headless x Primary/Extra) follow this pattern. `dispatch_initial_effects` with `source_chip: None` means `chip_name` defaults to empty string `""`.

The breaker builder's `.spawn()` is called from `setup_run` (`OnEnter(NodeState::Loading)`), which is the run initialization system.

### Chip effects

`dispatch_chip_effects` system (file: `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs`) is registered in `ChipsPlugin`:

```rust
.add_systems(
    Update,
    dispatch_chip_effects.run_if(in_state(ChipSelectState::Selecting)),
)
```

It runs in `Update`, gated on `ChipSelectState::Selecting`. It reads `ChipSelected` messages, looks up each chip in `ChipCatalog`, records it in `ChipInventory`, then dispatches effects.

**Flow for chip effects:**
1. `ChipSelected` message arrives
2. `dispatch_chip_effects` reads it, gets `def.effects: Vec<RootEffect>`
3. Iterates over `def.effects` (the pre-desugaring list)
4. For `Target::Breaker`: calls `dispatch_children` which handles Do/On/other separately
5. For everything else: wraps in `When(NodeStart, On(target, permanent: true, then))` and `push_bound_effects` to all Breaker entities
6. The chip is recorded in `ChipInventory` (stack check happens first; if already at max stacks, dispatch is skipped)

---

## 4. Pre-Desugaring vs Post-Desugaring: Index Availability

### Pre-desugaring state

At the time `dispatch_chip_effects` runs, `def.effects` is a `Vec<RootEffect>` — the raw deserialized list. The index of each `RootEffect` in that list is available via `.iter().enumerate()` on `def.effects`. The iteration currently uses a `for root_effect in &effects` loop at line 62 of `system.rs`, discarding the index.

**The pre-desugaring index is available at dispatch time but is not captured.** The `chip_name` from `def.name` is captured, but no `(chip_name, effect_index)` pair is formed.

### Post-desugaring state

Once the `AllBolts` path executes, the original `then: Vec<EffectNode>` children are moved wholesale into an `EffectNode::On { target: AllBolts, permanent: true, then }` node, which is nested inside `EffectNode::When { trigger: NodeStart, then: [...] }`. The outer `When` node is stored as a single `(chip_name, node)` entry in `BoundEffects`.

At the point of `ResolveOnCommand::apply` (when NodeStart fires and the On node is processed from StagedEffects), the `chip_name` is preserved but the original effect index is gone. There is no way to recover which position in `def.effects` this `On` node came from.

### Implication for source_id design

The plan doc's question: "Need to verify pre-desugaring indexes are accessible in `transfer_effects`."

Answer: The pre-desugaring index **is** accessible at dispatch time (in `dispatch_chip_effects`, line 62 iterates `&effects` — switching to `.iter().enumerate()` would give the index). However, by the time effects reach `TransferCommand` or `ResolveOnCommand`, the index has been lost. If a source_id is needed at the point of stamping (inside `ResolveOnCommand` or `TransferCommand`), it must be injected earlier — either embedded in the wrapped node structure at dispatch time, or derived from a stable hash of the effect content.

The chip_name is always preserved through the entire chain: `dispatch_chip_effects` → `push_bound_effects` → `BoundEffects` → `evaluate_bound_effects` → `evaluate_staged_effects` → `ResolveOnCommand` → `TransferCommand`. So `(chip_name, effect_index)` is a viable source_id if the index is captured at dispatch time and carried through. The most practical insertion point would be in `dispatch_chip_effects`'s main loop, before building the `wrapped` node.

---

## 5. `BoundEffects` Structure

File: `breaker-game/src/effect/core/types/definitions/enums.rs`

```rust
/// Permanent effect trees on an entity. Never consumed by trigger evaluation.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct BoundEffects(pub Vec<(String, EffectNode)>);

/// Working set of partially-resolved chains. Consumed when matched.
/// Each entry is `(chip_name, node)`.
#[derive(Component, Debug, Default, Clone)]
pub struct StagedEffects(pub Vec<(String, EffectNode)>);
```

**Key facts:**
- `BoundEffects` is a `Vec<(String, EffectNode)>` — ordered, allows duplicates, no per-entry dedup
- The `String` field is `chip_name` — the name of the chip that sourced the effect (empty `""` for bolt/cell/breaker-definition-sourced effects)
- `EffectNode` is the full tree node — a `When`, `Once`, `Until`, `On`, `Do`, or `Reverse` variant
- Both components are always inserted as a pair (`ensure_effect_components` inserts both together when either is missing)
- `BoundEffects` entries are **never consumed** by trigger evaluation — they are permanent for the entity's lifetime
- `StagedEffects` entries **are consumed** when matched — they are the working set for partially-resolved chains

**How individual effects are stored:**

After `AllBolts` resolves at `NodeStart`, each bolt entity gets one `BoundEffects` entry per dispatched `On` node, shaped like:

```
("chip_name", When { trigger: SomeTrigger, then: [Do(SomeEffect)] })
```

The nesting depth depends on the original RON definition. A chip with:
```
On(target: AllBolts, then: [When(trigger: Bumped, then: [Do(SpeedBoost(multiplier: 1.5))])])
```
results in each bolt having:
```
BoundEffects([("ChipName", When { trigger: Bumped, then: [Do(SpeedBoost { multiplier: 1.5 })] })])
```

A bare `Do` at the AllBolts level:
```
On(target: AllBolts, then: [Do(DamageBoost(1.3))])
```
results in the entire Do being executed when the On node is processed at NodeStart, so the bolt gets no BoundEffects entry — the fire call happens immediately in `TransferCommand::apply`.

---

## 6. System Chain Summary

### Chip → AllBolts → Entity BoundEffects

```
ChipSelected (message)
  │
  ▼
dispatch_chip_effects (Update, ChipSelectState::Selecting)
  │  reads: ChipSelected, ChipCatalog
  │  writes: ChipInventory, Commands
  │
  │  For Target::AllBolts (and non-Breaker targets):
  ▼
push_bound_effects (command, deferred to next frame)
  │  writes: Breaker BoundEffects ← When(NodeStart, On(AllBolts, perm:true, [children]))
  │
  │  [chip select completes, node starts]
  ▼
bridge_node_start (OnEnter(NodeState::Playing))
  │  reads: BoundEffects, StagedEffects on all entities
  │  calls: evaluate_bound_effects → evaluate_staged_effects
  │
  │  For Breaker entity: When(NodeStart) matches
  │    → On(AllBolts) pushed to Breaker's StagedEffects
  │    → evaluate_staged_effects: On node consumed, queues ResolveOnCommand
  ▼
ResolveOnCommand (command, applied same frame via Commands::queue)
  │  resolves AllBolts → all With<Bolt> entities (current frame snapshot)
  │  for each bolt: queues TransferCommand
  ▼
TransferCommand (command, applied same frame)
  │  splits children into Do/On/other
  │  fires Do children immediately
  │  pushes other children to BoundEffects (permanent: true)
  ▼
Bolt entity: BoundEffects contains the trigger chains
```

### Breaker definition → AllBolts → Entity BoundEffects

```
Breaker::builder().spawn(&mut commands) (setup_run, OnEnter(NodeState::Loading))
  │  calls: commands.dispatch_initial_effects(effects, None)
  ▼
DispatchInitialEffects (command, deferred)
  │  For AllBolts: wraps in When(NodeStart, On(AllBolts, perm:true, [children]))
  │  push_bound_to → first PrimaryBreaker's BoundEffects
  │
  [continues as above from bridge_node_start]
```

---

## 7. Edge Cases Observed in Code

**1. The pre-existing bolt gap (the stated problem)**

When `AllBolts` desugars at NodeStart, it queries `With<Bolt>` at that exact moment. Bolts spawned later in the node (by `SpawnBolts`, `MirrorProtocol`, etc. with `inherit: false`) are not in the query result and receive no effects.

**2. Dedup — currently absent**

`BoundEffects` is a plain Vec with no dedup. If a second Breaker entity spawns and `dispatch_chip_effects` runs again for the same chip (or the `When(NodeStart)` fires again), effects are double-appended. The plan doc identifies this as a real bug.

**3. `dispatch_chip_effects` and `dispatch_bolt_effects` diverge on AllBolts**

`dispatch_chip_effects` defers AllBolts via the `NodeStart` wrapper. `dispatch_bolt_effects` resolves AllBolts immediately (same as Bolt). This is intentional — bolt definitions are fixed at spawn time, chips are selected at chip-select time when no node entities exist.

**4. `chip_name` is empty string for non-chip sources**

Breaker definitions pass `source_chip: None` → empty string. Bolt and cell dispatch paths explicitly use `String::new()`. This means the `chip_name` field in `BoundEffects` is `""` for all non-chip effects, which limits the discriminating power of chip_name for source_id purposes in those paths.

**5. Commands are deferred; queries are point-in-time**

`PushBoundEffects` and `TransferCommand` are custom `Command` impls that run via `world.get_entity_mut(...)` — they execute within the same command application batch. However, `DispatchInitialEffects` and `ResolveOnCommand` are also custom commands that themselves call `.apply(world)` synchronously on nested commands. This means the entire chain resolves within a single command flush, not across frames.

---

## 8. Insertion Points for AllBoltsEffects Store

Based on the code reading, there are two viable insertion points for the proposed `AllBoltsEffects` resource:

### Option A: In `dispatch_chip_effects` (early, pre-desugaring)

Location: `system.rs` line 62–91, inside the `else` branch for non-Breaker targets.

At this point:
- `root_effect` is the raw `RootEffect::On { target: AllBolts, then }` from `def.effects`
- The iteration index is available via `.iter().enumerate()` (currently not captured)
- `chip_name` is available
- A `(chip_name, effect_index)` source_id is constructible here

The resource write and existing-entity stamp would happen here before the deferred wrapped node is pushed to the Breaker.

### Option B: In `ResolveOnCommand::apply` (late, at resolution time)

Location: `ext.rs` `ResolveOnCommand::apply`, for the `AllBolts | AllCells | AllWalls` arm.

At this point:
- `self.chip_name` is available
- `self.children` is the original `then` vec
- The effect index is NOT available (was lost at dispatch time)
- The source_id would need to be a hash of children content or would need to have been injected into the `ResolveOnCommand` struct at dispatch time

Option A is cleaner for index-based source_ids. Option B works if source_id uses content-hashing or if a source_id field is added to `ResolveOnCommand`.

---

## Key Files

- `breaker-game/src/effect/core/types/definitions/enums.rs` — `BoundEffects`, `StagedEffects`, `Target`, `EffectNode`, `RootEffect` definitions
- `breaker-game/src/effect/commands/ext.rs` — All command types: `TransferCommand`, `ResolveOnCommand`, `PushBoundEffects`, `DispatchInitialEffects` and the `resolve_all` / `resolve_default` functions
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — `dispatch_chip_effects` and `dispatch_children`; the deferred-AllBolts wrapping logic
- `breaker-game/src/bolt/systems/dispatch_bolt_effects/system.rs` — `dispatch_bolt_effects`; immediate AllBolts resolution
- `breaker-game/src/state/run/node/systems/dispatch_cell_effects/system.rs` — `dispatch_cell_effects`; immediate AllBolts resolution
- `breaker-game/src/breaker/builder/core/terminal.rs` — Breaker builder `.spawn()` calls `dispatch_initial_effects`
- `breaker-game/src/effect/triggers/node_start.rs` — `bridge_node_start`; `OnEnter(NodeState::Playing)` trigger that resolves deferred AllBolts nodes
- `breaker-game/src/effect/triggers/evaluate/system.rs` — `evaluate_bound_effects`, `evaluate_staged_effects`, `walk_bound_node`, `walk_staged_node`
- `breaker-game/src/chips/plugin.rs` — `dispatch_chip_effects` schedule: `Update`, `run_if(in_state(ChipSelectState::Selecting))`
- `breaker-game/src/bolt/plugin.rs` — `dispatch_bolt_effects` schedule: `FixedUpdate`, `before(EffectSystems::Bridge)`, `run_if(in_state(NodeState::Playing))`
- `breaker-game/src/cells/plugin.rs` — `dispatch_cell_effects` schedule: `OnEnter(NodeState::Loading)`, `after(NodeSystems::Spawn)`
