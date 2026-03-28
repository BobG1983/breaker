# Node Types

```rust
pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },
    Do(EffectKind),
    Once(Vec<EffectNode>),
    On { target: Target, permanent: bool, then: Vec<EffectNode> },
    Until { trigger: Trigger, then: Vec<EffectNode> },
    Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> },
}
```

## When

```rust
When { trigger: Trigger, then: Vec<EffectNode> }
```

Gate. If the current trigger matches, evaluate children. Otherwise skip.

- In **BoundEffects**: permanent — re-evaluates on every matching trigger.
- In **StagedEffects**: consumed when matched.

```ron
When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])
```

## Do

```rust
Do(EffectKind)
```

Terminal. Queues `commands.fire_effect(entity, effect)` — the effect fires on the entity whose chain is being evaluated. The effect handler receives the entity and `&mut World`, queries whatever components it needs.

```ron
Do(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))
```

## Once

```rust
Once(Vec<EffectNode>)
```

One-shot wrapper. Evaluate children against the current trigger. If any child matches, fire/arm it AND remove the Once from whichever component it's in. If nothing matches, keep it.

```ron
Once([When(trigger: BoltLost, then: [Do(SecondWind)])])
```

## On

```rust
On { target: Target, #[serde(default)] permanent: bool, then: Vec<EffectNode> }
```

Redirect. Resolves target to entity/entities from the trigger system's message context. **On never fires anything on the current entity** — it only transfers to other entities.

- Bare `Do` children → `commands.fire_effect(target_entity, effect)`
- Non-Do children → pushed to target entity's **StagedEffects** (default) or **BoundEffects** (if `permanent: true`)

The `permanent` flag (defaults to `false` via serde) controls where non-Do children land on the target:
- `permanent: false` (default) → StagedEffects (consumed on match — one-shot)
- `permanent: true` → BoundEffects (permanent — recurring)

At dispatch time, the flag is irrelevant — dispatch always pushes to BoundEffects regardless.

```ron
// One-shot: bolt gets speed boost on next cell hit only
On(target: Bolt, then: [When(trigger: Impacted(Cell), then: [Do(SpeedBoost(multiplier: 1.5))])])

// Permanent: bolt gets speed boost on every cell hit
On(target: Bolt, permanent: true, then: [When(trigger: Impacted(Cell), then: [Do(SpeedBoost(multiplier: 1.5))])])
```

## Until

```rust
Until { trigger: Trigger, then: Vec<EffectNode> }
```

Duration-scoped effects. "Apply these effects now, undo them when this trigger fires." Until is **not processed by trigger systems** — a dedicated desugaring system handles it.

See **[Until and Desugaring](until.md)** for the full explanation, desugaring mechanics, Reverse node, and detailed examples.

```ron
Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
```

## Reverse

```rust
Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> }
```

Not a RON-facing node — created internally by Until desugaring. Lives in StagedEffects inside a `When(trigger, [Reverse(...)])` wrapper.

See **[Until and Desugaring](until.md)** and **[Reversal](reversal.md)** for details.
