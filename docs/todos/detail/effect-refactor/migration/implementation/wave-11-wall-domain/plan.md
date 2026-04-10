# Wave 11: Wall Domain Migration (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Migrate wall domain to the unified death pipeline.

## What to migrate
- Create wall domain kill handler
- Add `Hp` and `KilledBy` to destructible wall builder (permanent walls unaffected)

## RED phase — write failing tests

- Wall kill handler: KillYourself<Wall> → inserts Dead, removes from spatial index, sends Destroyed<Wall> + DespawnEntity
- Destructible wall e2e: Hp(1), DamageDealt<Wall>, full pipeline to despawn
- Permanent wall: no Hp, not matched by apply_damage or detect_wall_deaths
- Shield wall: effect-spawned wall with Hp works through death pipeline

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
Create kill handler, update destructible wall builder.

## GREEN gate
All tests pass. All previous tests still pass.

## Docs to read
- `unified-death-pipeline/migration/systems-to-create/apply-damage-wall.md`
- `unified-death-pipeline/migration/systems-to-create/detect-wall-deaths.md`
- `unified-death-pipeline/rust-types/`
