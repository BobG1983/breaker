# Toughness + HP Scaling

## Summary
Replace per-cell-type HP with a Toughness enum and `hp_for(toughness, tier, node_index)` scaling formula. Cells get HP from their toughness level + contextual scaling, not from cell RON definitions.

## Context
Currently each cell type RON defines its own HP value. This doesn't scale — HP needs to increase with tier and node progression. The cell builder todo ships with `.hp(value)` directly. This todo adds the Toughness abstraction on top so node generation can specify toughness (weak/standard/tough) and get appropriately scaled HP.

Designed in [cell-modifiers.md](cell-builder-pattern/cell-modifiers.md) under "HP Model" and "Toughness Enum."

## Scope
- In: `Toughness` enum (Weak, Standard, Tough) with base HP values
- In: `hp_for(toughness, tier, node_index) -> f32` function with tier + node multipliers
- In: Builder integration: `.toughness(Toughness::Tough).tier_hp(tier, node_index)` sets HP
- In: Remove raw HP from cell type definitions (replaced by toughness at generation time)
- Out: Cell builder typestate (already done in builder todo)
- Out: Node sequencing / skeleton integration (separate todo)

## Dependencies
- Depends on: Cell builder pattern (provides the builder API to extend)
- Blocks: Node sequencing refactor (needs toughness to generate cells)

## Notes
- Base HP values for Weak/Standard/Tough need to be decided
- Tier multiplier formula needs to be decided (e.g., +50% per tier? exponential?)
- Node-within-tier multiplier needs to be decided (e.g., +10% per node index?)
- "Tough" replaces the old `tough.cell.ron` — it's just a higher toughness on a standard cell

## Status
`[NEEDS DETAIL]` — base values and scaling formula not decided
