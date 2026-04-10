# Types

No new types defined here. Death bridges consume types from the unified death pipeline.

## Types consumed

- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }` — message sent by domain kill handlers after confirming a kill. See [unified-death-pipeline/rust-types/destroyed.md](../../../../unified-death-pipeline/rust-types/destroyed.md).
- `GameEntity` trait — impl'd on Bolt, Cell, Wall, Breaker. See [unified-death-pipeline/rust-types/game-entity.md](../../../../unified-death-pipeline/rust-types/game-entity.md).

## Triggers dispatched

- `Died` — Local, on victim only
- `Killed(EntityKind)` — Local, on killer only (skipped when killer is None)
- `DeathOccurred(EntityKind)` — Global, on all entities

See [dispatching-triggers/death/](../../../dispatching-triggers/death/) for per-trigger behavioral specs.
