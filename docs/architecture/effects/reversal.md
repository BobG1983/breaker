# Reversal

**Every reversible effect defines `reverse()`.** Every reverse does meaningful cleanup. Reversal is not a node type and is not "desugared" anywhere — it's a function call from one of three call sites: `commands.reverse_effect`, the `evaluate_conditions` During state machine, or the `UntilEvaluateCommand` Until state machine.

This file covers what reverse means per effect category, the `Reversible` trait contract, and where each reversal call site is.

## The Reversible trait

```rust
pub trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        self.reverse(entity, source, world);
    }
}
```

`Reversible: Fireable` — every reversible config can also be fired. The reverse is the inverse of fire.

The default `reverse_all_by_source` calls `reverse` once. Stack-based passives override it to remove every entry from a given source via `EffectStack::retain_by_source`. Singleton effects keep the default because there is at most one active instance — one reverse undoes everything.

### What reverse does per category

| Effect category | Examples | What `reverse()` does |
|---|---|---|
| **Stack-based passive** | `SpeedBoost`, `DamageBoost`, `Piercing`, `SizeBoost`, `BumpForce`, `QuickStop`, `Vulnerable`, `RampingDamage`, `Anchor`, `FlashStep`, `Attraction` | Removes the matching entry from the entity's `EffectStack<Self>`. The recalculation system picks up the change next tick. |
| **Spawned entity** | `Pulse`, `Shield`, `SecondWind`, `GravityWell`, `CircuitBreaker`, `EntropyEngine` | Despawns the spawned child entity if still alive. Marker components on the parent are removed if applicable. |
| **Fire-and-forget** (NOT in `ReversibleEffectType`) | `Shockwave`, `Explode`, `ChainLightning`, `PiercingBeam`, `SpawnBolts`, `SpawnPhantom`, `ChainBolt`, `MirrorProtocol`, `TetherBeam`, `LoseLife`, `TimePenalty`, `Die`, `RandomEffect` | These are not in `ReversibleEffectType`. They cannot appear in `ScopedTree::Fire` — only in `Tree::Fire` outside any scope. There is nothing to reverse. |

The stack-based passives override `reverse_all_by_source` to call `EffectStack::retain_by_source` so that "remove every speed boost from this chip" is a single call, not N calls. The `evaluate_conditions` During state machine relies on this — when it tears down a scoped tree it uses `reverse_all_by_source_dispatch` for stack effects to ensure all instances of a given source are gone.

## reverse_dispatch and friends

Three dispatch functions live in `effect_v3/dispatch/reverse_dispatch/system.rs`:

```rust
pub fn reverse_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
);

pub fn fire_reversible_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
);

pub fn reverse_all_by_source_dispatch(
    effect: &ReversibleEffectType,
    entity: Entity,
    source: &str,
    world: &mut World,
);
```

All three are mechanical matches on the `ReversibleEffectType` variant that delegate to the corresponding config struct's trait method:

```rust
pub fn reverse_dispatch(effect: &ReversibleEffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.reverse(entity, source, world),
        ReversibleEffectType::SizeBoost(config)  => config.reverse(entity, source, world),
        // ... one arm per reversible variant
    }
}
```

Three functions exist because three different call sites need three different semantics:

- `reverse_dispatch` — single-instance reverse, used when the caller knows there is exactly one effect to undo (e.g. `commands.reverse_effect`, `UntilEvaluateCommand` Shape 1 reverse).
- `fire_reversible_dispatch` — fires from the `ReversibleEffectType` enum without first widening to `EffectType`. Used by Until and During code paths so they can stay in the reversible enum without an `EffectType::from` round-trip.
- `reverse_all_by_source_dispatch` — calls `Reversible::reverse_all_by_source`. Used when the caller wants to undo every effect from a source (during teardown of a scoped tree). For singleton effects this is identical to `reverse_dispatch`; for stack-based passives it removes every entry rather than just one.

## Reversal call sites

There are exactly three places in the codebase where reversal happens.

### Call site 1: `commands.reverse_effect`

Queues `ReverseEffectCommand`. At command flush, calls `reverse_dispatch(&effect, entity, &source, world)`.

Used by:
- `evaluate_conditions` Shape D disarm — when an armed `On(participant, Fire(reversible))` needs reversing on the participant.

This is the only "external API" reversal — the rest of the system reverses through direct dispatch calls inside its own commands.

### Call site 2: `evaluate_conditions` (the During state machine)

Lives in `effect_v3/conditions/evaluate_conditions/system.rs`. The condition poller iterates `Tree::During` entries every tick and calls `fire_scoped_tree` / `reverse_scoped_tree` on transitions:

```rust
if !was_active && is_true {
    fire_scoped_tree(&inner, entity, &source, world);
    da.0.insert(source);
} else if was_active && !is_true {
    reverse_scoped_tree(&inner, entity, &source, world);
    da.0.remove(&source);
}
```

`reverse_scoped_tree` matches on the `ScopedTree` shape:

- `ScopedTree::Fire(reversible)` → `reverse_dispatch(reversible, entity, source, world)`.
- `ScopedTree::Sequence(reversibles)` → `reverse_dispatch` on each.
- `ScopedTree::When(_, _)` → remove the armed `#armed[0]` entry from `BoundEffects`, then call `reverse_armed_tree` to undo any effects the armed When may have fired (uses `reverse_all_by_source_dispatch` because effects in armed inner trees were fired against the armed key).
- `ScopedTree::On(_, ScopedTerminal::Fire(reversible))` → drain the owner's `ArmedFiredParticipants` for this armed key; for each tracked participant, queue `commands.reverse_effect(participant, reversible, armed_key)`. This is the one place the loop hops back to call site 1.
- `ScopedTree::During(..)` → handled by Shape A reversal (the inner During has its own `#installed[0]` entry which is managed independently).

### Call site 3: `UntilEvaluateCommand` (the Until state machine)

Lives in `effect_v3/walking/until/system.rs`. Same shape as `evaluate_conditions` reversal but driven by trigger gating instead of condition cycling.

For Shapes 1–3 (Fire / Sequence / nested gate), reversal calls `reverse_scoped_tree` which mirrors the `evaluate_conditions` helper.

For Shape 4 (Until wrapping During), reversal calls `teardown_installed_during`, which:

1. Removes the `#installed[0]` entry from `BoundEffects`.
2. If the During was active (source in `DuringActive`), calls `reverse_scoped_tree_by_source` on the inner — uses `reverse_all_by_source_dispatch` so every effect instance fired from the install key gets removed in one pass.
3. Removes the install key from `DuringActive`.

The same effect is then removed from `UntilApplied` and the Until entry is dropped from `BoundEffects`.

## Where there is no Reverse node

The old design had a `Reverse` variant in the tree enum that recorded what to undo. It does not exist in the current code:

```rust
// This does NOT exist in Tree:
// Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> }
```

The walker never sees a "reverse this" node. Reversal is purely a function-call concern, not a tree-node concern. The state needed for reversal lives on the entity (`UntilApplied`, `DuringActive`, `ArmedFiredParticipants`) and inside the per-effect storage (`EffectStack<C>` for stack passives, marker components for singletons).

## What this means in practice

When you author a chip and use `Until` or `During`, the reversal is automatic — you do not need to spell it out. The state machine knows what to undo because the inner `ScopedTree::Fire` carries a `ReversibleEffectType`, and that enum has exactly one trait method to call (`Reversible::reverse`). The compiler enforces reversibility at the type level.

When you author a new effect that should be reversible, implement `Reversible` on the config struct (in addition to `Fireable`), add the variant to `ReversibleEffectType`, and add the conversion arms in `From<ReversibleEffectType>` and `TryFrom<EffectType>`. The compiler will then insist you handle it in `reverse_dispatch`, `fire_reversible_dispatch`, and `reverse_all_by_source_dispatch`. See `adding_effects.md`.
