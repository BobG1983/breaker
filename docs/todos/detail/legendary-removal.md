# Legendary Rarity Removal

## Summary
Remove the `Legendary` rarity entirely. Retune 8 surviving legendaries as multi-tier chips (Common/Uncommon/Rare). Cut 3 legendaries. Promote 2 to protocols (Deadline, Ricochet Protocol). Remove Anchor evolution. Must complete before the protocol & hazard system todo.

## Context
Legendary rarity is being removed as part of the protocol/hazard system design. Protocols replace legendaries as the "exciting, rare, run-altering" option. The 13 chips with legendary slots are either retuned as Rare chips (some with new Common/Uncommon tiers), cut entirely, or promoted to protocols. This todo is extracted from the protocol & hazard system design so it can be completed first — the protocol todo assumes Legendary is already gone.

## Design (complete)
Full chip-by-chip decisions, rarity tiers, stacking model, and code changes are documented in:

**[mod-system-design/legendary-retuning.md](mod-system-design/legendary-retuning.md)**

That file is the source of truth for all values, cuts, and promotions.

## What's Decided
- 8 chips survive: Glass Cannon, Desperation, Singularity, Chain Reaction, Parry, Powder Keg, Death Lightning, Tempo
- 6 of 8 get Common/Uncommon/Rare tiers with max_taken: 3
- 2 of 8 are Rare-only with max_taken: 1 (Glass Cannon, Chain Reaction)
- 3 chips cut: Whiplash (whiffs too easy to game), Gauntlet (weird identity), Feedback Loop (killed)
- 2 chips promoted to protocols: Deadline, Ricochet Protocol (RON values preserved in protocol detail files)
- 1 evolution removed: Anchor (promoted to protocol, RON values preserved in anchor.md)
- Stacking model: both-fire-independently (standard roguelite accumulation)

## Stacking Visual Behavior (resolved)

**Decision**: Both-fire-independently is the correct gameplay model. Visual overlaps are solved at the FX layer. Gameplay effects don't fire until the FX starts (e.g., shockwave damage begins when the ring starts expanding, not before the stagger delay).

| Effect | Stacking behavior |
|--------|------------------|
| **Explode** (Powder Keg) | Each explosion staggers in time (0.05s between) AND position (offset slightly from tile center, spread based on stack count). Damage waits for FX — each explosion's damage starts when its ring begins expanding. Looks like a rapid-fire chain of pops radiating outward from the death. |
| **ChainLightning** (Death Lightning) | Force divergence — 2nd chain excludes cells already hit by the 1st. Arcs spread to different targets. Needs exclusion tracking (pass a `HashSet<Entity>` of already-hit cells to the 2nd chain). |
| **SpawnBolts** (Chain Reaction, Desperation) | Angular spread (±5°) AND slightly offset spawn positions so you can visually see two distinct bolts being born. Both spawn on the same frame. |
| **Shield** (Parry) | Additive duration — stacking adds durations into one shield (e.g., Common 1.5s + Rare 3.0s = 4.5s single shield). Single timer, single shield entity. Visual glow can scale with total duration. |
| **SpeedBoost** (Tempo, Singularity) | Invisible — multiplicative stacking. No visual concern. Both boosts apply immediately. |
| **DamageBoost** (Glass Cannon, Singularity) | Invisible — multiplicative stacking. No visual concern. Both boosts apply immediately. |
| **Shockwave** (if any chip uses it) | Same as Explode — stagger in time + offset position. Damage waits for FX. |

## Dependencies
- Depends on: nothing (can start immediately)
- Blocks: protocol & hazard system todo (assumes Legendary is already gone)

## Status
`ready`
