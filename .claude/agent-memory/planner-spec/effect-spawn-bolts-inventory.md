---
name: effect-spawn-bolts-inventory
description: SpawnBolts effect structure, fire/reverse signatures, test layout, and key patterns
type: project
---

## SpawnBolts Effect — `breaker-game/src/effect/effects/spawn_bolts/`

### File Structure
- `mod.rs` — routing only: `pub(crate) use effect::*; mod effect; #[cfg(test)] mod tests;`
- `effect.rs` — `fire()`, `reverse()` (const no-op), `register()` (const no-op)
- `tests/mod.rs` — `mod fire_config; mod fire_inherit; mod fire_lifespan; mod fire_spawning; mod reverse;`

### fire() Signature
```rust
pub(crate) fn fire(entity: Entity, count: u32, lifespan: Option<f32>, inherit: bool, _source_chip: &str, world: &mut World)
```

### Key Helpers Used
- `super::super::entity_position(world, entity)` — from `fire_helpers.rs`, returns Position2D or Vec2::ZERO
- `Bolt::builder().at_position(...).config(...).extra().spawn(world)` — **`spawn_extra_bolt` was removed** in builder migration; each effect site calls the builder directly

### Test Pattern
- Each test file defines `fn world_with_bolt_config() -> World` inserting `BoltConfig::default()` + `GameRng::default()`
- Import: `use super::super::effect::*;` to access `fire()` and `reverse()`
- Query spawned bolts: `world.query_filtered::<Entity, (With<Bolt>, With<ExtraBolt>)>()`
- Primary bolt = entity with `Bolt` but without `ExtraBolt`

### BoundEffects
- `BoundEffects(Vec<(String, EffectNode)>)` — Component, Debug, Default, Clone
- Located in `effect/core/types/definitions/enums.rs`
