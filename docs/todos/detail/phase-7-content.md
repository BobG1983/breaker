# Phase 7: Content & Variety

**Goal**: Expand from vertical slice to real content depth. The combinatorial space must be large enough that run 100 still produces novel situations.

## More Cell Types

Each cell type creates a mechanically distinct situation — not just "harder cells" but "different puzzles":

- **Twin cells**: Linked destruction — destroying one damages its twin. Creates spatial strategy (hit the accessible twin to damage the protected one).
- **Gate cells**: Cannot be damaged until all neighbor cells are cleared. Forces clearing order.
- **Shield cells**: Periodically activate a shield that reflects the bolt. Timing-based interaction.
- **Spawner cells**: Periodically spawn new basic cells. Creates urgency within urgency.
- **Portal cells**: Sub-level mechanic — bolt enters a portal, player clears a mini-layout, bolt exits. Complex — scope carefully.

Each cell type multiplies situation variety: a Lock Cell + Regen Cell layout plays completely differently from a Twin Cell + Spawner layout, even at the same tier and difficulty.

## Expanded Chip Pool (40+ total)

Scale from the vertical slice's 16-20 to 40+:

- **More synergy chips**: Cross-chip interactions that weren't possible with the smaller pool. Chips whose effects interact with other chips' components.
- **Trade-off chips**: Benefit + cost. "Double bolt damage, but bolt is 30% smaller." These create agonizing decisions on the chip select screen (Pillar 5: Pressure, Not Panic).
- **Conditional chips**: Effects that only activate under specific circumstances. "While below 2 lives, bump grade is always perfect." Risk/reward chips that create dramatic moments.
- **Archetype-affinity chips**: Some chips are stronger with specific breakers. Doesn't restrict — just incentivizes. Creates build paths per archetype.

### Synergy Depth Target

At 40+ chips, the synergy web must be dense enough that players discover new interactions on run 100+. Design guideline: every chip should interact meaningfully with at least 2-3 other chips. The interaction graph should have no isolated nodes.

## More Evolution Recipes (10-15 total)

- Cross-type evolutions: passive + triggered chip combinations
- Chain evolutions: Evolved chip + another chip = further evolution (rare, powerful, deep knowledge reward)
- Secret evolutions: Undocumented recipes discoverable only through experimentation. Community wiki content.

### Immediate: New Chips + Evolutions (pre-Phase 7)

These are ready to build now — designed, guard-reviewed, and scoped. They close gaps in the evolution ingredient graph (Tether and Aftershock are currently orphaned — no evolution uses them) and add a new chip mechanic (dynamic speed accumulation).

#### Rename: ChainBolt → FlailBolt

The `Effect::ChainBolt` variant, `SpawnChainBolt` message, and all related systems/handlers/files are renamed to `FlailBolt`. The Tether chip RON references `FlailBolt(tether_distance: ...)`. Pure rename — no mechanic changes.

#### New Chip: Rebound (Dynamic Amp)

Wall bounce speed accumulation that resets on non-perfect bumps. First dynamic amp alongside Amp (which ramps on cell hits and resets on non-bump breaker impact). Rewards wall-bank play.

| Rarity | Prefix | Per-bounce | Cap |
|--------|--------|-----------|-----|
| Common | Mild | +10% | +40% |
| Uncommon | Steady | +15% | +60% |
| Rare | Fierce | +20% | +80% |

`max_taken: 3`

New effect: `Effect::Rebound { per_bounce: f32, max_bonus: f32 }`. New component: `ReboundBonus { current: f32, per_bounce: f32, max: f32 }` on bolt entities. Systems: increment on `BoltHitWall`, reset on non-perfect bump, integrate into speed calculation.

#### New Evolution: Razorwire (Tether x2 + Rebound x3)

The tether becomes a weapon. The energy arc connecting paired flail bolts damages any cell it passes through. Wall bounces extend tether distance and boost both bolts' speed. Resets on breaker contact.

**New mechanics**:
- **Tether damage**: Line-segment collision — cells the tether passes through take damage. New damage source type (not point collision). `RazorwireMarker` component on bolt pairs. `tether_damage` system casts ray along tether line, checks quadtree for intersecting cells.
- **Tether extension**: Wall bounces grow `DistanceConstraint.max_distance` (up to cap). `tether_extension` system increments on `BoltHitWall` for Razorwire bolts. Resets on breaker contact.
- **Catch timing risk/reward**: Delay catching the pair to build speed/extension, risk bolt loss

**Pillar analysis** (guard-game-design reviewed):
- **Speed**: Wall bounces literally make it faster. More chaos = more speed = more chaos.
- **Skill ceiling**: Beginner gets passive tether damage. Master angles the launch to maximize wall-bounce count, growing tether extension and sweeping through dense clusters at peak velocity. Three axes: launch angle, catch timing, catch position.
- **Tension**: Speed ramp and tether extension reset on breaker contact. Every bounce away from the breaker builds power that will be lost. Earning power by deferring safety.
- **Meaningful decisions**: Bump early for control vs. let the pair orbit wide for momentum. Position center for safe catch vs. off-center to redirect through a cluster.
- **Synergy web**: Aftershock (more wall bounces = more wall shockwaves), Ricochet Protocol (wall-bounce DamageBoost stacks with tether damage), Bolt Speed Boost (higher base speed = more wall contacts per orbit), Overclock (perfect bump burst starts the arc at high speed), Cascade (cells destroyed by tether arc trigger cascade shockwaves).
- **Juice**: Two bolts connected by a glowing energy whip that lengthens and brightens as speed builds. Damage numbers pop along the arc. Speed trails intensify.

#### New Evolution: Shatter Grid (Aftershock x3 + Cascade x3)

Recursive shockwave chaining. Every shockwave triggers secondary smaller shockwaves at each cell it damages, which trigger tertiary shockwaves, and so on until the range drops below a minimum threshold.

- Each generation: range × 0.6, damage × 0.5
- Stops when `base_range < 8.0` world units
- Safety cap: max depth 5

Closes Aftershock's orphan gap. The "I committed to shockwaves from every source" build. Screen fills with expanding rings.

#### Ingredient Resolution: Feedback Loop (Surge x3 + Rebound x3)

Resolves the long-standing "TBD" ingredients. Surge rewards each individual perfect bump with speed. Rebound rewards the wall-bounce chain between bumps. Combined fantasy: sustained momentum mastery — maintaining a speed chain through wall bounces between bumps, and every third perfect bump unleashes a swarm.

Gives Rebound TWO evolution paths: Razorwire (tether damage arc) vs Feedback Loop (sustained precision burst). "I maxed Rebound — which path do I commit to?" is a meaningful, build-defining decision. Surge also gains tension: x1 for Supernova vs x3 for Feedback Loop — different stack depths, different paths.

#### Design Rationale: Rejected Alternatives

- **Tether + Aftershock**: No mechanical overlap. Tether is paired bolt physics, Aftershock is wall-bounce area damage. "Two things that happen near walls" is not a build identity.
- **Rebound + Aftershock**: Too similar to Shatter Grid (both "wall bounce = more area damage"). Shatter Grid is better because shockwave chaining is a genuinely new interaction; Rebound + Aftershock would just be "bigger shockwaves when you bounce more" — a stat buff with extra steps.
- **"More speed" evolution from Rebound alone**: Fails the "not a stat buff" test from evolutions.md. Rebound is already speed accumulation. An evolution that is "even more speed accumulation" introduces no new interaction points.

#### Post-Change Ingredient Coverage

After these additions, every triggered common/uncommon/rare chip has at least one evolution path. No dead ends in the build-crafting tree.

| Chip | Evolution paths | Notes |
|------|----------------|-------|
| Piercing | Chain Reaction, Supernova, Railgun | 3 paths (high-utility) |
| Damage Boost | Nova Lance, Voltchain, Dead Man's Hand | 3 paths (high-utility) |
| Cascade | Entropy Engine, Chain Reaction, **Shatter Grid** | 3 paths |
| Bolt Speed | Nova Lance, Railgun | 2 paths |
| Surge | Supernova, **Feedback Loop** | 2 paths |
| Rebound | **Razorwire**, **Feedback Loop** | 2 paths (new chip, immediately high-value draft pick) |
| Wide Breaker | Phantom Breaker, Second Wind | 2 paths |
| Aftershock | **Shatter Grid** | 1 path (was orphaned) |
| Tether | **Razorwire** | 1 path (was orphaned) |
| Bump Force | Phantom Breaker | 1 path |
| Breaker Speed | Second Wind | 1 path |
| Chain Hit | Voltchain | 1 path |
| Flux | Entropy Engine | 1 path |
| Splinter | Chain Reaction | 1 path |
| Last Stand | Dead Man's Hand | 1 path |
| Magnetism | Gravity Well | 1 path |
| Bolt Size | Gravity Well | 1 path |

#### Implementation Waves

```
Wave 1 (parallel):  FlailBolt rename  |  Shatter Grid  |  Feedback Loop RON
                          ↓                   ↓                  ↓
Wave 2:             Rebound chip (after Wave 1 — shares definition.rs)
                          ↓
Wave 3:             Razorwire evolution (needs FlailBolt + Rebound)
                          ↓
Wave 4:             Scenarios + invariants → Final verification
```

#### Scenarios

- **rebound_buildup**: Wall-bounce bonus accumulates + resets correctly
- **rebound_cap**: Bonus never exceeds cap at max stacks
- **razorwire_tether_damage**: Tether line damages cells it passes through
- **shatter_grid_chain**: Shockwave chaining terminates correctly (no infinite loops)

#### Invariants

- `ReboundBonusCapped` — ReboundBonus.current ≤ max for all bolt entities
- `ShatterGridDepthBounded` — recursive shockwave depth never exceeds safety cap
- `TetherDamageOnlyWithMarker` — tether damage only fires when RazorwireMarker is present

## Active Nodes

Active nodes must feel fundamentally different from passive nodes, not just "passive nodes with moving cells":

- **Moving cell formations**: Cells orbit, oscillate, or march across the playfield
- **Hazards**: Environmental threats that attack the breaker (falling debris, laser sweeps)
- **Retaliating cells**: Cells that fire projectiles back at the breaker when hit
- **Timed cell patterns**: Cells that appear and disappear in phases

## Level Generation

- Procedural level layouts with difficulty parameters (or large hand-crafted pool — 20+ per node type)
- Layout templates with variable fill (same shape, different cell types per seed)
- Node type distribution follows the tier system from Phase 4e

## Infrastructure Note

**New agent for Phase 7:** Introduce `reviewer-compile-time` agent to monitor compile-time impact as the crate grows with 40+ chips, new cell types, and expanded RON content. Track incremental build times and flag when a module or derive macro pushes compile time past acceptable thresholds.

## Acceptance Criteria

1. 40+ chips with functional effects
2. At least 5 new cell types with distinct mechanics
3. 10-15 evolution recipes including at least 2 secret/undocumented ones
4. Active nodes feel mechanically distinct from passive nodes
5. Synergy interactions are dense enough that new combinations are discoverable after 50+ runs
