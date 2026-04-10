# System Sets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum DeathPipelineSystems {
    ApplyDamage,
    DetectDeaths,
}
```

## DeathPipelineSystems::ApplyDamage

Process damage messages, decrement Hp, set KilledBy. All run in FixedUpdate.

| System | Message consumed | Target |
|--------|-----------------|--------|
| `apply_damage::<Cell>` | `DamageDealt<Cell>` | Cell entities without `Locked` |
| `apply_damage::<Bolt>` | `DamageDealt<Bolt>` | Bolt entities |
| `apply_damage::<Wall>` | `DamageDealt<Wall>` | Wall entities |
| `apply_damage::<Breaker>` | `DamageDealt<Breaker>` | Breaker entities |

All four are the same generic system monomorphized per `GameEntity` type. Each reads its typed damage message, decrements Hp, and sets KilledBy on the killing blow. First kill wins — if KilledBy is already set, do not overwrite. All use `Without<Dead>` to skip entities already confirmed dead.

## DeathPipelineSystems::DetectDeaths

Detect Hp ≤ 0, send `KillYourself<T>`. All run in FixedUpdate, after ApplyDamage.

| System | Message produced | Query filter |
|--------|-----------------|--------------|
| `detect_cell_deaths` | `KillYourself<Cell>` | Cell entities `Without<Dead>` |
| `detect_bolt_deaths` | `KillYourself<Bolt>` | Bolt entities `Without<Dead>` |
| `detect_wall_deaths` | `KillYourself<Wall>` | Wall entities `Without<Dead>` |
| `detect_breaker_deaths` | `KillYourself<Breaker>` | Breaker entities `Without<Dead>` |

Each checks if Hp is ≤ 0 and sends KillYourself with victim entity and killer from KilledBy. The `Without<Dead>` filter skips entities already confirmed dead by their domain kill handler — prevents double-processing for entities killed directly (e.g., by the Die effect).

## process_despawn_requests

Not in a system set. Runs in **PostFixedUpdate** — after all FixedUpdate systems complete.

| System | Message consumed | Action |
|--------|-----------------|--------|
| `process_despawn_requests` | `DespawnEntity` | `commands.entity(msg.entity).try_despawn()` |

This is the ONLY system that despawns entities. Uses `try_despawn` because the entity may already be gone.

## Domain kill handlers (NOT in this plugin)

Between DetectDeaths and the effect system's death bridges, each domain has its own kill handler that:
1. Reads `KillYourself<T>`
2. Performs domain-specific cleanup (remove from spatial index, update counters, etc.)
3. Sends `Destroyed<T>` with victim, killer, and positions
4. Sends `DespawnEntity`

These are registered by their own domain plugins, not by the death pipeline plugin.
