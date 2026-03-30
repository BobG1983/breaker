---
name: Evolution RON structure and ingredient-coherence audit
description: How evolution RON files are structured, which files exist, which are missing, and ingredient-behavior coherence ratings
type: project
---

## RON Structure Pattern

Evolution RON files live at `breaker-game/assets/chips/evolution/*.evolution.ron`.

Each file declares: `name`, `description`, `effects` (EffectNode tree), `ingredients` (chip requirements).

Effect triggers follow a nested On/When/Do pattern:
- `On(target: Bolt/Breaker, then: [When(trigger: ..., then: [Do(...)])])`
- Common triggers: PerfectBumped, Bumped, CellDestroyed, Impacted(Cell), BoltLost, Bump

Common effect actions: Shockwave, ChainLightning, SpawnBolts, GravityWell, SpeedBoost, SpawnPhantom, SecondWind, EntropyEngine, TetherBeam, PiercingBeam, ChainBolt

## RON Files Present (11 total)

nova_lance, voltchain, phantom_breaker, supernova, dead_mans_hand, gravity_well, second_wind, entropy_engine, split_decision, arcwelder, railgun

## RON Files Missing (design doc entries with no file)

- chain_reaction.evolution.ron — design doc specifies Cascade x3 + Splinter x2 + Piercing x3
- feedback_loop.evolution.ron — design doc has TBD ingredients; cannot create until designed

## Ingredient Coherence Ratings (from audit)

| Evolution | Rating | Key reason |
|---|---|---|
| Entropy Engine | GOOD | Cascade → trigger domain, Flux → random pool mechanic |
| Nova Lance | WEAK | Both ingredients are passive stat stacks; no precision or shockwave hint |
| Voltchain | GOOD | Chain Reaction chip gives chain-destruction domain |
| Phantom Breaker | WEAK | Both are Breaker stat buffs; no mirroring/phantom concept |
| Supernova | GOOD | Surge → PerfectBump trigger, Piercing → pierce-then-explode |
| Dead Man's Hand | GOOD | Last Stand → BoltLost trigger, Damage Boost → power-to-explosion |
| Railgun | GOOD | Piercing + Speed directly evoke railgun fantasy |
| Gravity Well | GOOD | Magnetism → attraction domain, direct escalation |
| Second Wind | WEAK | Wide Breaker + Breaker Speed are pure mobility; no death-prevention concept |
| Split Decision | GOOD | Splinter → spawn domain, Piercing → inherited bolts threat |
| Arcwelder | GOOD | Tether → bolt-to-bolt connection domain, direct escalation |

## Weak Pairings Summary

Three evolutions have weak ingredient coherence:
1. **Nova Lance**: needs ingredients that hint at precision shooting and explosive impact (e.g., Surge + Cascade)
2. **Phantom Breaker**: needs a "reflection/echo/clone" chip concept that doesn't exist yet
3. **Second Wind**: needs a defensive or resilience chip; Last Stand would be a natural fit here instead of Wide Breaker

## All Ingredient Templates Verified Present

All 15 unique ingredient chip names referenced in the 11 evolution files have matching `.chip.ron` templates. No missing template files.
