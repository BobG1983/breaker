# Wave 12: Breaker Domain Migration (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Migrate breaker domain to the unified death pipeline.

## What to migrate
- Replace breaker life pool / lives with `Hp` component
- Add `KilledBy` to breaker builder
- Create breaker domain kill handler (transitions game state, does NOT despawn)

## RED phase — write failing tests

- Breaker with life_pool: Some(3) gets Hp(max: 3, current: 3) + KilledBy
- Breaker with life_pool: None does NOT get Hp or KilledBy
- apply_damage::<Breaker> decrements Hp on DamageDealt<Breaker>
- detect_breaker_deaths sends KillYourself<Breaker> when Hp ≤ 0
- Breaker kill handler: KillYourself<Breaker> → inserts Dead, transitions game state (run lost), sends Destroyed<Breaker> + DespawnEntity. Does NOT despawn breaker.
- Breaker life loss e2e: BoltLostOccurred → LoseLife → DamageDealt<Breaker> → Hp decrements. Repeat to 0 → kill handler fires.
- Infinite lives: BoltLostOccurred → LoseLife → DamageDealt<Breaker> → no Hp, nothing happens.
- Dead prevents double-processing

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
Replace life pool with Hp in builder, create kill handler.

## GREEN gate
All tests pass. All previous tests still pass.

## Docs to read
- `unified-death-pipeline/migration/systems-to-create/apply-damage-breaker.md`
- `unified-death-pipeline/migration/systems-to-create/detect-breaker-deaths.md`
- `unified-death-pipeline/rust-types/`
