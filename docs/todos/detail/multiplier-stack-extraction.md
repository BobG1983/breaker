# Extract `MultiplierStack` to deduplicate `DamageBoostStack` / `VulnerableStack`

## Summary
After source-filter support landed in `rantzsoft_dmg`, `DamageBoostStack` and `VulnerableStack` are now ~95% identical (~120 lines each, ~240 lines total duplication). Extract a crate-private generic backing type and make the two components newtype wrappers.

## Context
Surfaced by `/simplify` Code Reuse review during the source-filter-feature simplify pass. Pre-feature the duplication was ~30 lines per stack and acceptable. Post-feature both stacks share:

- `PersistentEntry` — identical (source, multiplier, filter)
- `OneShotEntry` — identical (multiplier, filter)
- `add`, `add_filtered`, `add_one_shot`, `add_one_shot_filtered`, `remove_by_source`, `aggregate_persistent`, `aggregate_and_consume_one_shots`, `aggregate_one_shots`, `is_empty` — identical bodies (modulo trivial binding-name differences)
- Doc comments have already started to drift (one says "filterless entries always apply" while the other says "per `entry_applies`") — exactly the early-divergence smell.

## Design (proposed)

Crate-private `MultiplierStack` in `rantzsoft_dmg/src/components/multiplier_stack/` with all eight methods + `PersistentEntry` / `OneShotEntry` types. Two `Component` newtypes (or struct-with-field) at the existing locations:

```rust
#[derive(Component, Debug, Default)]
pub struct DamageBoostStack(pub(crate) MultiplierStack);

#[derive(Component, Debug, Default)]
pub struct VulnerableStack(pub(crate) MultiplierStack);
```

Each newtype implements the public methods by delegation. ECS queries still discriminate dealer-side (`&mut DamageBoostStack`) vs target-side (`&mut VulnerableStack`). Negative-Clone contract preserved per-wrapper.

Alternative: a single trait with default methods + tag types. Decide during planning.

## Scope
- **In:** `rantzsoft_dmg/` only. Extract `MultiplierStack`, refactor both stack components to delegate, update internal call sites in `apply_damage_boosts` / `apply_vulnerable` / `preview.rs` (no public-API surface change). Tests stay where they are.
- **Out:** Game-side migration of `BurnoutDamageBoost` / `RiskyDamageBoost` / `DebtCashOut` markers (see `2026-04-27-damage-boost-source-filter.md` notes).

## Dependencies
- **Depends on:** todo #1 (source-filter feature) — now landed.
- **Blocks:** none directly. Defers a maintenance burden but no current consumer is affected.

## Notes
- The two stacks ARE distinct ECS components — preserve that. Don't merge them into one `Component`.
- `pub(crate)` on the inner field is fine; tests in sibling modules access the inner shape.
- After extraction, the ~240-line duplication becomes ~6 lines per wrapper plus the shared module.

## Status
`ready` — design captured, scope clear, no open questions.
