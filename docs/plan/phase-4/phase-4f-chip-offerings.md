# Phase 4f: Chip Offering System

**Goal**: Weighted random chip offerings with rarity, pool depletion, and seeded determinism.

**Wave**: 3 (integration) — parallel with 4g. **Session 7.**

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

## Scenario Coverage

### New Invariants
- **`OfferingNoDuplicates`** — no two chips in a single offering share the same name. Checked when entering ChipSelect state.
- **`MaxedChipNeverOffered`** — chips at max stacks never appear in offerings. Checked when entering ChipSelect state.

### New Scenarios
- `mechanic/chip_offering_seeded.scenario.ron` — Fixed seed, scripted input that always selects the first chip. Verifies deterministic offerings and pool depletion over multiple nodes.
- `stress/chip_pool_exhaustion.scenario.ron` — Long run with scripted chip selection to intentionally max out multiple chips. Verifies pool doesn't empty to fewer than 3 (or handles gracefully), no crashes, `OfferingNoDuplicates` holds.

## Acceptance Criteria

1. Same seed + same player choices = same offerings every time
2. Rare chips appear less often than common chips
3. Maxed chips never appear in offerings
4. Seen-but-not-taken chips appear less often over time
5. No duplicate chips within a single 3-card offering
6. Rarity is visually distinguishable on the chip select screen
