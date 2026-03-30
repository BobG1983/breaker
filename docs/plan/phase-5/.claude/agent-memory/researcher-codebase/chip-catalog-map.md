---
name: chip-catalog-map
description: Complete chip template and evolution inventory with evolution path coverage and known name/effect mismatches. Audited from actual RON files.
type: project
---

## Chip Templates (22 total, from breaker-game/assets/chips/templates/)

| Template name | File | Primary EffectKind | Has evolution path? |
|---|---|---|---|
| Splinter | splinter.chip.ron | SpawnBolts + SizeBoost on CellDestroyed | YES (Split Decision) |
| Piercing Shot | piercing.chip.ron | Piercing | YES (Supernova, Split Decision, Railgun) |
| Surge | surge.chip.ron | SpeedBoost on PerfectBumped | YES (Supernova) |
| Glass Cannon | glass_cannon.chip.ron | DamageBoost + SizeBoost | NO (legendary, intentional) |
| Damage Boost | damage_boost.chip.ron | DamageBoost | YES (Nova Lance, Dead Man's Hand, Voltchain, Arcwelder) |
| Quick Stop | quick_stop.chip.ron | QuickStop on Breaker | NO (no evolution yet) |
| Bump Force | bump_force.chip.ron | BumpForce on Breaker | YES (Phantom Breaker) |
| Pulse | pulse.chip.ron | Pulse on PerfectBumped/Bumped | NO (no evolution yet) |
| Wide Breaker | wide_breaker.chip.ron | SizeBoost on Breaker | YES (Phantom Breaker, Second Wind) |
| Ricochet Protocol | ricochet_protocol.chip.ron | DamageBoost Until on Impacted(Wall) | NO (legendary, intentional) |
| Bolt Speed | bolt_speed.chip.ron | SpeedBoost on Bolt | YES (Nova Lance, Railgun) |
| Tether | tether.chip.ron | ChainBolt on PerfectBumped+Impacted(Cell) | YES (Arcwelder) |
| Cascade | cascade.chip.ron | Shockwave on CellDestroyed | YES (Entropy Engine) |
| Flux | flux.chip.ron | RandomEffect on Bump (Breaker) | YES (Entropy Engine) |
| Last Stand | last_stand.chip.ron | SpeedBoost on BoltLost (Breaker) | YES (Dead Man's Hand) |
| Chain Reaction | chain_reaction.chip.ron | SpawnBolts on nested CellDestroyed | YES (Voltchain) |
| Bolt Size | bolt_size.chip.ron | SizeBoost on Bolt | YES (Gravity Well) |
| Magnetism | magnetism.chip.ron | Attraction(Cell) on Bolt | YES (Gravity Well) |
| Breaker Speed | breaker_speed.chip.ron | SpeedBoost on Breaker | YES (Second Wind) |
| Aftershock | aftershock.chip.ron | Shockwave on Impacted(Wall) | NO |
| Reflex | reflex.chip.ron | SpawnBolts on PerfectBump (Breaker) | NO |
| Feedback Loop | feedback_loop.chip.ron | SpeedBoost Until(TimeExpires(3.0)) on triple-condition chain | NO (legendary; planned evolution but no RON) |

## Evolution Templates (11 total, from breaker-game/assets/chips/evolution/)

| Evolution name | File | Ingredients | Effect |
|---|---|---|---|
| Nova Lance | nova_lance.evolution.ron | Damage Boost×2, Bolt Speed×2 | Shockwave (NOTE: should be PiercingBeam per 5t plan) |
| Supernova | supernova.evolution.ron | Piercing Shot×3, Surge×1 | SpawnBolts+Shockwave on CellDestroyed chain |
| Dead Man's Hand | dead_mans_hand.evolution.ron | Damage Boost×3, Last Stand×1 | Shockwave+SpeedBoost on BoltLost |
| Split Decision | split_decision.evolution.ron | Splinter×2, Piercing Shot×2 | SpawnBolts(inherit:true) on CellDestroyed |
| Entropy Engine | entropy_engine.evolution.ron | Cascade×2, Flux×2 | EntropyEngine on CellDestroyed |
| Phantom Breaker | phantom_breaker.evolution.ron | Wide Breaker×2, Bump Force×2 | SpawnPhantom(duration:5, max:1) on Bump |
| Voltchain | voltchain.evolution.ron | Chain Reaction×1, Damage Boost×2 | ChainLightning(arcs:3, range:96, mult:0.5) on CellDestroyed |
| Arcwelder | arcwelder.evolution.ron | Tether×2, Damage Boost×1 | TetherBeam(mult:1.5) on Bumped |
| Second Wind | second_wind.evolution.ron | Wide Breaker×3, Breaker Speed×3 | SecondWind on BoltLost |
| Gravity Well | gravity_well.evolution.ron | Bolt Size×2, Magnetism×2 | GravityWell(str:500,dur:3,rad:128,max:2) on CellDestroyed |
| Railgun | railgun.evolution.ron | Piercing Shot×3, Bolt Speed×4 | PiercingBeam(mult:3.0,width:30) on PerfectBumped |

## Key Known Issues (from audit)

1. **Nova Lance RON mismatch**: effect is currently `Shockwave`, design intent is `PiercingBeam`. Phase 5t explicitly flags this.
2. **Feedback Loop name collision**: legendary chip template name = planned evolution name. If evolution is ever authored as `name: "Feedback Loop"`, it overwrites the legendary in ChipCatalog.
3. **Last Stand effect mismatch**: name implies survival/defensive, effect is SpeedBoost on Breaker.
4. **RampingDamage, LoseLife, TimePenalty, Explode**: fully implemented EffectKinds with NO chip template using them.
5. **Shield**: no chip template uses Shield — only Aegis breaker definition uses it.
6. **ChipTemplate has no description field**: all non-evolution chips have empty description in ChipDefinition.

**Why:** Documented during chip coverage audit task. Load-bearing for future chip content authoring and Phase 5 VFX wiring.
**How to apply:** When writing specs for new chip content or Phase 7 content wave, reference this map to avoid the collision issue, find chips without evolution paths, and flag the Nova Lance RON update as a prerequisite.
