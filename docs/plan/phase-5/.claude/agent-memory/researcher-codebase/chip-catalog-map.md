---
name: chip-catalog-map
description: Complete chip template and evolution inventory with evolution path coverage and known name/effect mismatches. Audited from actual RON files.
type: project
---

## Chip Templates (33 total confirmed, from breaker-game/assets/chips/templates/)

| Template name | File | Primary EffectKind | Has evolution path? |
|---|---|---|---|
| Splinter | splinter.chip.ron | SpawnBolts + SizeBoost on CellDestroyed | YES (Split Decision) |
| Piercing Shot | piercing.chip.ron | Piercing | YES (Supernova, Split Decision, Railgun) |
| Surge | surge.chip.ron | SpeedBoost on PerfectBumped | YES (Supernova) |
| Glass Cannon | glass_cannon.chip.ron | DamageBoost + SizeBoost | NO (legendary, intentional) |
| Damage Boost | damage_boost.chip.ron | DamageBoost | YES (Nova Lance, Dead Man's Hand, Voltchain, Arcwelder) |
| Quick Stop | quick_stop.chip.ron | QuickStop on Breaker | NO (FlashStep evolution designed but no RON) |
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

## Evolution Templates (17 total as of feature/scenario-coverage, from breaker-game/assets/chips/evolution/)

| Evolution name | File | Ingredients | Effect | Status |
|---|---|---|---|---|
| Nova Lance | nova_lance.evolution.ron | Damage Boost×2, Bolt Speed×3 | PiercingBeam(mult:2.5,width:40) on PerfectBumped+Impacted(Cell) | VALID |
| Supernova | supernova.evolution.ron | Piercing Shot×3, Surge×1 | SpawnBolts+Shockwave on CellDestroyed chain | VALID |
| Dead Man's Hand | dead_mans_hand.evolution.ron | Damage Boost×3, Last Stand×1 | Shockwave+SpeedBoost on BoltLost | VALID |
| Split Decision | split_decision.evolution.ron | Splinter×2, Piercing Shot×2 | SpawnBolts(inherit:true) on CellDestroyed | VALID |
| Entropy Engine | entropy_engine.evolution.ron | Cascade×2, Flux×2 | EntropyEngine on CellDestroyed | VALID |
| Phantom Breaker | phantom_breaker.evolution.ron | Wide Breaker×2, Bump Force×2 | SpawnPhantom(duration:5, max:1) on Bump | VALID |
| Voltchain | voltchain.evolution.ron | Chain Reaction×1, Damage Boost×2 | ChainLightning(arcs:3, range:96, mult:0.5) on CellDestroyed | VALID |
| Arcwelder | arcwelder.evolution.ron | Tether×2, Damage Boost×1 | TetherBeam(mult:1.5) on Bumped | VALID |
| Second Wind | second_wind.evolution.ron | Wide Breaker×3, Breaker Speed×3 | SecondWind on BoltLost | VALID |
| Gravity Well | gravity_well.evolution.ron | Bolt Size×2, Magnetism×2 | GravityWell(str:500,dur:3,rad:128,max:2) on CellDestroyed | VALID |
| Railgun | railgun.evolution.ron | Piercing Shot×3, Bolt Speed×4 | PiercingBeam(mult:3.0,width:30) on PerfectBumped | VALID |
| Circuit Breaker | circuit_breaker.evolution.ron | Feedback Loop×1, Bump Force×2 | SpawnBolts+Shockwave on PerfectBumped | VALID |
| Resonance Cascade | resonance_cascade.evolution.ron | Pulse×2, Bolt Size×2 | Pulse(passive) on Bolt | VALID |
| Mirror Protocol | mirror_protocol.evolution.ron | Reflex×1, Piercing Shot×2 | SpawnBolts(count:2,inherit:true) on PerfectBump (Breaker) | VALID |
| Anchor | anchor.evolution.ron | Quick Stop×2, Bump Force×2 | BumpForce(2.0)+QuickStop(3.0) passive on Breaker | BROKEN: BumpForce wrong syntax |
| FlashStep | flashstep.evolution.ron | Breaker Speed×2, Reflex×1 | FlashStep on Dash | BROKEN: FlashStep+Dash not in enums |
| Chain Reaction (evolution) | chain_reaction.evolution.ron | Chain Reaction×1, Aftershock×2, Cascade×2 | Shockwave on CellDestroyed | VALID but name collision with Chain Reaction template |

## Key Known Issues (updated after RON–code compat audit, feature/scenario-coverage)

1. **Feedback Loop name collision**: legendary chip template name = potential evolution name. Authoring a "Feedback Loop" evolution would overwrite the legendary in ChipCatalog.
2. **Nova Lance RON was stale**: previous memory entry said "Shockwave on PerfectBumped+Impacted(Cell)" — now correctly PiercingBeam. Updated.
3. **powder_keg.chip.ron BROKEN**: Uses `Explode(base_range:..., range_per_level:..., stacks:..., speed:...)` which are Shockwave params. Explode only has `range` and `damage_mult`. Will fail deserialization.
4. **anchor.evolution.ron BROKEN**: Uses `BumpForce(multiplier: 2.0)` — BumpForce is a tuple variant, must be `BumpForce(2.0)`.
5. **flashstep.evolution.ron BROKEN**: `FlashStep` not in EffectKind enum; `Dash` not in Trigger enum. Both need to be added.
6. **Chain Reaction name collision**: Evolution file named "Chain Reaction" collides with template chip of the same name in ChipCatalog.
7. **desperation.chip.ron — SpeedBoost on Breaker is a no-op**: Breaker has no ActiveSpeedBoosts component; the boost silently does nothing.
8. **Shield**: no chip template uses Shield — only Aegis breaker definition uses it (confirmed via Parry legendary which does use Shield).
9. **ChipTemplate has no description field**: all non-evolution chips have empty description in ChipDefinition.
10. **SecondWind VFX**: wall despawns silently — no graphical flourish as described in design doc.
11. **GravityWell cap enforcement**: design says "oldest despawned first"; code despawn order is arbitrary (ECS query iteration not FIFO).
12. **PiercingBeam**: design says "fast-expanding" beam; code fires all damage in one tick via deferred request (no visual expansion entity).
13. **Pulse reverse()**: design says "no-op"; code removes PulseEmitter. Code is more correct; design doc is stale.
14. **Chain Hit chip**: listed in chip-catalog.md but no EffectKind variant and no RON file — design placeholder never implemented.

**Why:** Documented during comprehensive code-vs-design effect audit. Load-bearing for VFX implementation, content authoring, and any Phase 5+ effect system changes.
**How to apply:** When writing specs for VFX wiring, new chip content, or effect system changes, reference these known gaps to ensure they are addressed or tracked.
