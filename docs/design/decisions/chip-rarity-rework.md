# Chip Rarity Rework

**Decision**: Design Rare first as the baseline, then derive weaker Common/Uncommon variants. Rare opens synergy paths that lower rarities don't.

## Design-from-Rare

Rare is the "intended" version of every chip — the version that fully realizes the design intent. Common and Uncommon are deliberately weakened derivatives:

| Rarity | Design Role | Power Budget |
|--------|------------|-------------|
| **Common** | Functional but limited — introduces the concept | Lower values, fewer effects, no synergy hooks |
| **Uncommon** | Meaningful upgrade — noticeable improvement | Moderate values, may add a secondary effect |
| **Rare** | Full realization — the strongest chip tier. Opens synergy paths; some chips are Rare-only. | Full values, includes effects that interact with other chips |

## Synergy Gating by Rarity

Rare versions include effects that create synergy hooks which lower rarities lack. This makes Rare genuinely exciting to find, not just "more damage":

**Example — Piercing:**
- Common: `On(target: Bolt, then: [Do(Piercing(1))])` — bolt passes through 1 cell
- Uncommon: `On(target: Bolt, then: [Do(Piercing(2))])` — bolt passes through 2 cells
- Rare: `On(target: Bolt, then: [Do(Piercing(3)), Do(DamageBoost(1.1))])` — bolt passes through 3 cells AND gets a damage bonus, which amplifies other damage-scaling effects

The Rare version interacts with the DamageBoost ecosystem. The Common/Uncommon versions are useful but don't unlock that synergy path.

## Rare-Only Chips

Some chips are too powerful or mechanically distinct to scale across tiers. These are Rare-only with `max_taken: 1`:
- Always `max_taken: 1` — no stacking
- Template has only the `rare:` slot filled (Common/Uncommon: None)
- Power comes from unique mechanics, not raw stats

## Evolution Distinction

Evolutions are the pinnacle of the build system — above even Rare chips in power and visual treatment.

| | Rare | Evolution |
|---|---|---|
| **Acquisition** | Random chip offering (weighted by rarity) | Combine two maxed chips at boss node |
| **Power level** | Strong build enabler | Run-defining power spike |
| **Stacking** | Varies by chip | Single instance, replaces ingredients |
| **Build impact** | Opens synergy paths | Transforms the run |
| **VFX** | Normal chip VFX | Screen-readable spectacle moment |

## Rationale

- **Design efficiency** — designing Rare first gives a clear target; weakening is easier than strengthening
- **Discovery reward** — finding Rare feels meaningful because it unlocks new interactions
- **Build depth** — players learn "I need the Rare version for my synergy to work" which drives run-to-run strategy
- **Power curve** — Common→Rare is a smooth power curve within a single chip concept; Evolutions sit above as a distinct power tier
