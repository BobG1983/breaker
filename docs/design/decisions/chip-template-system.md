# Chip Template System

**Decision**: Chip content is authored as templates — one file per chip concept with per-rarity slots. The loader expands templates into individual `ChipDefinition`s at load time.

## Template Format

Each template defines the chip concept (name, max_taken) and up to four rarity slots:

```ron
(
    name: "Piercing Shot",
    max_taken: 3,
    common: (prefix: "Basic", effects: [On(target: Bolt, then: [Do(Piercing(1))])]),
    uncommon: (prefix: "Keen", effects: [On(target: Bolt, then: [Do(Piercing(2))])]),
    rare: (prefix: "Brutal", effects: [On(target: Bolt, then: [Do(Piercing(3)), Do(DamageBoost(1.1))])]),
)
```

Each slot is an optional `RaritySlot` (slots with no entry are omitted entirely in RON). `RaritySlot` has:
- `prefix: String` — adjective prepended to the template name (e.g., "Basic Piercing Shot"). Empty string means no prefix.
- `effects: Vec<RootEffect>` — full effect list for that rarity (no magic scaling, no inheritance). `RootEffect::On { target, then }` is the top-level wrapper ensuring all effects name a target entity before trigger dispatch.

## Structural Guarantees

- **`max_taken` is shared across all rarities.** Taking "Basic Piercing" and "Brutal Piercing" both count toward the same cap. The template is the unit of max enforcement, not the individual chip.
- **No duplicate templates in a single offering.** The offering system prevents showing two rarities of the same template concept in one chip selection screen.
- **Full effect list per slot.** Each rarity explicitly lists all its effects. No implicit inheritance from lower rarities — what you see in the RON is what you get.

## Loader Expansion

At load time, the template loader reads each template file and generates N `ChipDefinition`s (one per non-None slot):
- `name`: `"{prefix} {name}"` (or just `"{name}"` if prefix is empty)
- `rarity`: corresponding tier
- `max_taken`: from the template
- `effects`: from the slot
- `template_name`: back-reference to the template for shared max enforcement

## Naming Convention

Adjective prefixes per rarity tier:

| Rarity | Style | Examples |
|--------|-------|---------|
| Common | Weak/basic adjective | "Basic Piercing", "Minor Damage Boost" |
| Uncommon | Moderate adjective | "Keen Piercing", "Potent Damage Boost" |
| Rare | Strong/aggressive adjective | "Brutal Piercing", "Savage Damage Boost" |
| Legendary | Unique proper name | "Ricochet Protocol", "Glass Cannon" |

Legendaries use unique names rather than prefixed template names — they are standalone build-arounds, not rarity tiers of a common concept.

## Rationale

- **Structural max_taken enforcement** — impossible for an author to accidentally give different rarities different caps
- **Authoring convenience** — one file per concept, all variants visible together
- **No runtime overhead** — templates expand to flat `ChipDefinition`s at load time
- **Evolution recipes remain separate** — evolutions have their own format (recipe-based, not rarity-tiered)
