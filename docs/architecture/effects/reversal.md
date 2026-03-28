# Reversal

**Every effect defines `reverse()`.** Every reverse does meaningful cleanup.

## What Reverse Does Per Effect Type

| Effect type | What reverse does |
|------------|-------------------|
| Passive buffs (SpeedBoost, DamageBoost, Piercing, SizeBoost, BumpForce) | Removes the matching entry from the Active* vec. Recalculation system picks up the change. |
| Entity-spawning effects (Shockwave, SpawnBolts, ChainBolt, SpawnPhantom, GravityWell, Shield, SecondWind, Pulse, TetherBeam) | Despawns the spawned entity/entities if still alive. |
| Component-inserting effects (Attraction, RampingDamage, QuickStop) | Removes the inserted components from the entity. |
| State-modifying effects (LoseLife, TimePenalty) | Undoes the state change (restore life, restore time). |
| Fire-and-forget effects (Explode, ChainLightning, PiercingBeam, RandomEffect, EntropyEngine) | No-op reverse — these are instant effects with no persistent state to undo. |

## The Two Kinds of Reversal

When Until desugars, it records what it set up into a `Reverse` node. The Reverse node stores two separate lists because the Until's children fall into two categories that need different reversal strategies:

### 1. Effects from `Do` children → call `reverse()`

When Until encounters a `Do(effect)` child, it fires the effect immediately. To undo this later, the effect is stored in `Reverse.effects`. On reversal, each effect is reversed by calling `commands.reverse_effect(entity, effect)`, which calls the effect's own `reverse()` function. The effect module knows how to undo itself (remove from vec, despawn entity, remove component, etc.).

**Example:** `Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])` — the SpeedBoost was fired immediately. On reversal, `SpeedBoost.reverse()` removes the 1.3 multiplier from ActiveSpeedBoosts.

### 2. Chains from non-Do children → remove from BoundEffects

When Until encounters a non-Do child (like a `When` node), it pushes that child to the entity's **BoundEffects** as a recurring chain. To undo this later, the entire chain is stored in `Reverse.chains`. On reversal, each chain is found and **removed from BoundEffects** — no `reverse()` call needed because the chain itself was never "fired," it was just installed. Removing it stops it from matching future triggers.

**Example (non-Do pushed to BoundEffects by Until):** `Until(trigger: PerfectBumped, permanent: true, then: [When(trigger: Impacted(Cell), then: [Do(Shockwave(...))])])` — Until pushes the When chain to BoundEffects (recurring). On reversal, the When chain is found in BoundEffects and removed. No shockwave reverse is called — the shockwaves that already fired are gone (fire-and-forget). Only the recurring chain is removed to prevent future shockwaves.

**Example (non-Do pushed to StagedEffects via On, permanent: false):** `When(trigger: PerfectBumped, then: [On(target: Bolt, then: [When(trigger: Impacted(Cell), then: [Do(DamageBoost(1.5))])])])` — when PerfectBumped fires, the On pushes the inner When to the bolt's StagedEffects (one-shot, `permanent: false`). On the next cell hit, the When matches, DamageBoost fires, and the entry is consumed automatically. If it is still present in StagedEffects Reverse removes it, if it isn't, Reverse does nothing.

**Example (non-Do pushed to BoundEffects via On, permanent: true):** `When(trigger: PerfectBumped, then: [On(target: Bolt, permanent: true, then: [When(trigger: Impacted(Cell), then: [Do(DamageBoost(1.5))])])])` — when PerfectBumped fires, the On pushes the inner When to the bolt's **BoundEffects** (recurring, `permanent: true`). The bolt now gets a damage boost on every cell hit, permanently. If this was inside an Until, the Reverse node would store this chain and remove it from the bolt's BoundEffects on reversal — stopping the recurring damage boost.

## Reverse Node Structure

```rust
Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> }
```

- `effects` — effects that were fired via `Do`. Reversed by calling `commands.reverse_effect(entity, effect)` for each.
- `chains` — nodes that were pushed to BoundEffects. Reversed by finding and removing each from the entity's BoundEffects.

These are fundamentally different operations:
- **Effects** were *applied* to the entity → they need to be *undone* (the effect's reverse function handles the specifics)
- **Chains** were *installed* on the entity → they need to be *removed* (just delete the matching entry from BoundEffects)

## Reversal Flow

1. Until desugars → fires Do children, pushes non-Do children to BoundEffects, replaces itself with `When(trigger, [Reverse(effects, chains)])`
2. Until-trigger fires → When matches → Reverse executes:
   - For each entry in `effects`: `commands.reverse_effect(entity, effect)` → queues ReverseEffectCommand
   - For each entry in `chains`: find matching node in entity's BoundEffects and remove it
3. ReverseEffectCommand applies at `apply_deferred` → calls `effect.reverse(entity, world)`
4. The effect's reverse function does the actual cleanup (remove from vec, despawn entity, etc.)
