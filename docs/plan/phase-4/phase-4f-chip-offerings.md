# Phase 4f: Chip Offering System

**Goal**: Weighted random chip offerings with rarity, pool depletion, and seeded determinism.

## Dependencies

- 4a (Seeded RNG) — offerings must be deterministic
- 4c (Chip Pool & Rarity) — need the chip pool with rarity tiers

## What to Build

### Offering Generation

Replace the current "first 3 from registry" logic with:

1. Build the active pool: all chips minus maxed-out chips
2. Apply weight decay to seen-but-not-taken chips
3. Draw 3 chips from the pool using seeded RNG + rarity weights
4. No duplicates within a single offering (re-roll if collision)

### Rarity Weights

Base offering weights per rarity (RON-configurable):

| Rarity | Base Weight |
|--------|------------|
| Common | 100 |
| Uncommon | 50 |
| Rare | 15 |
| Legendary | 3 |

### Pool Depletion

Two depletion mechanisms:

1. **Max stack removal**: When a chip hits max stacks, it is permanently removed from the pool for the rest of the run
2. **Weight decay**: When a chip is offered but not taken, its weight is reduced by a configurable factor (e.g., 0.8x). This accumulates across multiple sightings.

Both mechanisms are tracked in `ChipInventory`.

### Integration with ChipSelect Screen

- `spawn_chip_select` calls the offering system instead of reading the registry directly
- Offerings are generated deterministically from `GameRng`
- The chip select screen displays rarity visually (card border color/glow per rarity tier)

## Acceptance Criteria

1. Same seed + same player choices = same offerings every time
2. Rare chips appear less often than common chips
3. Maxed chips never appear in offerings
4. Seen-but-not-taken chips appear less often over time
5. No duplicate chips within a single 3-card offering
6. Rarity is visually distinguishable on the chip select screen
