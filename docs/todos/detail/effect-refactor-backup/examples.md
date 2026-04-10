# Effect System Examples

Builder and RON side-by-side for every pattern.

## 1. Simple passive

```rust
EffectDef::stamp(Bolt)
    .fire(DamageBoost { multiplier: 3.0 })?;
```
```ron
Stamp(Bolt, Fire(DamageBoost(multiplier: 3.0)))
```

## 2. Triggered effect

```rust
EffectDef::stamp(Bolt)
    .when(Impacted(Wall))
    .fire(Shockwave { base_range: 64.0, speed: 500.0 })?;
```
```ron
Stamp(Bolt, When(Impacted(Wall), Fire(Shockwave(base_range: 64.0, speed: 500.0))))
```

## 3. Nested triggers

```rust
EffectDef::stamp(Bolt)
    .when(PerfectBumped)
    .when(Impacted(Cell))
    .fire(ChainBolt { tether_distance: 120.0 })?;
```
```ron
Stamp(Bolt, When(PerfectBumped, When(Impacted(Cell), Fire(ChainBolt(tether_distance: 120.0)))))
```

## 4. Until (timed speed boost, reversed when timer fires)

```rust
EffectDef::stamp(Bolt)
    .when(PerfectBumped)
    .until(TimeExpires(3.0))
    .fire(SpeedBoost { multiplier: 1.5 })?;  // must be Reversible
```
```ron
Stamp(Bolt, When(PerfectBumped, Until(TimeExpires(3.0), Fire(SpeedBoost(multiplier: 1.5)))))
```

## 5. During (node-scoped)

```rust
EffectDef::stamp(EveryBolt)
    .during(NodeActive)
    .fire(SpeedBoost { multiplier: 1.3 })?;  // must be Reversible
```
```ron
Stamp(EveryBolt, During(NodeActive, Fire(SpeedBoost(multiplier: 1.3))))
```

## 5b. During — shield-scoped

```rust
// "While shield is up, bolts deal double damage"
EffectDef::stamp(EveryBolt)
    .during(ShieldActive)
    .fire(DamageBoost { multiplier: 2.0 })?;
```
```ron
Stamp(EveryBolt, During(ShieldActive, Fire(DamageBoost(multiplier: 2.0))))
```

## 5c. During — combo-scoped

```rust
// "After 3 consecutive perfect bumps, speed boost until streak breaks"
EffectDef::stamp(EveryBolt)
    .during(ComboActive(3))
    .fire(SpeedBoost { multiplier: 1.5 })?;
```
```ron
Stamp(EveryBolt, During(ComboActive(3), Fire(SpeedBoost(multiplier: 1.5))))
```

## 6. Route — one-shot (powder keg)

```rust
EffectDef::stamp(Bolt)
    .when(Impacted(Cell))
    .on(ImpactTarget::Impactee)
    .route(
        EffectTree::when(Died)
            .fire(Explode { range: 48.0, damage: 10.0 })?
    )?;
```
```ron
Stamp(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Route(
    When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
))))
```

Cell dies -> explode fires -> entry consumed. Hit the cell again to re-arm.

## 7. Stamp — permanent (hypothetical "cursed" cell)

```rust
EffectDef::stamp(Bolt)
    .when(Impacted(Cell))
    .on(ImpactTarget::Impactee)
    .stamp(
        EffectTree::when(Died)
            .fire(Explode { range: 48.0, damage: 10.0 })?
    )?;
```
```ron
Stamp(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Stamp(
    When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
))))
```

Cell dies -> explode fires -> re-arms. Explodes on every death, forever.

## 8. Mixed targets (glass cannon)

```rust
EffectDef::stamp(Bolt)
    .fire(DamageBoost { multiplier: 3.0 })?;
EffectDef::stamp(Breaker)
    .when(BoltLostOccurred)
    .fire(LoseLife)?;
```
```ron
Stamp(Bolt, Fire(DamageBoost(multiplier: 3.0))),
Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife))),
```

## 9. Breaker routing to bolts (aegis)

```rust
EffectDef::stamp(Breaker)
    .when(BoltLostOccurred)
    .fire(LoseLife)?;
EffectDef::stamp(EveryBolt)
    .when(PerfectBumped)
    .fire(SpeedBoost { multiplier: 1.5 })?;
EffectDef::stamp(EveryBolt)
    .when(EarlyBumped)
    .fire(SpeedBoost { multiplier: 1.1 })?;
```
```ron
Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife))),
Stamp(EveryBolt, When(PerfectBumped, Fire(SpeedBoost(multiplier: 1.5)))),
Stamp(EveryBolt, When(EarlyBumped, Fire(SpeedBoost(multiplier: 1.1)))),
```

## 10. Spawned (fire-and-forget on new entities)

```rust
EffectDef::stamp(Bolt)
    .spawned(Bolt)
    .fire(Piercing { count: 3 })?;
```
```ron
Stamp(Bolt, Spawned(Bolt, Fire(Piercing(count: 3))))
```

## 11. Until + nested When (non-reversible OK)

```rust
// Explode is NOT Reversible, but nested When relaxes the constraint
EffectDef::stamp(Bolt)
    .until(Died)
    .when(PerfectBumped)
    .fire(Explode { range: 50.0, damage: 10.0 })?;
```
```ron
Stamp(Bolt, Until(Died, When(PerfectBumped, Fire(Explode(range: 50.0, damage: 10.0)))))
```

Reversal removes the `PerfectBumped` listener, not the individual explosions.

## 12. Kill attribution (local trigger)

```rust
EffectDef::stamp(Bolt)
    .when(Killed(Cell))
    .fire(SpeedBoost { multiplier: 1.3 })?;
```
```ron
Stamp(Bolt, When(Killed(Cell), Fire(SpeedBoost(multiplier: 1.3))))
```

"When **I** kill a cell" -- local trigger, fires on the killer (bolt).

## 13. Cross-entity participant redirect

```rust
// Effect on breaker, but fire on the bolt participant
EffectDef::stamp(Breaker)
    .when(PerfectBumped)
    .on(BumpTarget::Bolt)
    .fire(FlashStep)?;
```
```ron
Stamp(Breaker, When(PerfectBumped, On(BumpTarget::Bolt, Fire(FlashStep))))
```

## Participant Target Enums

Shared enums -- multiple triggers use the same participant type:

```rust
enum BumpTarget { Bolt, Breaker }
// Used by: PerfectBumped, EarlyBumped, LateBumped, Bumped,
//          PerfectBumpOccurred, BumpOccurred, BumpWhiffOccurred, NoBumpOccurred

enum ImpactTarget { Impactor, Impactee }
// Used by: Impacted, ImpactOccurred

enum DeathTarget { Victim, Killer }
// Used by: Died, Killed, DeathOccurred

enum BoltLostTarget { Bolt, Breaker }
// Used by: BoltLostOccurred
```

## Terminal Semantics

| Terminal | Destination | Permanence | Re-arms |
|---|---|---|---|
| `Fire(effect)` | Immediate | N/A | N/A |
| `Stamp(tree)` | BoundEffects | Permanent | Yes |
| `Route(tree)` | StagedEffects | One-shot | No -- consumed on match |
