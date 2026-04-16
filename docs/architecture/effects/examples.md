# Examples

Worked RON examples of common chip shapes. Each example shows the RON, what gets stamped, and what happens at runtime.

The variant names below are real `Tree` / `RootNode` / `EffectType` variants — copy them verbatim. Type names use UpperCamelCase; field names use snake_case.

## Passive: speed boost on equip

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig(
    multiplier: 1.5,
))))
```

1. **Dispatch**: chip selected → `dispatch_chip_effects` resolves `StampTarget::Bolt`, but bolts don't exist yet at chip-select time. The tree is stamped onto the breaker's `BoundEffects` (under the chip name).
2. **Next node starts**: bolts spawn; their `BoundEffects` gets the chip's tree via spawn watchers (or via deferred dispatch through `When(NodeStartOccurred, ...)` patterns — see `dispatch.md`).
3. **First walk** finds `Tree::Fire(SpeedBoost(...))` at the root → queues a `FireEffectCommand` → `SpeedBoostConfig::fire` pushes onto `EffectStack<SpeedBoostConfig>` on the bolt.

This case skips the `BoundEffects` entirely on the bolt — `dispatch_tree` checks for a top-level `Tree::Fire` and fires immediately rather than stamping.

## Bumped → speed boost (gated)

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::When(
    Trigger::PerfectBumped,
    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig(multiplier: 1.5))),
))
```

1. **Dispatch**: stamps `When(PerfectBumped, Fire(SpeedBoost))` into the bolt's `BoundEffects`.
2. **PerfectBumped fires**: bump bridge calls `walk_bound_effects` on the bolt with `Trigger::PerfectBumped`. The walker matches the When → recurses into `Fire(SpeedBoost)` → queues `FireEffectCommand`.
3. **Re-arms**: the When stays in `BoundEffects`. Next perfect bump fires the boost again.

## On bump → boost the bolt (participant redirect)

```ron
RootNode::Stamp(StampTarget::Breaker, Tree::When(
    Trigger::Bumped,
    Tree::On(
        ParticipantTarget::Bump(BumpTarget::Bolt),
        Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig(multiplier: 1.5))),
    ),
))
```

1. **Dispatch**: stamps `When(Bumped, On(Bump(Bolt), Fire(DamageBoost)))` into the breaker's `BoundEffects`.
2. **Bumped fires on the breaker**: bump bridge passes `TriggerContext::Bump { bolt: Some(bolt_e), breaker: breaker_e }`. The walker matches the When → recurses into the On → `evaluate_on` resolves `Bump(Bolt)` against the context → `Some(bolt_e)` → queues `FireEffectCommand` for the bolt.

The boost is applied to the *bolt* even though the chain lives on the *breaker*.

## Nested When (two events required)

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::When(
    Trigger::PerfectBumped,
    Tree::When(
        Trigger::Impacted(EntityKind::Cell),
        Tree::Fire(EffectType::Shockwave(ShockwaveConfig(
            base_range: 24.0,
            range_per_level: 6.0,
            stacks: 1,
            speed: 400.0,
        ))),
    ),
))
```

1. **Dispatch**: stamps the outer When into `BoundEffects`.
2. **PerfectBumped fires**: walker matches outer When → inner is a gate → `evaluate_when` arms the inner via `commands.stage_effect`. The freshly-staged `When(Impacted(Cell), Fire(Shockwave))` lands in `StagedEffects` after command flush.
3. **Bolt hits cell**: impact bridge walks staged first (gate matches), then bound (outer When matches but its inner is the same gate, so re-arms... but the inner already exists in staged, and `commands.stage_effect` doesn't dedupe — see note below). The staged inner fires the shockwave and is consumed by `walk_staged_effects` via entry-specific `remove_staged_effect`.
4. **Outer stays bound**: every subsequent perfect bump re-arms a new staged inner.

Note: a bolt that bumps perfectly twice in a row will queue two staged inner entries; whichever fires first is consumed. The staging path doesn't dedupe — multiple arms = multiple potential consumes.

## Once: first bolt-lost grants a second wind

```ron
RootNode::Stamp(StampTarget::Breaker, Tree::Once(
    Trigger::BoltLostOccurred,
    Tree::Fire(EffectType::SecondWind(SecondWindConfig())),
))
```

1. **Dispatch**: stamps the Once into the breaker's `BoundEffects`.
2. **First bolt lost**: bolt-lost bridge walks bound. The Once gate matches → inner is a Fire (not a gate) → walker recurses, fires `SecondWind`, then queues `commands.remove_effect(entity, source)`.
3. **Subsequent bolt-lost events**: the Once is gone, no further arms.

## During: speed boost while the node is active

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::During(
    Condition::NodeActive,
    ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig(multiplier: 1.3))),
))
```

1. **Dispatch**: stamps `During(NodeActive, Fire(SpeedBoost(1.3)))` into the bolt's `BoundEffects`.
2. **First walk**: `evaluate_during` queues a `DuringInstallCommand` that idempotently inserts the same During under `format!("{source}#installed[0]")` (no-op if it's already top-level — which it is here).
3. **Condition poller**: sees `NodeActive` is true (node is `Playing`), inserts `source` into `DuringActive`, fires `SpeedBoost(1.3)` via `fire_reversible_dispatch`.
4. **Node ends**: `NodeActive` becomes false. Poller removes from `DuringActive` and reverses the boost.
5. **Next node**: cycle repeats automatically — the original `During` entry is still in `BoundEffects`.

`SpeedBoost` is a reversible effect, so `ScopedTree::Fire(ReversibleEffectType::SpeedBoost)` type-checks. A non-reversible effect like `Shockwave` cannot be used here at the type level.

## Until: timed buff on perfect bump

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::When(
    Trigger::PerfectBumped,
    Tree::Until(
        Trigger::TimeExpires(2.0),
        ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig(multiplier: 1.3))),
    ),
))
```

1. **Dispatch**: stamps the outer When into the bolt's `BoundEffects`.
2. **PerfectBumped fires**: walker matches When → inner is a gate (Until counts as a gate for arming purposes) → arms the Until via `commands.stage_effect`.
3. **Next walk**: the staged Until is walked; `evaluate_until` queues an `UntilEvaluateCommand`. The command fires `SpeedBoost(1.3)` and inserts `source` into `UntilApplied`.
4. **Two seconds pass**: the time category's tick system decrements the per-entity countdown; when it hits zero, the time bridge fires `Trigger::TimeExpires(2.0)`. The `UntilEvaluateCommand` (queued again on the next walk) sees the gate matches the active trigger, reverses `SpeedBoost`, removes from `UntilApplied`, and removes the Until entry from `BoundEffects`.
5. **Outer When stays bound**: another perfect bump arms a fresh Until. Each Until is independent — the chip can produce overlapping timed buffs.

## Sequence: multi-effect grant

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::When(
    Trigger::PerfectBumped,
    Tree::Sequence(vec![
        Terminal::Fire(EffectType::DamageBoost(DamageBoostConfig(multiplier: 1.5))),
        Terminal::Fire(EffectType::SizeBoost(SizeBoostConfig(multiplier: 1.2))),
    ]),
))
```

1. **PerfectBumped fires**: walker matches When → inner is a Sequence → `evaluate_sequence` iterates the terminals → queues a `FireEffectCommand` for each.

Sequence is unconditional ordered fire/route. The surrounding `When` provides the gate.

## Cascade: shockwave on cell death

```ron
RootNode::Stamp(StampTarget::Bolt, Tree::When(
    Trigger::DeathOccurred(EntityKind::Cell),
    Tree::Fire(EffectType::Shockwave(ShockwaveConfig(
        base_range: 20.0,
        range_per_level: 4.0,
        stacks: 1,
        speed: 350.0,
    ))),
))
```

1. **Dispatch**: stamps into the bolt's `BoundEffects`.
2. **A cell dies anywhere**: death bridge fires the global `DeathOccurred(Cell)` trigger and walks every entity with effects. The bolt's chain matches → fires a shockwave on the bolt.

## Spawn root: install on every bolt the run produces

```ron
RootNode::Spawn(EntityKind::Bolt, Tree::When(
    Trigger::Impacted(EntityKind::Wall),
    Tree::Fire(EffectType::SpeedBoost(SpeedBoostConfig(multiplier: 1.1))),
))
```

1. **Dispatch**: writes the `(Bolt, chip_name, tree)` triple into `SpawnStampRegistry.entries`.
2. **A bolt is added** (existing or future): `stamp_spawned_bolts` watches `Added<Bolt>`, finds the registry entry whose `EntityKind == Bolt`, and calls `commands.stamp_effect(new_bolt, chip_name, tree.clone())`.
3. **The bolt's `BoundEffects`** now contains the When. From here it behaves like any other stamped chain.

`SpawnStampRegistry` entries persist across nodes — a chip with a `Spawn` root keeps applying its tree to every new bolt for the rest of the run. To remove the entry, the chip would need an explicit unequip path that removes from the registry vec.

## Why these examples don't show "All*" targets

There are no `AllBolts` / `AllCells` / `AllWalls` variants in `StampTarget`. The actual variants are `ActiveBolts` / `EveryBolt` (and `Active*` / `Every*` for cells, walls, breakers), which collapse to the same query result at dispatch time. Use `Spawn(EntityKind::*, ...)` for "install on every entity of this kind, including future spawns."
