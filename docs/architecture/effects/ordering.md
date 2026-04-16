# System Ordering

All effect systems run in `FixedUpdate`. The ordering is configured by `EffectV3Plugin::build` via system sets defined in `effect_v3/sets.rs`.

## EffectV3Systems

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectV3Systems {
    Bridge,
    Tick,
    Conditions,
    Reset,
}
```

| Set | What runs there | Schedule |
|---|---|---|
| **Bridge** | Trigger bridge systems (one per trigger category × scope). `SpawnStampRegistry` watcher systems (`stamp_spawned_bolts`, etc.). | `FixedUpdate` |
| **Tick** | Per-effect runtime systems registered by `Fireable::register` — shockwave expansion, pulse interval ticking, gravity well duration, etc. | `FixedUpdate` |
| **Conditions** | `evaluate_conditions` — the During poller. | `FixedUpdate` |
| **Reset** | Per-node reset systems (e.g. `reset_entropy_counter`, `reset_ramping_damage`). | `OnEnter(NodeState::Loading)` |

The set ordering inside `FixedUpdate` is configured by `EffectV3Plugin::build`:

```rust
app.configure_sets(
    FixedUpdate,
    (
        EffectV3Systems::Bridge,
        EffectV3Systems::Tick.after(EffectV3Systems::Bridge),
        EffectV3Systems::Conditions.after(EffectV3Systems::Tick),
    ),
);
```

So the order is **Bridge → Tick → Conditions** within FixedUpdate. `Reset` is its own thing — it runs only on `OnEnter(NodeState::Loading)` and is not part of the FixedUpdate chain at all.

## Bridge: trigger bridges and spawn watchers

Every trigger category registers its bridge systems into `EffectV3Systems::Bridge` via its `register::register(app)` function. For example, `triggers::bump::register::register`:

```rust
pub fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            bridges::on_bumped,
            bridges::on_perfect_bumped,
            bridges::on_early_bumped,
            bridges::on_late_bumped,
            bridges::on_bump_occurred,
            bridges::on_perfect_bump_occurred,
            bridges::on_early_bump_occurred,
            bridges::on_late_bump_occurred,
            bridges::on_bump_whiff_occurred,
            bridges::on_no_bump_occurred,
        )
            .in_set(EffectV3Systems::Bridge),
    );
}
```

Same pattern for `impact`, `death`, `bolt_lost`, `node`, `time`. The category modules are responsible for adding additional `.after()` constraints relative to the producing game systems (e.g. impact bridges go `.after(BoltSystems::CellCollision)`).

The `SpawnStampRegistry` watcher systems (`stamp_spawned_bolts`, `stamp_spawned_cells`, `stamp_spawned_walls`, `stamp_spawned_breakers`) are registered directly by `EffectV3Plugin::build` in `EffectV3Systems::Bridge`:

```rust
app.add_systems(
    FixedUpdate,
    (
        stamp_spawned_bolts,
        stamp_spawned_cells,
        stamp_spawned_walls,
        stamp_spawned_breakers,
    )
        .in_set(EffectV3Systems::Bridge),
);
```

They run alongside the trigger bridges. They use `Added<T>` queries so each newly spawned entity is processed exactly once.

## Tick: per-effect runtime systems

Each effect's `Fireable::register` impl can add tick / cleanup / per-frame systems into `EffectV3Systems::Tick`. The default `Fireable::register` is a no-op, but `EffectV3Plugin::build` calls `XxxConfig::register(&mut app)` for every config struct unconditionally:

```rust
effects::AnchorConfig::register(app);
effects::AttractionConfig::register(app);
effects::BumpForceConfig::register(app);
effects::ChainBoltConfig::register(app);
// ... 30 calls in total
```

This is the "no silently-dropped systems" guarantee: even if an effect currently has no per-tick logic, its `register(app)` call exists in the plugin so adding a system later cannot be forgotten.

Concrete examples of what runs in `Tick`:

- `apply_speed_boosts` — recalculates effective bolt speed from `EffectStack<SpeedBoostConfig>`.
- `tick_shockwave` — expands shockwave entity radius, applies damage, despawns when range exceeded.
- `tick_pulse` — interval-based pulse emitter.
- `tick_anchor` / `cleanup_anchor` — anchor lifecycle.
- `tick_gravity_well` — duration countdown for gravity wells.
- `tick_chain_lightning` — arc traversal and damage.

Tick ordering rules:

- Tick systems within the set run in parallel by default. Add `.after()` constraints when one effect's tick depends on another.
- Recalculation systems (`apply_*`) typically run after the corresponding stack mutation. Since stack mutations happen via commands (deferred until the next flush point), the recalculation in `Tick` naturally sees the post-mutation state.

## Conditions: the During poller

`evaluate_conditions` is registered into `EffectV3Systems::Conditions`:

```rust
app.add_systems(
    FixedUpdate,
    conditions::evaluate_conditions.in_set(EffectV3Systems::Conditions),
);
```

It runs once per FixedUpdate, after `Tick`. It is an exclusive system (`fn evaluate_conditions(world: &mut World)`) because it needs both the iteration phase (immutable borrow) and the fire/reverse phase (mutable borrow) within a single tick.

Why after `Tick`? Conditions like `ShieldActive` depend on entity state (`ShieldWall` entities) that may have just been spawned or despawned by `Tick` systems. Polling after `Tick` ensures the predicate sees the final per-frame state. Without this ordering, a tick that destroys the last `ShieldWall` would leave the condition reading "true" until the next frame.

## Reset: per-node reset systems

`Reset` is intentionally not part of the FixedUpdate ordering chain — it runs on `OnEnter(NodeState::Loading)`:

```rust
// Effect modules with state that must reset between nodes register here.
// Example (conceptual):
//   app.add_systems(OnEnter(NodeState::Loading), reset_entropy_counter.in_set(EffectV3Systems::Reset));
```

Examples of effects that need per-node reset:
- `EntropyEngine` — kill counter resets between nodes.
- `RampingDamage` — accumulator resets between nodes.

## Cross-domain ordering

Bridge systems must run **after** the game systems that produce their messages. Each trigger category's `register::register` function adds the necessary `.after()` constraints — typically one per source-message type. Examples:

- Impact bridges run `.after(BoltSystems::CellCollision)` and `.after(BoltSystems::BreakerCollision)`.
- Bolt-lost bridge runs `.after(BoltSystems::BoltLost)`.
- Death bridge runs `.after(EntitySystems::Death)`.

Effect runtime systems in `Tick` run **after** `EffectV3Systems::Bridge`, so any commands queued by bridges have flushed and the effect entities they spawned exist before tick systems iterate them. This is implicit from the `EffectV3Systems` set ordering — Bridge → Tick.

## What is NOT scheduled

- **No "Until desugaring" system.** Until is handled inline by `evaluate_until` (which queues `UntilEvaluateCommand`). There is no separate desugaring pass.
- **No "Reverse node" processor.** Reversal is a function call from inside the condition poller and `UntilEvaluateCommand`, not a tree-walking pass.
- **No "Apply effects" system.** Effects apply through the standard Bevy command flush.
