# Chip Offering System

**Decision**: Weighted random pool with rarity tiers, seeded determinism, and Isaac-style pool depletion.

## Model

- Chip offerings are drawn from a weighted pool using the run seed + node index
- Each chip has a rarity tier: Common, Uncommon, Rare
- Rarity affects both offering weight (how often it appears) AND stats (rarer = stronger per-stack)
- 3 chips offered per node (pick 1 of 3)
- No duplicate chips within a single offering
- Maxed chips leave the pool entirely
- Seen-but-not-taken chips have their weight reduced (Isaac-style decay)

## Rarity Tiers

| Tier | Offering Weight | Stat Scaling |
|------|----------------|--------------|
| Common | High | Baseline |
| Uncommon | Medium | Moderate boost |
| Rare | Low | Strong boost |

Exact weights are RON-configurable and will be tuned during playtesting.

## Rationale

- **Weighted random** creates variety while preserving agency (Pillar 6: RNG Shapes, Skill Decides)
- **Rarity affects stats** makes rare chips feel meaningfully better, not just scarcer
- **Pool depletion** rewards commitment and creates run-over-run variety
- **Weight decay** on seen chips rewards decisive play (Pillar 5: Pressure, Not Panic)
