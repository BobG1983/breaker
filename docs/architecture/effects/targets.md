# Targets and Participants

There are **three** target concepts in the effect system, and they live at different layers. Conflating them is the most common source of confusion.

| Type | Where it appears | What it identifies |
|---|---|---|
| `StampTarget` | `RootNode::Stamp(StampTarget, Tree)` | Which entities the chip dispatch system installs the tree onto |
| `EntityKind` | `RootNode::Spawn(EntityKind, Tree)`, `Trigger::Impacted(EntityKind)`, `Trigger::DeathOccurred(EntityKind)` | Entity *type* — for spawn registration and trigger filtering |
| `ParticipantTarget` | `Tree::On(ParticipantTarget, Terminal)` | A *role* in a trigger event (Bolt vs Breaker in a bump, Impactor vs Impactee in a collision) |

`StampTarget` is consumed at dispatch time by the chip system and is never seen by the walker. `EntityKind` filters at trigger evaluation time. `ParticipantTarget` resolves at walk time inside `evaluate_on` against the active `TriggerContext`.

## StampTarget — root-level entity selection

```rust
pub enum StampTarget {
    // Singletons (use the primary entity)
    Bolt,
    Breaker,

    // Active = entities that exist now
    ActiveBolts,
    ActiveCells,
    ActiveWalls,
    ActiveBreakers,

    // Every = existing + future spawns
    EveryBolt,
    EveryCell,
    EveryWall,
    EveryBreaker,

    // Bolt subsets
    PrimaryBolts,    // bolts marked PrimaryBolt
    ExtraBolts,      // bolts marked ExtraBolt
}
```

### Resolution at chip dispatch

`chips/systems/dispatch_chip_effects/system.rs` resolves `StampTarget` to a concrete entity list using a bundled `DispatchTargets` system param:

```rust
fn resolve_target_entities(target: StampTarget, targets: &DispatchTargets) -> Vec<Entity> {
    match target {
        StampTarget::Breaker
        | StampTarget::ActiveBreakers
        | StampTarget::EveryBreaker => targets.breakers.iter().collect(),

        StampTarget::Bolt
        | StampTarget::ActiveBolts
        | StampTarget::EveryBolt
        | StampTarget::PrimaryBolts
        | StampTarget::ExtraBolts => targets.bolts.iter().collect(),

        StampTarget::ActiveCells | StampTarget::EveryCell => targets.cells.iter().collect(),
        StampTarget::ActiveWalls | StampTarget::EveryWall => targets.walls.iter().collect(),
    }
}
```

For `Breaker` targets the dispatch happens **immediately** because breakers exist at chip-select time. For everything else (bolts, cells, walls), the dispatch wraps the tree in a deferred install — see `dispatch.md`.

`Active*` and `Every*` collapse to the same Entity list at resolution time. The distinction matters only when the entity set changes between nodes: `ActiveCells` is a one-time snapshot; `EveryCell` would also need a `SpawnStampRegistry` entry so that future cell spawns receive the tree. The current implementation does not yet branch on this — both variants resolve to the same query — and `Spawn`-based registration is the proper mechanism for "future spawns" semantics.

### `Spawn` root vs `Stamp` root

```rust
pub enum RootNode {
    Stamp(StampTarget, Tree),    // dispatch-time install on resolved entities
    Spawn(EntityKind, Tree),     // future-spawn install via SpawnStampRegistry
}
```

`Stamp` is for "install on these entities right now." `Spawn` is for "install on every entity of this kind that is added in the future." The latter writes a `SpawnStampRegistry` entry; the per-kind watcher systems (`stamp_spawned_bolts`, `stamp_spawned_cells`, ...) iterate `Added<Bolt>` etc. each tick and call `commands.stamp_effect` on matching new entities. See `dispatch.md` for the watchers.

## EntityKind — type filtering for triggers and spawns

```rust
pub enum EntityKind {
    Cell,
    Bolt,
    Wall,
    Breaker,
    Any,
}
```

Used by:
- `Trigger::Impacted(EntityKind)` / `Trigger::ImpactOccurred(EntityKind)` — match collisions involving entities of this kind.
- `Trigger::Killed(EntityKind)` / `Trigger::DeathOccurred(EntityKind)` — match deaths of entities of this kind.
- `RootNode::Spawn(EntityKind, Tree)` — register a tree to install on every entity of this kind on `Added<T>`.

`Any` is valid for trigger payloads (matches collisions / deaths regardless of entity type) but is not valid for `Spawn`-based registration because the watcher systems are per-kind. The watchers ignore `Any` entries:

> **`stamp_spawned_bolts`**: "Iterates the `SpawnStampRegistry.entries` for each newly-added `Bolt` and delegates to `commands.stamp_effect` for every entry whose `EntityKind` exactly matches `EntityKind::Bolt`. `EntityKind::Any` entries are ignored — wildcarding is reserved for trigger-side matching, not spawn-time stamping."

## ParticipantTarget — role within a trigger event

```rust
pub enum BumpTarget       { Bolt, Breaker }
pub enum ImpactTarget     { Impactor, Impactee }
pub enum DeathTarget      { Victim, Killer }
pub enum BoltLostTarget   { Bolt, Breaker }

pub enum ParticipantTarget {
    Bump(BumpTarget),
    Impact(ImpactTarget),
    Death(DeathTarget),
    BoltLost(BoltLostTarget),
}
```

Used only inside `Tree::On(ParticipantTarget, Terminal)`. Resolution happens at walk time in `walking/on/system.rs`:

```rust
const fn resolve_participant(
    target: ParticipantTarget,
    context: &TriggerContext,
) -> Option<Entity> {
    match (target, context) {
        (ParticipantTarget::Bump(BumpTarget::Bolt),    TriggerContext::Bump { bolt, .. })    => *bolt,
        (ParticipantTarget::Bump(BumpTarget::Breaker), TriggerContext::Bump { breaker, .. }) => Some(*breaker),
        (ParticipantTarget::Impact(ImpactTarget::Impactor), TriggerContext::Impact { impactor, .. }) => Some(*impactor),
        (ParticipantTarget::Impact(ImpactTarget::Impactee), TriggerContext::Impact { impactee, .. }) => Some(*impactee),
        (ParticipantTarget::Death(DeathTarget::Victim), TriggerContext::Death { victim, .. })  => Some(*victim),
        (ParticipantTarget::Death(DeathTarget::Killer), TriggerContext::Death { killer, .. })  => *killer,
        (ParticipantTarget::BoltLost(BoltLostTarget::Bolt),    TriggerContext::BoltLost { bolt, .. })    => Some(*bolt),
        (ParticipantTarget::BoltLost(BoltLostTarget::Breaker), TriggerContext::BoltLost { breaker, .. }) => Some(*breaker),
        _ => None,
    }
}
```

Two cases return `Option<Entity>` rather than `Some(...)`:

- `Bump(Bolt)` against `BumpWhiff` / `NoBump` — those events have no bolt, so the `On` is silently skipped.
- `Death(Killer)` against an environmental death — no killer, `On` skipped.

Mismatched pairs (e.g. `Bump(Bolt)` against `TriggerContext::Impact`) also return `None`. The mismatch indicates a chip wired up against the wrong trigger family — silent skip is the chosen failure mode rather than a panic, because the wrong-trigger combinations cannot construct a working chain in any case.

## Why the three concepts don't merge

It would be tempting to have a single `Target` enum that covers all of "Bolt", "Breaker", "Cell", "Wall" plus their plurals plus the role enums. Three reasons not to:

1. **`StampTarget` cannot have role variants.** "Impactor" doesn't make sense at chip-select time — there's no impact event yet.
2. **`ParticipantTarget` cannot have plural variants.** "AllBolts" doesn't make sense as a participant — the trigger event has at most one bolt.
3. **`EntityKind` is the only one that supports `Any`.** Wildcards make sense for trigger filtering but break dispatch (which needs concrete entities) and participant resolution (which needs a single entity).

Keeping the three concepts as separate enums means the type checker enforces the right enum at each layer.
