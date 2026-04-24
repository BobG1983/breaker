# Eliminate duplicated pierce-decision damage math

## Problem

`breaker-game/src/bolt/systems/bolt_cell_collision/system.rs` computes the fully-multiplied damage value `base × boost × vuln` LOCALLY — twice:

1. `effective_damage` (outer loop, line ~166): `base * DamageBoostStack::aggregate_persistent`
2. `cell_damage` (in `resolve_bolt_cell_hit`, line ~284): `effective_damage * VulnerableStack::aggregate_persistent`

The purpose is to drive `would_destroy` — the pierce/reflect decision must answer "would this hit kill the target?" synchronously inside the CCD sweep, before the `rantzsoft_dmg` pipeline runs.

The `rantzsoft_dmg` pipeline ALSO multiplies the same factors independently in `apply_damage_boosts::<Cell>` and `apply_vulnerable::<Cell>`, via `aggregate_and_consume_one_shots`. Two concrete problems:

1. **Drift risk.** If the pipeline formula changes (e.g., adds a third multiplier, reorders operations, handles stack saturation differently), the pierce decision silently diverges.
2. **One-shot semantic asymmetry.** `aggregate_persistent` excludes one-shot boosts/vulnerabilities — the pipeline consumes them. So a one-shot `DamageBoost` that WOULD let the bolt kill the cell (and pierce) is applied to delivered damage but ignored by the pierce decision. The bolt stops instead of piercing, even though the kill happens. This is currently undocumented design.

## Proposed fix

Expose a pure helper from `rantzsoft_dmg`:

```rust
pub fn preview_damage<T: Dmgable>(
    base: f32,
    boosts: Option<&DamageBoostStack>,
    vuln: Option<&VulnerableStack>,
    mode: PreviewMode, // Persistent | IncludeOneShot
) -> f32 { ... }
```

- One source of truth for the multiplier chain.
- `mode` makes the one-shot semantic explicit at the call site — `bolt_cell_collision` picks `Persistent` and the reason is visible.
- The applicators (`apply_damage_boosts::<T>`, `apply_vulnerable::<T>`) call the same helper internally with `IncludeOneShot`.

## Scope

- New `preview_damage` function in `rantzsoft_dmg` (probably in `components/mod.rs` or a new `preview.rs`).
- `bolt_cell_collision::resolve_bolt_cell_hit` calls it instead of doing the multiplication inline.
- `apply_damage_boosts::<T>` / `apply_vulnerable::<T>` refactored to call it internally (or equivalent — the key is ONE implementation).
- Tests pin the mode semantics: `Persistent` vs `IncludeOneShot` produce different results when one-shots are present.

## Non-goals

- Don't change the pierce decision's game behavior — `Persistent` stays the mode. A separate TODO (see "piercing model revisit") decides whether that's right.
- Don't touch `PiercingBeamConfig::fire` — it has its own duplication problem handled by the `Fireable::fire` refactor (port-to-rantzsoft-dmg.md W9).

## Verification

- Existing `bolt_cell_collision` tests pass unchanged.
- New tests in `rantzsoft_dmg` for `preview_damage` covering both modes.
- Grep for other `aggregate_persistent` / `aggregate_and_consume_one_shots` callsites — ensure none of them duplicate the formula outside the crate.

## Risk

Low. The refactor is structural, not behavioral. The one-shot semantic change is explicitly NOT happening in this TODO — it's deferred to piercing-model-revisit.
