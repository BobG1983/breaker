# Behavior Trace: Powder Keg Chip Effect

## Executive Summary

The Powder Keg chip has a **critical bug** in step 4 of its intended behavior. When the bolt hits a cell and `Impacted(Cell)` fires, the nested `On(target: Cell, ...)` node is processed by `ResolveOnCommand` which resolves `Target::Cell` to **all living cells** in the world, not to the specific cell that was hit. As a result, `When(Died, [Do(Explode)])` is pushed to every cell's `StagedEffects`. Whichever cell dies first will explode — regardless of whether it was the cell that was hit.

---

## Trigger

Player selects the "Powder Keg" chip during a node. The chip RON (`breaker-game/assets/chips/standard/powder_keg.chip.ron`) defines:

```ron
On(target: Bolt, then: [
    When(trigger: Impacted(Cell), then: [
        On(target: Cell, then: [
            When(trigger: Died, then: [
                Do(Explode(range: 48.0, damage_mult: 1.0)),
            ]),
        ]),
    ]),
])
```

---

## Step 1 — RON Parsing and RootEffect Deserialization

**File:** `breaker-game/src/effect/core/types/definitions/enums.rs`

The RON `On(target: Bolt, then: [...])` deserializes to `RootEffect::On { target: Target::Bolt, then: Vec<EffectNode> }` (lines 131–141).

`RootEffect` is a one-variant enum (`On { target, then }`) — it enforces that every effect chain names its target entity. The `From<RootEffect> for EffectNode` impl (lines 142–151) converts it to `EffectNode::On { target, permanent: false, then }`.

The inner `On(target: Cell, ...)` in the RON is parsed as `EffectNode::On` directly (not `RootEffect`), because the nested `On` is inside a `When` which is itself inside the `RootEffect::On.then` vec. The `#[serde(default)] permanent: bool` field defaults to `false` (line 174).

**Result after deserialization:**
```
RootEffect::On {
    target: Target::Bolt,
    then: [
        EffectNode::When {
            trigger: Trigger::Impacted(ImpactTarget::Cell),
            then: [
                EffectNode::On {
                    target: Target::Cell,
                    permanent: false,   // serde default
                    then: [
                        EffectNode::When {
                            trigger: Trigger::Died,
                            then: [
                                EffectNode::Do(EffectKind::Explode { range: 48.0, damage_mult: 1.0 })
                            ]
                        }
                    ]
                }
            ]
        }
    ]
}
```

`ChipTemplate` (`breaker-game/src/chips/definition/types.rs` lines 56–74) carries `effects: Vec<RootEffect>`. `expand_chip_template` (lines 100–133) converts the legendary `RaritySlot` into a `ChipDefinition` with `effects` cloned verbatim.

---

## Step 2 — Effect Binding: dispatch_chip_effects

**File:** `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs`

`dispatch_chip_effects` (lines 32–91) runs in the `Update` schedule, reading `ChipSelected` messages.

The root effect's target is `Target::Bolt` — **not** `Target::Breaker` — so it enters the **deferred dispatch** path (lines 73–89). The system wraps the entire effect tree in a `NodeStart` wrapper and pushes it to the Breaker's `BoundEffects`:

```rust
EffectNode::When {
    trigger: Trigger::NodeStart,
    then: vec![EffectNode::On {
        target: Target::Bolt,
        permanent: true,   // hardcoded here
        then: [
            // original children...
            EffectNode::When {
                trigger: Trigger::Impacted(ImpactTarget::Cell),
                then: [
                    EffectNode::On {
                        target: Target::Cell,
                        permanent: false,
                        then: [When(Died, [Do(Explode)])]
                    }
                ]
            }
        ]
    }]
}
```

This node is pushed to the Breaker's `BoundEffects` via `commands.push_bound_effects(breaker_entity, ...)`.

**Key detail:** the Bolt entity does not yet exist at chip-select time (chips are selected between nodes). The `permanent: true` on the wrapping `On(Bolt)` ensures the `When(Impacted(Cell), ...)` ends up in the bolt's `BoundEffects` (permanent), not `StagedEffects`, when the node starts.

---

## Step 3 — NodeStart fires: effect installs on Bolt

**File:** `breaker-game/src/effect/triggers/node_start.rs`

`bridge_node_start` runs on `OnEnter(PlayingState::Active)` (line 33). It iterates all entities with `BoundEffects` — including the Breaker — and calls `evaluate_bound_effects(&Trigger::NodeStart, ...)`.

**In the evaluate system** (`breaker-game/src/effect/triggers/evaluate/system.rs`, lines 49–71):

`walk_bound_node` matches `When(NodeStart, [On(Bolt, permanent: true, [...])])`. The child `On(Bolt, ...)` is not a `Do`, so it is pushed to the Breaker's `StagedEffects`.

Then `evaluate_staged_effects` runs with `NodeStart` (line 27). `walk_staged_node` (lines 74–153) encounters the `EffectNode::On` arm (lines 138–152): the `On` node is **always consumed** regardless of the active trigger, and queues `ResolveOnCommand { target: Target::Bolt, permanent: true, children: [When(Impacted(Cell), [...])], ... }`.

`ResolveOnCommand.apply()` (`breaker-game/src/effect/commands/ext.rs`, lines 178–191) calls `resolve_target_from_world(Target::Bolt, world)` which queries all `With<Bolt>` entities (line 202–204). For each bolt, `TransferCommand` (lines 125–161) is applied:
- `permanent: true` → non-`Do` children go to `BoundEffects`
- The child `When(Impacted(Cell), [...])` is pushed to the bolt's `BoundEffects`

`ensure_effect_components` (lines 91–98) inserts `BoundEffects` and `StagedEffects` on the bolt if absent. Bolts spawned via `.build()` without `.with_effects()` do not have these components initially (see `breaker-game/src/bolt/builder/core.rs` lines 399–413).

**End state after NodeStart:** Every bolt entity has `When(Impacted(Cell), [On(Cell, permanent: false, [When(Died, [Do(Explode)])])])` in its `BoundEffects`.

---

## Step 4 — Impacted(Cell) trigger fires

**File:** `breaker-game/src/effect/triggers/impacted/system.rs`

`bridge_impacted_bolt_cell` (lines 26–60) reads `BoltImpactCell` messages in `FixedUpdate`, after `BoltSystems::CellCollision`, in `EffectSystems::Bridge`.

`BoltImpactCell` (`breaker-game/src/bolt/messages.rs`) carries both `bolt: Entity` and `cell: Entity`.

The bridge fires `Trigger::Impacted(ImpactTarget::Cell)` on `msg.bolt` (lines 29–40):
1. `evaluate_bound_effects(&Impacted(Cell), bolt_entity, ...)` — `walk_bound_node` matches `When(Impacted(Cell), [On(Cell, ...)])`. The child `On(Cell, ...)` is not a `Do`, so it is pushed to the **bolt's** `StagedEffects`.
2. `evaluate_staged_effects(&Impacted(Cell), bolt_entity, ...)` — `walk_staged_node` encounters `EffectNode::On { target: Target::Cell, permanent: false, then: [When(Died, [Do(Explode)])] }`.

The `EffectNode::On` arm of `walk_staged_node` (lines 138–152) always returns `true` (consumed) and queues:
```rust
ResolveOnCommand {
    target: Target::Cell,
    chip_name: "Powder Keg",
    children: [When(Died, [Do(Explode { range: 48.0, damage_mult: 1.0 })])],
    permanent: false,
}
```

**The bug is here.** The `msg.cell` (the specific cell that was hit) is **not passed** to `ResolveOnCommand`. The target is `Target::Cell`, not a specific entity.

`ResolveOnCommand.apply()` calls `resolve_target_from_world(Target::Cell, world)` (lines 196–215):
```rust
Target::Cell | Target::AllCells => {
    let mut query = world.query_filtered::<Entity, With<Cell>>();
    query.iter(world).collect()
}
```

This returns **all living Cell entities** in the world. `TransferCommand` is applied to each one with `permanent: false`, pushing `When(Died, [Do(Explode)])` into each cell's `StagedEffects`.

`ensure_effect_components` inserts `BoundEffects + StagedEffects` on any cell that doesn't have them (cells are spawned without these components — see `breaker-game/src/run/node/systems/spawn_cells_from_layout/system.rs` lines 148–188).

---

## Step 5 — Died trigger fires on a Cell

**File:** `breaker-game/src/effect/triggers/died.rs`

`bridge_died` (lines 18–36) reads `RequestCellDestroyed` messages (sent by `handle_cell_hit` when HP reaches 0 — `breaker-game/src/cells/systems/handle_cell_hit/system.rs` lines 69–74). The cell entity is still alive at this point (two-phase destruction).

`bridge_died` calls `evaluate_staged_effects(&Trigger::Died, cell_entity, ...)` on `msg.cell`.

If `When(Died, [Do(Explode)])` is in the cell's `StagedEffects`, `walk_staged_node` matches it, calls `commands.fire_effect(cell_entity, Explode { range: 48.0, damage_mult: 1.0 }, "Powder Keg")`.

---

## Step 6 — Explode effect fires

**File:** `breaker-game/src/effect/effects/explode/effect.rs`

`fire()` (lines 30–57) is called with `entity = cell_entity`:
1. Reads `Position2D` from `cell_entity` — **the cell's position** (correct: explode originates at the dying cell)
2. Reads `ActiveDamageBoosts` from `cell_entity` — likely absent; defaults to multiplier 1.0
3. Reads `BoltBaseDamage` from `cell_entity` — **likely absent** on cells; falls back to `DEFAULT_BOLT_BASE_DAMAGE` (defined in `breaker-game/src/bolt/resources/`)
4. Spawns an `ExplodeRequest` entity at the cell's position with `damage_mult = 1.0 * 1.0 = 1.0`, `base_damage = DEFAULT_BOLT_BASE_DAMAGE`

`process_explode_requests` (lines 66–93) runs in `FixedUpdate` after `PhysicsSystems::MaintainQuadtree`. It queries the `CollisionQuadtree` for all cells within 48 world units of the explode position, sends `DamageCell` for each, then despawns the request entity.

**The explode does fire at the dying cell's position — this part is correct.**

---

## Data Flow Summary

```
[ChipSelected("Powder Keg")]
    → dispatch_chip_effects (Update)
    → Breaker BoundEffects: When(NodeStart, On(Bolt, permanent=true, [When(Impacted(Cell), [...])]))

[OnEnter(PlayingState::Active)]
    → bridge_node_start
    → Bolt BoundEffects: When(Impacted(Cell), [On(Cell, permanent=false, [When(Died, [Do(Explode)])])])

[BoltImpactCell { bolt, cell }]
    → bridge_impacted_bolt_cell (FixedUpdate, EffectSystems::Bridge)
    → evaluate_bound_effects(Impacted(Cell), bolt)
    → bolt StagedEffects += On(target: Cell, permanent=false, [When(Died, ...)])
    → evaluate_staged_effects(Impacted(Cell), bolt)
    → ResolveOnCommand { target: Cell, permanent: false, ... } queued
    [end of frame: commands apply]
    → ALL Cell entities get StagedEffects += When(Died, [Do(Explode)])

[RequestCellDestroyed { cell: ANY_CELL }]
    → bridge_died (FixedUpdate, EffectSystems::Bridge)
    → evaluate_staged_effects(Died, cell_entity)
    → FireEffectCommand { entity: cell_entity, Explode { range: 48.0, damage_mult: 1.0 } }
    [end of frame: commands apply]
    → ExplodeRequest spawned at cell's Position2D

[FixedUpdate, after PhysicsSystems::MaintainQuadtree]
    → process_explode_requests
    → DamageCell sent for all cells within 48 units of explode position
```

---

## State Machine

There is no explicit state machine for this effect. It uses `StagedEffects` as implicit transient state:

| State | Storage | Contents |
|-------|---------|----------|
| After chip selected | Breaker BoundEffects | When(NodeStart, ...) |
| After NodeStart | Bolt BoundEffects | When(Impacted(Cell), [On(Cell, ...)]) |
| After bolt hits ANY cell | All Cell StagedEffects | When(Died, [Do(Explode)]) |
| Cleared | After cell dies — staged entry is consumed | — |

---

## Edge Cases

### Bug: On(target: Cell) retargets to ALL cells, not the hit cell

`ResolveOnCommand` has no concept of "the entity that triggered this chain." `resolve_target_from_world(Target::Cell, world)` returns all cells (line 202–209 of `ext.rs`). There is no `Target::This` or `Target::Context` variant.

**Observed behavior:** The bolt hits cell A. `When(Died, [Do(Explode)])` is pushed to ALL cells. The next cell to die — which could be cell B from a different bolt hit, or even an unrelated cell — will trigger an explosion.

**Intended behavior (from RON design):** Only the cell that was actually hit by the bolt should have the `When(Died, ...)` watcher installed.

**This pattern is not aspirational — it compiles and runs, but produces incorrect game behavior.** There are no tests covering the specific Powder Keg multi-cell scenario.

### `StagedEffects` accumulate on cells across multiple bolt impacts

Because `permanent: false` sends children to `StagedEffects`, each bolt hit on any cell re-broadcasts `When(Died, [Do(Explode)])` to all cells. Cells accumulate multiple copies in their `StagedEffects` — one per bolt-cell impact this node. A cell with N copies will attempt to fire N explosions when it dies (but only the first `When(Died)` match is consumed per `evaluate_staged_effects` call per frame, since `retain` removes matching entries; however `walk_staged_node` processes all entries in one pass, so all N copies could fire in one evaluation frame if `bridge_died` runs once).

Actually re-reading `evaluate_staged_effects` (lines 42–47): `staged.0.retain(|(chip_name, node)| !walk_staged_node(...))` with `additions` collected separately. Multiple `When(Died, ...)` entries will each be tested against the `Died` trigger in the same `retain` pass — all matching ones are consumed AND their `Do` effects fire. So a cell hit by 3 bolts before dying will fire 3 explosions (3 entries × each spawning a separate `ExplodeRequest`).

### No `BoundEffects`/`StagedEffects` on cells at spawn time

Cells are spawned without `BoundEffects` or `StagedEffects` (see `spawn_cells_from_layout/system.rs`). `ensure_effect_components` in `TransferCommand` inserts them lazily when `ResolveOnCommand` first targets a cell. This is by design and works correctly.

### Explode uses cell's damage boost, not bolt's

`fire()` reads `ActiveDamageBoosts` from `entity` (the cell entity, line 40–42 of `effect.rs`). Cells do not have `ActiveDamageBoosts`; the default multiplier of 1.0 is used. The player's damage boost chips (which affect the bolt) are **not applied** to Powder Keg explosions. This may or may not be intended.

### Explode uses DEFAULT_BOLT_BASE_DAMAGE, not bolt's current damage

`fire()` reads `BoltBaseDamage` from `entity` (the cell, line 44–46). Cells have no `BoltBaseDamage`. The fallback is `DEFAULT_BOLT_BASE_DAMAGE`. This is always the base value regardless of any damage amplifiers on the bolt (e.g. from Amp chips).

### Multi-bolt scenario

With multiple bolts active, each bolt that hits a cell will independently broadcast `When(Died, ...)` to all cells. The entries are not deduplicated.

---

## Key Question: Is the nested On(target: Cell) pattern implemented?

**It is implemented but incorrect.** The infrastructure exists: `EffectNode::On` inside `StagedEffects` is handled by `walk_staged_node` (lines 138–152), which queues `ResolveOnCommand`. `ResolveOnCommand` transfers children to target entities. The code path compiles and executes.

The gap is that `ResolveOnCommand` resolves `Target::Cell` to ALL cells, not to the specific cell that triggered the chain. The `BoltImpactCell` message carries `msg.cell` (the specific cell entity), but this information is lost when `On(Cell, ...)` is pushed to the bolt's `StagedEffects` — the staged effect has no memory of which cell triggered it.

To fix this correctly, the effect system would need a way to bind a specific entity to a `Target::Cell` context at the point `Impacted(Cell)` fires. There is no such mechanism today. The closest approach would be a new `Target::Triggering` variant (or a context-entity field on `StagedEffects` entries), or restructuring Powder Keg as `On(target: Cell)` at the chip level so that `ResolveOnCommand` runs immediately at NodeStart against all cells and installs `When(Impacted(Bolt), When(Died, [Do(Explode)]))` permanently on each cell — but that changes the semantics (every cell, not just the hit one, would explode on death).

---

## Key Files

- `breaker-game/assets/chips/standard/powder_keg.chip.ron` — chip definition
- `breaker-game/src/effect/core/types/definitions/enums.rs` — `RootEffect`, `EffectNode`, `Trigger`, `Target`, `EffectKind::Explode` definitions (lines 131–395)
- `breaker-game/src/chips/systems/dispatch_chip_effects/system.rs` — chip effect binding; deferred dispatch path (lines 73–89); `dispatch_children` (lines 98–133)
- `breaker-game/src/effect/triggers/evaluate/system.rs` — `walk_bound_node` (line 49), `walk_staged_node` (line 74); On-node handling (lines 138–152)
- `breaker-game/src/effect/commands/ext.rs` — `ResolveOnCommand` (lines 171–215); `resolve_target_from_world` (lines 196–215) — **the bug lives here** (Target::Cell resolves to all cells)
- `breaker-game/src/effect/triggers/impacted/system.rs` — `bridge_impacted_bolt_cell` (lines 26–60); fires `Impacted(Cell)` on bolt
- `breaker-game/src/effect/triggers/died.rs` — `bridge_died` (lines 18–36); fires `Died` on dying entity
- `breaker-game/src/effect/effects/explode/effect.rs` — `fire()` (lines 30–57); reads position from entity (cell position); `process_explode_requests` (lines 66–93)
- `breaker-game/src/effect/triggers/node_start.rs` — `bridge_node_start` (line 15); installs bolt effects on `OnEnter(PlayingState::Active)`
- `breaker-game/src/run/node/systems/spawn_cells_from_layout/system.rs` — cells spawned without `BoundEffects`/`StagedEffects` (lines 148–188)
