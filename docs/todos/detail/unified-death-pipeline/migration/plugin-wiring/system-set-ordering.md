# System Set Ordering

## Within the death pipeline

```
DeathPipelineSystems::ApplyDamage
    ↓ (after)
DeathPipelineSystems::DetectDeaths
```

- **ApplyDamage before DetectDeaths**: Damage must be applied and Hp decremented before death detection checks for Hp ≤ 0. Without this ordering, detect systems would see stale Hp values.

`process_despawn_requests` runs in PostFixedUpdate — no ordering relationship with the FixedUpdate sets. It runs after everything else in the frame.

## Parallelism within sets

### ApplyDamage set
All four `apply_damage::<T>` systems CAN run in parallel — they read different typed messages and query different entity populations (Cell vs Bolt vs Wall vs Breaker). No shared mutable state.

### DetectDeaths set
All four `detect_*_deaths` systems CAN run in parallel — they query different entity populations and send different typed messages.

## External dependencies

### Must run AFTER (systems that produce damage messages)

| External system | Produces | Death pipeline consumer |
|----------------|----------|------------------------|
| Bolt collision systems | `DamageDealt<Cell>`, `DamageDealt<Wall>` | `apply_damage::<Cell>`, `apply_damage::<Wall>` |
| Breaker collision systems | `DamageDealt<Cell>` | `apply_damage::<Cell>` |
| Shockwave damage system | `DamageDealt<Cell>` | `apply_damage::<Cell>` |
| Chain lightning system | `DamageDealt<Cell>` | `apply_damage::<Cell>` |
| Tether beam damage system | `DamageDealt<Cell>` | `apply_damage::<Cell>` |
| Any effect that sends `DamageDealt<T>` | `DamageDealt<T>` | `apply_damage::<T>` |

The ApplyDamage set must run after ALL systems that produce `DamageDealt<T>` messages. In practice, this means after `EffectV3Systems::Tick` (which contains shockwave, chain lightning, tether beam damage systems).

### Must run BEFORE (systems that consume death results)

| Death pipeline output | External consumer |
|----------------------|-------------------|
| `KillYourself<T>` | Domain kill handlers (per-domain plugins) |
| `Destroyed<T>` (from domain kill handlers) | `EffectV3Systems::Bridge` (death bridges: `on_destroyed::<T>`) |
| `DespawnEntity` (from domain kill handlers) | `process_despawn_requests` (PostFixedUpdate) |

### Full frame ordering

```
Game systems (collision, bump grading, etc.)
    ↓
EffectV3Systems::Bridge (trigger dispatch, effect firing)
    ↓
EffectV3Systems::Tick (shockwave, chain lightning, tether beam produce DamageDealt)
    ↓
EffectV3Systems::Conditions (condition polling)
    ↓
DeathPipelineSystems::ApplyDamage (process DamageDealt, decrement Hp)
    ↓
DeathPipelineSystems::DetectDeaths (detect Hp ≤ 0, send KillYourself)
    ↓
Domain kill handlers (cleanup, send Destroyed + DespawnEntity)
    ↓
PostFixedUpdate: process_despawn_requests (despawn entities)
```

Note: `DamageDealt<T>` messages sent by effect bridges (e.g., LoseLife firing DamageDealt) are processed by ApplyDamage in the same frame because ApplyDamage runs after Bridge. `DamageDealt<T>` messages sent by tick systems (shockwave, chain lightning, tether beam) are also processed in the same frame because ApplyDamage runs after Tick. Cascade damage (damage dealt by death triggers) is processed next frame — the death bridge fires in Bridge, which runs before ApplyDamage, so any DamageDealt sent by a death-triggered effect is picked up next frame's ApplyDamage. This one-frame delay is acceptable at 60fps.
