# Examples

## Overclock — temporary speed boost on perfect bump

```ron
On(target: Bolt, then: [
    When(trigger: PerfectBumped, then: [
        Until(trigger: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
    ])
])
```

1. **Dispatch**: pushes `When(PerfectBumped, [Until(...)])` to bolt's BoundEffects
2. **PerfectBumped fires on bolt**: When matches → child is Until → desugar:
   - Fire SpeedBoost(1.3) on bolt → bolt speeds up
   - Replace Until in StagedEffects with `When(TimeExpires(2.0), [Reverse(effects: [SpeedBoost(1.3)], chains: [])])`
3. **2 seconds pass**: timer decrements → When(TimeExpires) matches → Reverse fires → removes SpeedBoost from ActiveSpeedBoosts
4. **BoundEffects entry stays** — next perfect bump re-triggers

## Recurring shockwave until perfect bump

```ron
On(target: Bolt, then: [
    Until(trigger: PerfectBumped, then: [
        When(trigger: Impacted(Cell), then: [Do(Shockwave(base_range: 32.0, ...))]),
        Do(DamageBoost(2.0))
    ])
])
```

1. **Dispatch**: pushes `Until(PerfectBumped, [...])` to bolt's BoundEffects
2. **First evaluation**: Until encountered → desugar:
   - Fire DamageBoost(2.0) → damage boosted
   - Push `When(Impacted(Cell), Do(Shockwave))` to bolt's **BoundEffects** (recurring)
   - Replace Until with `When(PerfectBumped, [Reverse(effects: [DamageBoost(2.0)], chains: [When(Impacted(Cell), ...)])])`
3. **Bolt hits cell**: Impacted(Cell) matches the When in BoundEffects → shockwave fires. Repeats on every cell hit.
4. **PerfectBumped fires**: When(PerfectBumped) matches in StagedEffects → Reverse executes:
   - Reverse DamageBoost → removes from ActiveDamageBoosts
   - Remove `When(Impacted(Cell), ...)` from BoundEffects → no more shockwaves

## Wall redirecting to bolt

```ron
On(target: Wall, then: [
    When(trigger: Impacted(Bolt), then: [
        On(target: Bolt, then: [Do(SpeedBoost(multiplier: 1.2))])
    ])
])
```

1. **Dispatch**: pushes When to wall's BoundEffects
2. **Bolt hits wall**: `Impacted(Bolt)` fires on wall → When matches → child is On(Bolt, ...)
3. **On resolves**: Bolt = the bolt from BoltImpactWall message → bare Do → `commands.fire_effect(bolt, SpeedBoost(1.2))`

## Cascade — shockwave on cell destruction

```ron
On(target: Bolt, then: [
    When(trigger: CellDestroyed, then: [Do(Shockwave(base_range: 20.0, ...))])
])
```

1. **Dispatch**: pushes When to bolt's BoundEffects
2. **CellDestroyed fires** (global): bolt's When matches → `commands.fire_effect(bolt, Shockwave(...))`

## Last Stand — breaker speed boost on bolt lost

```ron
On(target: Breaker, then: [
    When(trigger: BoltLost, then: [Do(SpeedBoost(multiplier: 1.15))])
])
```

1. **Dispatch**: pushes When to breaker's BoundEffects
2. **BoltLost fires** (global): breaker's When matches → SpeedBoost fires on breaker → queries breaker for speed components → boosts breaker speed (not bolt speed — effect acts on self)

## Impact chip — nested triggers with arming

```ron
On(target: Bolt, then: [
    When(trigger: PerfectBumped, then: [
        When(trigger: Impacted(Cell), then: [
            Do(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))
        ])
    ])
])
```

1. **Dispatch**: pushes outer When to bolt's BoundEffects
2. **PerfectBumped fires**: outer When matches → inner When is non-Do → pushed to bolt's StagedEffects
3. **Bolt hits cell**: `Impacted(Cell)` → StagedEffects entry matches → shockwave fires → entry consumed
4. **BoundEffects stays** — next perfect bump re-arms

## Passive piercing — chip dispatch

```ron
On(target: Bolt, then: [Do(Piercing(1))])
```

1. **Dispatch**: resolves Bolt → primary bolt. Child is bare Do → `commands.fire_effect(bolt, Piercing(1))`. No trigger needed.
