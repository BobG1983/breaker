# Wave 9: Cell Domain Migration (RED → GREEN)

## Specs
- [spec-tests.md](spec-tests.md) — Behavioral test spec
- [spec-code.md](spec-code.md) — Implementation spec

## Goal
Migrate cell domain to the unified death pipeline.

## What to migrate
- Remove `handle_cell_hit` (damage + visual feedback combined)
- Extract visual feedback into a damage response system
- Remove `cleanup_cell`
- Create cell domain kill handler
- Add `Hp` and `KilledBy` to cell builder
- Update `bolt_cell_collision` to send `DamageDealt<Cell>`

## RED phase — write failing tests

- apply_damage::<Cell> with real cell entities (Hp decrements, Locked skipped)
- detect_cell_deaths sends KillYourself<Cell> when Hp ≤ 0
- Cell kill handler: KillYourself<Cell> → inserts Dead, removes from spatial index, updates RequiredToClear tracking, sends Destroyed<Cell> + DespawnEntity
- Cell damage e2e: Hp(3), three DamageDealt<Cell>, full pipeline to despawn
- Locked cell: DamageDealt silently dropped
- bolt_cell_collision sends DamageDealt<Cell>
- Visual feedback: damage response reads Hp changes for material update

## RED gate
All tests compile. All tests fail.

## GREEN phase — implement
Remove old systems, create kill handler, update builder, update collision system.

## GREEN gate
All tests pass. All previous tests still pass.

## Docs to read
- `unified-death-pipeline/migration/systems-to-remove.md` — handle_cell_hit, cleanup_cell
- `unified-death-pipeline/migration/systems-to-create/apply-damage-cell.md`
- `unified-death-pipeline/migration/systems-to-create/detect-cell-deaths.md`
- `unified-death-pipeline/rust-types/` — all death pipeline types
