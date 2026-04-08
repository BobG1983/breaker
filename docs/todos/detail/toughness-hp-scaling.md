# Toughness + HP Scaling

## Summary
Replace per-cell-type HP with a Toughness enum and `hp_for(toughness, tier, node_index)` scaling formula. Cells get HP from their toughness level + contextual scaling, not from cell RON definitions.

## Context
Currently each cell type RON defines its own HP value (Standard=10, Lock=10, Regen=20, Tough=30). This doesn't scale — HP needs to increase with tier and node progression. The cell builder todo ships with `.hp(value)` directly. This todo adds the Toughness abstraction on top so node generation can specify toughness (weak/standard/tough) and get appropriately scaled HP.

Designed in [cell-modifiers.md](cell-builder-pattern/cell-modifiers.md) under "HP Model" and "Toughness Enum."

## Decisions

### Base HP Values

| Toughness | Base HP |
|-----------|---------|
| Weak | 10.0 |
| Standard | 20.0 |
| Tough | 30.0 |

### Scaling Formula

**Exponential ramp:** `hp = base * 1.2^tier * (1.0 + 0.05 * node_index)`

- **Tier multiplier**: 1.2x per tier (exponential). Gentle early, steep late — standard roguelite curve.
- **Node multiplier**: +5% per node index within a tier (linear within-tier ramp).
- **Growth**: ~4.3x from tier 0 to tier 8.

### Sample Values

```
Tier 0, Node 0: Weak=10, Std=20, Tough=30
Tier 0, Node 4: Weak=12, Std=24, Tough=36
Tier 3, Node 0: Weak=17, Std=35, Tough=52
Tier 3, Node 4: Weak=21, Std=42, Tough=63
Tier 8, Node 0: Weak=43, Std=86, Tough=129
```

### Implementation

```rust
enum Toughness {
    Weak,      // base 10.0
    Standard,  // base 20.0
    Tough,     // base 30.0
}

fn hp_for(toughness: Toughness, tier: u32, node_index: u32) -> f32 {
    let base = match toughness {
        Toughness::Weak => 10.0,
        Toughness::Standard => 20.0,
        Toughness::Tough => 30.0,
    };
    base * 1.2_f32.powi(tier as i32) * (1.0 + 0.05 * node_index as f32)
}
```

### Builder Integration

```rust
Cell::builder()
    .position(pos)
    .dimensions(w, h)
    .toughness(Toughness::Tough)
    .tier_hp(tier, node_index)  // computes HP from toughness + formula
    .rendered(mesh, material)
    .spawn(commands);
```

- `.toughness()` stores the toughness level
- `.tier_hp(tier, node_index)` computes and sets HP using `hp_for()`
- `.hp(value)` still works as a direct override (for tests or special cases)
- Calling `.hp()` after `.tier_hp()` overrides the computed value

## Scope
- In: `Toughness` enum (Weak, Standard, Tough) with base HP values
- In: `hp_for(toughness, tier, node_index) -> f32` function with exponential tier scaling
- In: Builder integration: `.toughness()` + `.tier_hp(tier, node_index)`
- In: Remove raw HP from cell type definitions (replaced by toughness at generation time)
- In: Store base values and multipliers in a resource (tunable via RON config)
- Out: Cell builder typestate (already done in builder todo)
- Out: Node sequencing / skeleton integration (separate todo)

## Dependencies
- Depends on: Cell builder pattern (provides the builder API to extend)
- Blocks: Node sequencing refactor (needs toughness to generate cells)

## Notes
- "Tough" replaces the old `tough.cell.ron` — it's just a higher toughness on a standard cell
- Base values and multipliers should live in a config resource so they can be tuned without recompiling
- The 1.2x tier multiplier and 0.05 node multiplier are starting points — expect playtesting adjustments
- Consider whether chip damage upgrades need to scale similarly to avoid trivialization at high tiers

## Status
`ready`
