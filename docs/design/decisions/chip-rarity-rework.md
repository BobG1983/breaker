# Chip Rarity Rework

**Decision**: Design Rare first as the baseline, then derive weaker Common/Uncommon variants. Rare opens synergy paths that lower rarities don't.

## Design-from-Rare

Rare is the "intended" version of every chip — the version that fully realizes the design intent. Common and Uncommon are deliberately weakened derivatives:

| Rarity | Design Role | Power Budget |
|--------|------------|-------------|
| **Common** | Functional but limited — introduces the concept | Lower values, fewer effects, no synergy hooks |
| **Uncommon** | Meaningful upgrade — noticeable improvement | Moderate values, may add a secondary effect |
| **Rare** | Full realization — opens synergy paths | Full values, includes effects that interact with other chips |
| **Legendary** | Niche build-around — max:1, conditional power | Unique mechanics, not strictly better than evolutions |

## Synergy Gating by Rarity

Rare versions include effects that create synergy hooks which lower rarities lack. This makes Rare genuinely exciting to find, not just "more damage":

**Example — Piercing:**
- Common: `On(target: Bolt, then: [Do(Piercing(1))])` — bolt passes through 1 cell
- Uncommon: `On(target: Bolt, then: [Do(Piercing(2))])` — bolt passes through 2 cells
- Rare: `On(target: Bolt, then: [Do(Piercing(3)), Do(DamageBoost(1.1))])` — bolt passes through 3 cells AND gets a damage bonus, which amplifies other damage-scaling effects

The Rare version interacts with the DamageBoost ecosystem. The Common/Uncommon versions are useful but don't unlock that synergy path.

## Legendary Rules

- Always `max_taken: 1` — no stacking
- Template has only the `legendary` slot filled (Common/Uncommon/Rare: None)
- Must be conditional or niche — never "strictly better than Rare"
- Power comes from unique mechanics, not raw stats
- Should not be more powerful than evolutions — evolutions are the pinnacle of the build system
- Each Legendary suggests a specific build direction without mandating it

## Evolution Distinction

| | Legendary | Evolution |
|---|---|---|
| **Acquisition** | Random chip offering (weighted by rarity) | Combine two maxed chips at boss node |
| **Power level** | Niche build-around | Run-defining power spike |
| **Stacking** | max:1, no stacking | Single instance, replaces ingredients |
| **Build impact** | Suggests a direction | Transforms the run |
| **VFX** | Normal chip VFX | Screen-readable spectacle moment |

## Rationale

- **Design efficiency** — designing Rare first gives a clear target; weakening is easier than strengthening
- **Discovery reward** — finding Rare feels meaningful because it unlocks new interactions
- **Build depth** — players learn "I need the Rare version for my synergy to work" which drives run-to-run strategy
- **Power curve** — Common→Rare is a smooth power curve within a single chip concept; Legendaries and Evolutions sit above it as distinct power tiers
