# Systems to Create

## Death pipeline systems (owned by death pipeline)

- [apply-damage-cell.md](apply-damage-cell.md) — Process DamageDealt\<Cell\>, decrement Hp, set KilledBy
- [apply-damage-bolt.md](apply-damage-bolt.md) — Process DamageDealt\<Bolt\>, decrement Hp, set KilledBy
- [apply-damage-wall.md](apply-damage-wall.md) — Process DamageDealt\<Wall\>, decrement Hp, set KilledBy
- [apply-damage-breaker.md](apply-damage-breaker.md) — Process DamageDealt\<Breaker\>, decrement Hp, set KilledBy
- [detect-cell-deaths.md](detect-cell-deaths.md) — Detect Hp ≤ 0 on cells, send KillYourself\<Cell\>
- [detect-bolt-deaths.md](detect-bolt-deaths.md) — Detect Hp ≤ 0 on bolts, send KillYourself\<Bolt\>
- [detect-wall-deaths.md](detect-wall-deaths.md) — Detect Hp ≤ 0 on walls, send KillYourself\<Wall\>
- [detect-breaker-deaths.md](detect-breaker-deaths.md) — Detect Hp ≤ 0 on breakers, send KillYourself\<Breaker\>
- [process-despawn-requests.md](process-despawn-requests.md) — Despawn entities from DespawnEntity messages

## Death bridge systems (moved to effect domain)

`bridge_destroyed<T>` systems dispatch death triggers (Died, Killed, DeathOccurred) and call `walk_effects`. They are effect bridges registered by the EffectPlugin, not the death pipeline.

See [effect-refactor/migration/new-trigger-implementations/death/](../../../effect-refactor/migration/new-trigger-implementations/death/index.md).
