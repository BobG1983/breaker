# Legendary Rarity Removal + Retuning

## Summary
Delete the `Legendary` rarity. Retune 8 legendary chips as Rare (6 with new Common/Uncommon tiers, 2 Rare-only). Cut 3 legendaries entirely. Promote 2 legendaries (Deadline, Ricochet Protocol) to protocols. Remove Anchor evolution.

## Final Chip Roster

### Desperation — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | bolt-lost → SpeedBoost(1.2) |
| Uncommon | Keen | bolt-lost → SpeedBoost(1.3) |
| Rare | | bolt-lost → SpeedBoost(1.5) + SpawnBolts(count: 1, inherit: true) |

Bolt spawn is the Rare payoff. Lower tiers just get speed on bolt-lost.

### Singularity — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | SizeBoost(0.85) + DamageBoost(1.3) + SpeedBoost(1.1) |
| Uncommon | Keen | SizeBoost(0.8) + DamageBoost(1.5) + SpeedBoost(1.2) |
| Rare | | SizeBoost(0.7) + DamageBoost(2.0) + SpeedBoost(1.3) + Piercing(1) |

Progressive shrink with progressive power. "Small fast hard-hitter" fantasy scales well. Rare Piercing rider opens the cell-drill build (stacks with Piercing Shot).

### Parry — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | PerfectBump → Shield(duration: 1.5, reflection_cost: 0.5) |
| Uncommon | Keen | PerfectBump → Shield(duration: 2.0, reflection_cost: 0.5) |
| Rare | | PerfectBump → Shield(duration: 3.0, reflection_cost: 0.5) + pushes a staged Once(Impacted(Cell) → Shockwave(range: 32, speed: 400)) onto the bumped bolt |

Progressive shield duration. Rare also routes a one-shot shockwave-on-next-cell-impact to the bumped bolt — perfect bump becomes defensive AND offensive.

### Powder Keg — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | cell death → Explode(range: 24, damage: 4) |
| Uncommon | Keen | cell death → Explode(range: 28, damage: 6) |
| Rare | | cell death → Explode(range: 36, damage: 8) |

Progressive range and damage. Explosions at every tier, bigger as you go.

### Death Lightning — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | cell death → ChainLightning(arcs: 1, range: 32, damage_mult: 0.5) |
| Uncommon | Keen | cell death → ChainLightning(arcs: 1, range: 40, damage_mult: 0.7) |
| Rare | | cell death → ChainLightning(arcs: 2, range: 48, damage_mult: 0.8) |

Single arc at lower tiers. Rare gets 2 arcs. Progressive chain reach.

### Tempo — 3 tiers, max_taken: 3

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Common | Minor | Bumped → Until(BumpWhiff) → SpeedBoost(1.08) |
| Uncommon | Keen | Bumped → Until(BumpWhiff) → SpeedBoost(1.12) |
| Rare | | Bumped → Until(BumpWhiff) → SpeedBoost(1.2) + DamageBoost(1.15) |

Progressive speed reward. Rare combo also applies a damage multiplier — whiffing loses both speed AND damage, raising the skill ceiling.

### Glass Cannon — Rare only, max_taken: 1

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Rare | | DamageBoost(3.0) + bolt-lost → LoseLife |

Risk/reward tradeoff doesn't scale well at lower tiers. Rare-only. Not stackable — double life loss is suicide. Damage multiplier kept at 3.0 so the life-loss penalty actually feels justified.

### Chain Reaction — Rare only, max_taken: 1

| Rarity | Prefix | Effects |
|--------|--------|---------|
| Rare | | cell destroyed → cell destroyed → SpawnBolts(lifespan: None) |

Recursive chain identity can't be simplified. Rare-only. Not stackable — recursive spawning is already exponential. No lifespan — trust the double-trigger structure to keep the chain scoped.

## Chips Cut (3)

| Chip | Reason |
|------|--------|
| **Whiplash** | Whiffs are too easy to deliberately trigger. Rewards intentionally playing badly. |
| **Gauntlet** | Big bolt is a weird identity. Doesn't synergize well with anything. |
| **Feedback Loop** | Killed. |

## Chips Promoted to Protocols (2)

| Chip | Action |
|------|--------|
| **Deadline** | Remove `legendary:` slot. Create `assets/protocols/deadline.protocol.ron`. |
| **Ricochet Protocol** | Remove `legendary:` slot. Create `assets/protocols/ricochet_protocol.protocol.ron`. |

## Evolution Removal (1)

| Evolution | Action |
|-----------|--------|
| **Anchor** | Delete `anchor.evolution.ron`. Create `assets/protocols/anchor.protocol.ron`. |

## Code Changes

1. Remove `Legendary` variant from `Rarity` enum
2. Remove `rarity_weight_legendary` from `ChipSelectConfig` + `defaults.chipselect.ron`
3. For the 6 multi-rarity chips: replace the `legendary:` slot with `common:`, `uncommon:`, `rare:` slots using the values above
4. For the 2 Rare-only chips: replace `legendary:` with `rare:`
5. Delete `whiplash.chip.ron`, `gauntlet.chip.ron`, `feedback_loop.chip.ron` entirely
6. Remove Legendary color config entries
7. Delete `anchor.evolution.ron`, remove Anchor recipe
8. Update any tests referencing `Rarity::Legendary`
9. Update `max_taken` for the 6 multi-rarity chips from 1 to 3

## Stacking Model

All 6 multi-rarity chips use **both-fire-independently** stacking (standard roguelite model). Each stack installs its own `BoundEffects` entry. When the trigger fires, ALL matching entries fire. This is how the chip system already works — no new mechanics needed.

Examples:
- **Common + Rare Powder Keg**: cell death fires TWO explosions (24/4 AND 36/8). AoEs overlap.
- **Common + Uncommon + Rare Tempo**: three SpeedBoost entries. 1.08 × 1.12 × 1.2 = 1.45x combined. One whiff kills all three.
- **Common + Rare Death Lightning**: cell death fires TWO chain lightnings (1 arc/32 AND 2 arcs/48). Arcs fork in different directions.

### Visual stacking notes (FX-layer, not gameplay)

| Effect type | Visual concern | Solution |
|-------------|---------------|----------|
| Explosion overlap | Two rings at same position | FX layer staggers by 0.05s for chain-pop feel, or merges into one larger ring |
| Chain Lightning overlap | Two arc chains from same cell | Arcs naturally fork in different directions — looks great |
| Shield overlap | Two shields on same breaker | Shields stack duration-wise (one expires, other continues). Visual: single shield with brighter glow |
| SpawnBolts overlap | Two bolts at same position/velocity | Add slight random angular spread on spawned bolt direction |

These are FX polish items for Phase 5, not gameplay design changes.

## Prefix Convention
All 6 multi-rarity chips use the standard prefix pattern: Common = "Minor", Uncommon = "Keen", Rare = "" (empty). These can be revised later for flavor but match the existing chip template convention.

## Protocol Migration — Preserve RON Values

When this todo is extracted and queued before the protocol/hazard todo, the Deadline and Ricochet Protocol legendary RON values must be preserved for protocol migration. Record them here:

### Deadline (legendary → protocol)
```ron
// From deadline.chip.ron legendary: slot
effects: [
    On(target: Bolt, then: [
        When(trigger: NodeTimerThreshold(0.25), then: [
            Until(trigger: NodeEnd, then: [
                Do(SpeedBoost(multiplier: 2.0)),
                Do(DamageBoost(2.0)),
            ]),
        ]),
    ]),
]
```

### Ricochet Protocol (legendary → protocol)
```ron
// From ricochet_protocol.chip.ron legendary: slot
effects: [
    On(target: Bolt, then: [
        When(trigger: Impacted(Wall), then: [
            Until(trigger: Impacted(Cell), then: [
                Do(DamageBoost(3.0)),
            ]),
        ]),
    ]),
]
```

These RON trees will be migrated to the new effect system's `ValidDef` format when the protocol RON files are created (after effect refactor completes).

## Status
`ready`
