# Phase 4h: Chip Evolution

**Goal**: Boss nodes offer evolution rewards. Two chips at minimum stacks + boss kill = evolved chip.

**Wave**: 4 (capstone) — parallel with 4i. **Session 8.**

## Dependencies

- 4c (Chip Pool) — need ingredient chips with stacking
- 4d (Trigger/Effect Architecture) — evolved overclocks use the trigger chain system
- 4e (Node Escalation) — boss nodes as the evolution trigger point

## What to Build

### Evolution Recipes

RON-defined recipes match `EvolutionRecipe` shape:

```ron
// assets/evolutions/cleaving_arc.evolution.ron
(
    ingredients: [
        (chip_name: "Piercing Shot", stacks_required: 2),
        (chip_name: "Wide Breaker", stacks_required: 2),
    ],
    result_definition: (
        name: "Cleaving Arc",
        description: "Wide piercing bolt that shatters cells on contact",
        rarity: Legendary,
        max_stacks: 1,
        effects: [Amp(Piercing(5))],
    ),
)
```

### Evolution Flow

1. Player beats a boss node
2. `generate_chip_offerings` checks `EvolutionRegistry.eligible_evolutions(&ChipInventory)` and injects `ChipOffering::Evolution` entries before normal chip slots
3. If the player qualifies for 1+ evolutions: evolution offerings appear first on the ChipSelect screen; remaining slots filled with normal chips
4. If the player qualifies for 0 evolutions: normal chip offerings fill all slots as usual
5. Confirming an evolution offering consumes ingredient stacks from `ChipInventory` and grants the evolved chip via the normal `ChipSelected` message flow

### Evolution Offering — Implementation Note

Evolution offerings are integrated directly into the existing **ChipSelect screen** via `ChipOffering::Evolution { ingredients, result }` rather than a separate reward screen. `handle_chip_input` pattern-matches on `ChipOffering::Evolution` to consume ingredient stacks before transitioning. This reuses the full chip-select infrastructure (timer, navigation, transition) without needing a new screen.

### Target: 3-4 Evolutions

Mixed types to prove the architecture handles variety:
- Amp + Amp -> evolved Amp
- Augment + Augment -> evolved Augment
- Amp + Overclock -> evolved Overclock (cross-kind)
- (Optional 4th depending on chip pool composition)

Specific recipes designed during implementation alongside the chip pool.

### Evolution Registry

- `EvolutionRegistry` resource in `chips/resources.rs` — stores recipes as a flat `Vec<EvolutionRecipe>`
- Provides `eligible_evolutions(&ChipInventory)` returning all recipes whose ingredient stacks are met
- RON data in `assets/evolutions/` (not yet authored — infrastructure in place, data pending)

## Scenario Coverage

### New Invariants
- **`EvolutionConsumesIngredients`** — after evolution, ingredient chips are removed from `ChipInventory` and the evolved chip is present. Described in plan but not yet implemented as a scenario runner invariant. Ingredient consumption is tested via unit tests in `handle_chip_input`.

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
