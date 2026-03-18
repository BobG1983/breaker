# Phase 4h: Chip Evolution

**Goal**: Boss nodes offer evolution rewards. Two chips at minimum stacks + boss kill = evolved chip.

**Wave**: 4 (capstone) — parallel with 4i. **Session 8.**

## Dependencies

- 4c (Chip Pool) — need ingredient chips with stacking
- 4d (Trigger/Effect Architecture) — evolved overclocks use the trigger chain system
- 4e (Node Escalation) — boss nodes as the evolution trigger point

## What to Build

### Evolution Recipes

RON-defined recipes:

```ron
// assets/evolutions/cleaving_arc.evolution.ron
(
    name: "Cleaving Arc",
    description: "Wide piercing bolt that shatters cells on contact",
    ingredients: [
        (chip: "Piercing Shot", min_stacks: 2),
        (chip: "Wide Breaker", min_stacks: 2),
    ],
    result: (
        kind: Amp,
        rarity: Legendary,
        effect: /* evolved effect — stronger than either ingredient */
    ),
)
```

### Evolution Flow

1. Player beats a boss node
2. System checks `ChipInventory` against all evolution recipes
3. If the player qualifies for 1+ evolutions: present evolution choice screen
4. If the player qualifies for 0 evolutions: offer alternative boss reward (chips, stats, etc.)
5. Evolving consumes both ingredient chips from inventory and adds the evolved chip

### Evolution Reward Screen

- Similar to chip select but shows evolution options
- Displays the recipe (which two chips combine)
- Shows the resulting evolved chip with its effect
- No timer — boss rewards are a brief respite (but still fast UI)

### Target: 3-4 Evolutions

Mixed types to prove the architecture handles variety:
- Amp + Amp -> evolved Amp
- Augment + Augment -> evolved Augment
- Amp + Overclock -> evolved Overclock (cross-kind)
- (Optional 4th depending on chip pool composition)

Specific recipes designed during implementation alongside the chip pool.

### Evolution Registry

- Loaded from RON files in `assets/evolutions/`
- Indexed by ingredient pair for fast lookup
- Hot-reloadable

## Scenario Coverage

### New Invariants
- **`EvolutionConsumesIngredients`** — after evolution, both ingredient chips are removed from `ChipInventory` and the evolved chip is present. Checked on state transition out of evolution reward screen.

### New Scenarios
- `mechanic/evolution_boss_reward.scenario.ron` — Scripted run that accumulates specific chips to min stacks, then clears a boss. Verifies evolution is offered, ingredients consumed, evolved chip applied.
- `stress/evolution_with_effects.scenario.ron` — Chaos input after evolution. Verifies evolved chip effects work under stress, no entity leaks, no NaN from evolved effect values.

## Acceptance Criteria

1. Beating a boss with qualifying chips offers evolution
2. Evolving consumes ingredients and grants the evolved chip
3. Evolved chips are visibly stronger than their ingredients
4. Cross-kind evolution works (Amp + Overclock -> evolved)
5. Non-qualifying boss kills offer alternative rewards
6. 3-4 evolutions exist and are functional
