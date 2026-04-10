# Wave 7: Death Pipeline Systems (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Write tests for the unified death pipeline systems, then implement them.

## RED phase — write failing tests

### apply_damage::<T>
- Decrements Hp, sets KilledBy on killing blow, first kill wins
- Skips Dead entities, apply_damage::<Cell> skips Locked
- Multiple DamageDealt in same frame all processed

### detect_*_deaths
- Sends KillYourself<T> when Hp ≤ 0, skips Hp > 0, skips Dead
- Does NOT insert Dead, does NOT despawn

### process_despawn_requests
- Despawns from DespawnEntity, uses try_despawn, runs in PostFixedUpdate

### Dead integration
- Without<Dead> prevents double-processing across both apply_damage and detect_deaths

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
- apply_damage::<T> (generic, monomorphized per GameEntity)
- detect_cell/bolt/wall/breaker_deaths
- process_despawn_requests

## GREEN gate
All tests pass. Do NOT modify tests.

## Docs to read
- `unified-death-pipeline/rust-types/` — Hp, KilledBy, Dead, DamageDealt, KillYourself, Destroyed, DespawnEntity
- `unified-death-pipeline/migration/systems-to-create/` — all 9 system behavioral specs
- `unified-death-pipeline/migration/query-data-to-create/` — DamageTargetData, DeathDetectionData
- `unified-death-pipeline/migration/plugin-wiring/system-sets.md`
