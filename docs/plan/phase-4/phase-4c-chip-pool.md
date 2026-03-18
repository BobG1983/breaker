# Phase 4c: Chip Pool & Rarity

**Goal**: 16-20 functional chips across Amp/Augment/Overclock kinds and Common/Uncommon/Rare/Legendary rarity tiers.

**Wave**: 2 (after 4b) — parallel with 4d and 4e

## Dependencies

- 4b (Chip Effect System) — need the effect type enums and application mechanism

## Sub-Stages

This stage mixes system work (rarity, inventory tracking) with content creation (16-20 RON files) and design validation (synergies). Split into three sub-stages.

### 4c.1: Rarity Enum + ChipInventory Resource (Session 3)

**Domain**: chips/

Add `Rarity` enum to chip definitions:

```rust
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}
```

Rarity affects:
- **Offering weight** — how often the chip appears in offerings (configured in RON)
- **Stat scaling** — rarer chips have stronger per-stack values

Build `ChipInventory` resource to track the player's current build:
- Which chips are held and at what stack level
- Which chips have been maxed (for pool removal in 4f)
- Which chips have been seen in offerings (for weight decay in 4f)

Each chip has:
- `max_stacks: u32` — hard cap on how many times it can stack
- `effect_per_stack` — what one stack gives you
- Stacking is flat: stack N = N * effect_per_stack

**Delegatable**: Yes — writer-tests → writer-code, scoped to chips/ domain.

### 4c.2: Chip RON Authoring (Session 6)

**Domain**: assets/

Author 16-20 chip RON files distributed across:

| Kind | Common | Uncommon | Rare | Legendary |
|------|--------|----------|------|-----------|
| Amp | 2-3 | 1-2 | 1 | 1 |
| Augment | 2-3 | 1-2 | 1 | 1 |
| Overclock | 1-2 | 1-2 | 1 | 0-1 |

Specific chip designs happen during implementation — the plan defines the structure, not the content.

**Delegatable**: Partially — content authoring is manual, but RON parse validation can be delegated.

### 4c.3: Synergy Design Review (Session 6)

Use **guard-game-design** to validate synergy coverage.

At least 30% of the pool (5-6 chips) must have effects that **interact with other chips**, not just modify independent stats. See the [Synergy Design Principle](../../design/decisions/chip-stacking.md#synergy-design-principle).

Examples of synergistic chips (concrete designs TBD):
- An amp whose damage scales with total augment stacks held
- An augment that widens the bump timing window per amp type equipped
- An overclock that triggers on any chip stack threshold reached
- A chip that converts bolts-lost into a temporary damage buff

The pool should include at least 1-2 "build-around" chips per rarity tier — chips weak alone but powerful with specific other chips.

## Acceptance Criteria

1. 16-20 chip RON files exist and parse
2. Each chip has a rarity tier that affects its offering weight
3. Stacking works up to max_stacks, then the chip leaves the pool
4. ChipInventory accurately tracks the player's build throughout a run
5. All chip effects are functional (even if simple)
6. At least 5 chips have effects that reference or interact with other chips' effects
7. At least 1 "build-around" chip exists per rarity tier (Common through Legendary)
