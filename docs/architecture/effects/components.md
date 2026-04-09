# Components

Both `BoundEffects` and `StagedEffects` are **always present** on any entity that participates in the effect system. Neither is optional — StagedEffects may be empty, but it's always there. Trigger systems query `(Entity, &BoundEffects, &mut StagedEffects)` without `Option`.

## BoundEffects

```rust
#[derive(Component, Debug, Default, Clone)]
pub struct BoundEffects(pub Vec<(String, EffectNode)>);
```

Permanent effect trees on an entity. Each entry is `(chip_name, node)` where `chip_name` is the name of the chip that installed this chain (empty string for breaker-defined chains). The chip name is carried for **damage attribution** — so the game can trace which chip caused a shockwave, damage boost, or other effect.

Populated by:
- Chip dispatch (chip domain)
- Breaker init (breaker domain)
- Cell init (cell domain — optional `effects` field, defaults to None)
- Until nodes installing recurring When chains

**Never consumed by trigger evaluation** — entries persist and re-evaluate each time a matching trigger fires.

Until can push When children here (not StagedEffects) when those children represent recurring behavior that should fire on every matching trigger until reversed. On reversal, the Reverse node removes those entries.

## StagedEffects

```rust
#[derive(Component, Debug, Default, Clone)]
pub struct StagedEffects(pub Vec<(String, EffectNode)>);
```

Working set of partially-resolved chains. Same `(chip_name, node)` tuple as BoundEffects — the chip name propagates through so that effects fired from staged chains retain attribution.

Entries are **consumed when matched**. Populated by:
- BoundEffects evaluation (non-Do children of matching When nodes get pushed here)
- On transfers from other entities
- Until desugaring (Until is replaced with a `When(trigger, [Reverse(...)])` entry)

**Always present, may be empty.** Spawned alongside BoundEffects on every effect-participating entity. An empty StagedEffects means no partially-resolved chains are pending — this is the normal state between trigger evaluations.

## Command Extensions

Effect execution and cross-entity mutation are deferred through `EffectCommandsExt` on Bevy `Commands`. Bridge systems take `Commands` as a parameter and call extension methods (fire_effect, stamp_effect, transfer_effect, etc.). Each command implements `EntityCommand` and executes with `&mut World` when Bevy flushes the command queue.

See [Commands](commands.md) for the full EffectCommandsExt trait.
