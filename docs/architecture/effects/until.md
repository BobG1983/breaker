# Until and Desugaring

Until is the most complex node type. It provides duration-scoped effects — effects that are active for a period and then automatically reversed.

## EffectNode Definition

```rust
Until { trigger: Trigger, then: Vec<EffectNode> }
```

In RON:
```ron
Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
```

## Core Concept

Until means "apply these effects now, and undo them when this trigger fires." It is syntactic sugar in the chain data — at runtime, it gets **desugared** into simpler nodes that the trigger system already knows how to handle.

## Desugaring System

Until is **not processed by trigger systems**. A dedicated Until desugaring system handles it:

1. Runs **after all trigger bridge systems** in FixedUpdate
2. Queries all entities with BoundEffects and StagedEffects
3. Finds any `Until` nodes in either component
4. Desugars each one (see below)
5. Removes the original Until from whichever component it was in

This means:
- **Dispatch** can freely push Until nodes to BoundEffects — the desugaring system handles them on the next tick
- **Trigger systems** never see Until — they only see When, Do, Once, On, and Reverse
- **Ordering is explicit**: collision systems → trigger bridges → Until desugaring → timer system → apply_deferred

## How Desugaring Works

For each Until node found, the desugaring system processes its children and builds a Reverse node:

### Do children → fire immediately

```ron
Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
```

The `Do(SpeedBoost)` fires immediately on the entity via `commands.fire_effect()`. The effect is stored in `Reverse.effects` for later reversal.

### When children → push to BoundEffects (recurring)

```ron
Until(trigger: PerfectBumped, then: [
    When(trigger: Impacted(Cell), then: [Do(Shockwave(base_range: 32.0, ...))])
])
```

The `When(Impacted(Cell), ...)` is pushed to the entity's **BoundEffects** — it becomes a recurring chain that fires a shockwave on every cell hit. The chain is stored in `Reverse.chains` for later removal.

### Replace Until with When+Reverse

After processing all children, the Until is replaced in **StagedEffects** with:

```
When(until_trigger, [Reverse(effects: [...], chains: [...])])
```

This is a normal When node that the trigger system knows how to handle. When the until-trigger fires, the When matches, and the Reverse node does the cleanup.

## The Reverse Node

```rust
Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> }
```

Not a RON-facing node — created internally by Until desugaring only. Lives in StagedEffects inside a `When(trigger, [Reverse(...)])` wrapper.

- `effects` — Do effects that were fired. On reversal: `commands.reverse_effect(entity, effect)` for each.
- `chains` — When chains that were pushed to BoundEffects. On reversal: find and remove each from BoundEffects.

See [Reversal](reversal.md) for the full explanation of the two kinds of reversal.

## Complete Example: Simple timed buff

```ron
Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
```

**Desugaring** (Until desugaring system runs):
1. Fire `SpeedBoost(1.3)` on entity → bolt speeds up
2. Remove Until, replace with:
   ```
   When(TimeExpires(2.0), [Reverse(effects: [SpeedBoost(1.3)], chains: [])])
   ```
   This goes into StagedEffects.

**2 seconds later** (timer system decrements remaining):
1. `When(TimeExpires(0.0))` matches → Reverse executes
2. `commands.reverse_effect(entity, SpeedBoost(1.3))` → removes from ActiveSpeedBoosts
3. Entry consumed from StagedEffects

**If this Until was in BoundEffects** (e.g., from dispatch): the original BoundEffects entry that contained the Until stays as-is after desugaring? No — the Until is **removed** from BoundEffects by the desugaring system. The desugared When+Reverse lives in StagedEffects. If the Until was inside a When in BoundEffects (e.g., `When(PerfectBumped, [Until(...)])`), then the When stays in BoundEffects and produces a new Until in StagedEffects each time it matches — which then gets desugared on the next tick.

## Complete Example: Recurring chain with reversal

```ron
Until(trigger: PerfectBumped, then: [
    When(trigger: Impacted(Cell), then: [Do(Shockwave(base_range: 32.0, ...))]),
    Do(DamageBoost(2.0))
])
```

**Desugaring**:
1. Fire `DamageBoost(2.0)` on entity → damage boosted
2. Push `When(Impacted(Cell), Do(Shockwave(...)))` to entity's **BoundEffects** → shockwave fires on every cell hit
3. Remove Until, replace with:
   ```
   When(PerfectBumped, [Reverse(
       effects: [DamageBoost(2.0)],
       chains: [When(Impacted(Cell), Do(Shockwave(...)))]
   )])
   ```
   This goes into StagedEffects.

**While active**: every cell hit fires a shockwave (recurring, from BoundEffects).

**PerfectBumped fires**:
1. `When(PerfectBumped)` matches in StagedEffects → Reverse executes:
   - `commands.reverse_effect(entity, DamageBoost(2.0))` → removes damage boost
   - Finds and removes `When(Impacted(Cell), ...)` from BoundEffects → no more shockwaves
2. Entry consumed from StagedEffects

## Complete Example: Overclock (Until inside a When)

```ron
On(target: Bolt, then: [
    When(trigger: PerfectBumped, then: [
        Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
    ])
])
```

1. **Dispatch**: pushes `When(PerfectBumped, [Until(...)])` to bolt's BoundEffects
2. **PerfectBumped fires**: When matches in BoundEffects → child is Until → pushed to bolt's StagedEffects
3. **Until desugaring system runs**: finds the Until in StagedEffects → desugars:
   - Fire SpeedBoost(1.3) → bolt speeds up
   - Replace with `When(TimeExpires(2.0), [Reverse(effects: [SpeedBoost(1.3)], chains: [])])`
4. **2 seconds pass**: timer system → When(TimeExpires) matches → Reverse → speed boost removed
5. **BoundEffects entry stays** — next perfect bump produces a new Until → desugars again → new timed buff
