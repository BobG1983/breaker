# Wave 10: Bolt Domain Migration (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Migrate bolt domain to the unified death pipeline.

## What to migrate
- Update `tick_bolt_lifespan`: RequestBoltDestroyed → KillYourself<Bolt>
- Update `bolt_lost`: RequestBoltDestroyed → KillYourself<Bolt>
- Update `BoltLost` message: add bolt + breaker fields
- Remove `RequestBoltDestroyed` message type
- Remove `cleanup_destroyed_bolts` system
- Create bolt domain kill handler
- Add `Hp` and `KilledBy` to bolt builder

## RED phase — write failing tests

- tick_bolt_lifespan sends KillYourself<Bolt> on expiry (not RequestBoltDestroyed), killer = None
- bolt_lost sends KillYourself<Bolt> and BoltLost { bolt, breaker } with fields populated
- Bolt kill handler: KillYourself<Bolt> → inserts Dead, removes from spatial index, sends Destroyed<Bolt> + DespawnEntity
- Bolt lifespan e2e: BoltLifespan(0.5), tick past expiry, full pipeline to despawn
- Bolt lost e2e: trigger bolt_lost, KillYourself → kill handler → Destroyed → despawn
- Dead prevents double-processing

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
Update lifespan/bolt_lost systems, remove old types, create kill handler, update builder.

## GREEN gate
All tests pass. All previous tests still pass.

## Docs to read
- `unified-death-pipeline/migration/systems-to-modify/tick-bolt-lifespan.md`
- `unified-death-pipeline/migration/systems-to-modify/bolt-lost.md`
- `unified-death-pipeline/migration/systems-to-remove.md` — cleanup_destroyed_bolts
- `unified-death-pipeline/migration/systems-to-create/apply-damage-bolt.md`
- `unified-death-pipeline/migration/systems-to-create/detect-bolt-deaths.md`
- `effect-refactor/migration/new-trigger-implementations/bolt-lost/types.md`
- `unified-death-pipeline/rust-types/`
