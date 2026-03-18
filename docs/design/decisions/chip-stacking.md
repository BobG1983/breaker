# Chip Stacking

**Decision**: Hybrid model — per-chip caps with flat integer stacks.

## Model

- Each chip defines `max_stacks` in RON (e.g., Piercing Shot max_stacks: 3)
- Each stack adds the same flat bonus (stack 1 = +1 pierce, stack 2 = +2 pierce, stack 3 = +3 pierce)
- When a chip hits max stacks, it **leaves the offering pool**
- Pool depletion uses Isaac-style weight decay: seeing a chip in offerings (even if not taken) reduces its weight

## Rationale

- **Flat stacking** is visceral and easy to communicate. No mental math for diminishing returns.
- **Per-chip caps** prevent degenerate edge cases while preserving the "Build the Broken Build" pillar.
- **Pool depletion** rewards commitment: maxing a chip narrows the pool, surfacing rarer options. The pool naturally evolves through a run.
- **Weight decay on seen chips** rewards decisive play and punishes browsing — aligned with the "Pressure, Not Panic" pillar.

## Power Ceiling

**Breakable but rare.** A committed player who understands synergies can build something that outpaces the difficulty curve. But "broken" runs require knowledge + commitment, not luck. Rarity controls how often god-tier builds happen — most runs are competitive but beatable.

## Synergy Design Principle

**At least 30% of chips must interact with other chips' effects.** Independent stat mods create *different* runs; synergistic interactions create *surprising* runs. The difference is the core of Pillar 9 (Every Run Tells a Story).

Types of cross-chip interaction:
- **Scaling chips**: effect grows based on other chips held (e.g., "bonus damage per unique chip kind")
- **Trigger amplifiers**: amps/augments that feed overclock triggers (e.g., "piercing counts as an impact for trigger purposes")
- **Conditional chips**: effect activates only when another chip type is present (e.g., "if you have 3+ amp stacks, breaker gains a speed burst on bump")
- **Synergy combos**: two specific chips together produce an effect neither has alone (lighter than evolution — no boss kill required)

The goal: players should discover interactions on run 50 they didn't know about on run 10. The wiki should document chip synergies. Community knowledge-sharing is a longevity multiplier.

## Research Context

Informed by analysis of Hades (diminishing Pom returns), Slay the Spire (unlimited card duplicates + deck dilution), Binding of Isaac (pool depletion + weight decay), Vampire Survivors (fixed slots + evolution), and Balatro (unique items + slot scarcity). See the stacking research for full comparison.
