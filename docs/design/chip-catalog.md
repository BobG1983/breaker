# Chip Catalog

Authoring reference for all chip templates and their rarity variants. Each entry defines the chip concept, naming convention, per-rarity effects, and synergy notes.

See `docs/design/decisions/chip-template-system.md` for the template format and `docs/design/decisions/chip-rarity-rework.md` for the rarity philosophy.

## Notation

Effects are shown in condensed RON syntax matching the actual `.chip.ron` files. All chips use `On(target: X, then: [...])` to specify which entity the effects target. Triggers inside the `On` wrapper use targeted names (`PerfectBumped`, `Impacted(Cell)`, `DestroyedCell`) because they fire on the specific entity, not globally.

## Naming Convention

| Rarity | Prefix Style |
|--------|-------------|
| Common | Weak adjective (Basic, Minor, Slight) |
| Uncommon | Moderate adjective (Keen, Potent, Sturdy) |
| Rare | Strong adjective (Brutal, Savage, Lethal) |
| Legendary | Unique proper name |

## Passive Chips

### Piercing Shot
Bolt passes through cells before stopping.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Bolt) → Do(Piercing(1))` | Pure utility |
| Uncommon | Keen | `On(Bolt) → Do(Piercing(2))` | Enables multi-cell combos |
| Rare | Brutal | `On(Bolt) → Do(Piercing(3)), Do(DamageBoost(1.1))` | Opens DamageBoost synergy path |

`max_taken: 3`

### Damage Boost
Multiplicative damage boost per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → Do(DamageBoost(1.1))` | Pure damage |
| Uncommon | Potent | `On(Bolt) → Do(DamageBoost(1.2))` | Meaningful scaling |
| Rare | Savage | `On(Bolt) → Do(DamageBoost(1.35))` | High-value target for damage builds |

`max_taken: 5`

### Bolt Speed
Percentage-based bolt speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `On(Bolt) → Do(SpeedBoost(multiplier: 1.1))` | 10% speed boost |
| Uncommon | Swift | `On(Bolt) → Do(SpeedBoost(multiplier: 1.2))` | 20% speed boost |
| Rare | Blazing | `On(Bolt) → Do(SpeedBoost(multiplier: 1.3)), Do(DamageBoost(1.05))` | 30% speed + damage synergy |

`max_taken: 4`

`max_taken: 3`

### Bolt Size
Multiplicative bolt radius increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Broad | `On(Bolt) → Do(SizeBoost(1.15))` | Wider hits |
| Uncommon | Expanded | `On(Bolt) → Do(SizeBoost(1.25))` | Noticeably larger |
| Rare | Massive | `On(Bolt) → Do(SizeBoost(1.4))` | Huge bolt — hard to miss cells |

`max_taken: 3`

### Magnetism
Bolt attracts toward nearest entity of the given type. See [effects/attraction.md](effects/attraction.md) for Attraction mechanics.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Faint | `On(Bolt) → Do(Attraction(Cell, force: 0.5, max_force: 50))` | Slight pull toward cells |
| Uncommon | Steady | `On(Bolt) → Do(Attraction(Cell, force: 1.0, max_force: 80))` | Noticeable attraction |
| Rare | Powerful | `On(Bolt) → Do(Attraction(Cell, force: 1.5, max_force: 120)), Do(SizeBoost(1.1))` | Strong pull + size synergy |

`max_taken: 3`

### Breaker Speed
Percentage-based breaker speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Quick | `On(Breaker) → Do(SpeedBoost(multiplier: 1.1))` | 10% speed boost |
| Uncommon | Agile | `On(Breaker) → Do(SpeedBoost(multiplier: 1.2))` | 20% speed boost |
| Rare | Lightning | `On(Breaker) → Do(SpeedBoost(multiplier: 1.3)), Do(BumpForce(1.1))` | 30% speed + bump force synergy |

`max_taken: 3`

### Bump Force
Multiplicative bump force increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Firm | `On(Breaker) → Do(BumpForce(1.1))` | Slight force boost |
| Uncommon | Strong | `On(Breaker) → Do(BumpForce(1.2))` | Noticeable |
| Rare | Crushing | `On(Breaker) → Do(BumpForce(1.35))` | High force — enables speed-through-force builds |

`max_taken: 3`

### Wide Breaker
Multiplicative breaker width increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `On(Breaker) → Do(SizeBoost(1.05))` | Slightly wider |
| Uncommon | Sturdy | `On(Breaker) → Do(SizeBoost(1.1))` | Noticeably wider |
| Rare | Massive | `On(Breaker) → Do(SizeBoost(1.15))` | Very wide — safety at the cost of a chip slot |

`max_taken: 3`

## Named Chips

### Amp
Ramping damage bonus that stacks on cell hits.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Bolt) → Do(RampingDamage(damage_per_trigger: 0.02))` | Mild ramp |
| Uncommon | Potent | `On(Bolt) → Do(RampingDamage(damage_per_trigger: 0.04))` | Decent ramp |
| Rare | Savage | `On(Bolt) → Do(RampingDamage(damage_per_trigger: 0.06))` | High ramp, rewards sustained combos |

`max_taken: 2`

### Augment
Breaker width increase + bump force boost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Breaker) → Do(SizeBoost(1.15)), Do(BumpForce(1.15))` | Mild utility |
| Uncommon | Sturdy | `On(Breaker) → Do(SizeBoost(1.25)), Do(BumpForce(1.25))` | Noticeable |
| Rare | Fortified | `On(Breaker) → Do(SizeBoost(1.4)), Do(BumpForce(1.35)), Do(SpeedBoost(multiplier: 1.15))` | Width + force + speed |

`max_taken: 2`

### Overclock
Timed speed burst after perfect bump. Uses `Until(TimeExpires)` for automatic removal.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Bolt) → When(PerfectBumped) → Until(TimeExpires(2.0)) → Do(SpeedBoost(multiplier: 1.3))` | Short burst |
| Uncommon | Charged | `On(Bolt) → When(PerfectBumped) → Until(TimeExpires(3.0)) → Do(SpeedBoost(multiplier: 1.5))` | Longer, stronger |
| Rare | Supercharged | `On(Bolt) → When(PerfectBumped) → Until(TimeExpires(4.0)) → Do(SpeedBoost(multiplier: 1.8))` | Major burst |

`max_taken: 2`

### Flux
Randomness/instability — fires random effect from weighted pool on bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Breaker) → When(Bump) → Do(RandomEffect([(0.5, SpeedBoost(1.1)), (0.5, Shockwave(base_range: 24))]))` | 2-effect pool |
| Uncommon | Volatile | `On(Breaker) → When(Bump) → Do(RandomEffect([(0.35, SpeedBoost(1.15)), (0.35, Shockwave(base_range: 32)), (0.30, ChainBolt(tether_distance: 100))]))` | 3-effect pool |
| Rare | Critical | `On(Breaker) → When(Bump) → Do(RandomEffect([(0.3, SpeedBoost(1.2)), (0.25, Shockwave(base_range: 40)), (0.25, ChainBolt(tether_distance: 120)), (0.2, SpawnBolts())]))` | 4-effect pool — SpawnBolts opens multi-bolt synergy |

`max_taken: 2`

### Last Stand
Bolt-lost redemption — breaker speeds up when a bolt is lost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Breaker) → When(BoltLost) → Do(SpeedBoost(multiplier: 1.15))` | Mild comfort |
| Uncommon | Strong | `On(Breaker) → When(BoltLost) → Do(SpeedBoost(multiplier: 1.3))` | Meaningful recovery |
| Rare | Desperate | `On(Breaker) → When(BoltLost) → Do(SpeedBoost(multiplier: 1.5))` | Major boost — rewards surviving bolt loss |

`max_taken: 1`

## Triggered Chips

### Surge
Speed boost on perfect bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Bolt) → When(PerfectBumped) → Do(SpeedBoost(multiplier: 1.2))` | Mild boost |
| Uncommon | Strong | `On(Bolt) → When(PerfectBumped) → Do(SpeedBoost(multiplier: 1.35))` | Noticeable |
| Rare | Extreme | `On(Bolt) → When(PerfectBumped) → Do(SpeedBoost(multiplier: 1.5))` | Major speed — synergizes with damage builds |

`max_taken: 3`

### Cascade
Shockwave on cell destruction.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → When(DestroyedCell) → Do(Shockwave(base_range: 20, range_per_level: 5, stacks: 1, speed: 400))` | Small wave |
| Uncommon | Spreading | `On(Bolt) → When(DestroyedCell) → Do(Shockwave(base_range: 30, range_per_level: 8, stacks: 1, speed: 400))` | Medium wave |
| Rare | Devastating | `On(Bolt) → When(DestroyedCell) → Do(Shockwave(base_range: 40, range_per_level: 12, stacks: 1, speed: 400))` | Large wave — evolution ingredient for Entropy Engine |

`max_taken: 3`

### Impact
Shockwave on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(Shockwave(base_range: 24, range_per_level: 6, stacks: 1, speed: 400))` | Conditional shockwave |
| Uncommon | Strong | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(Shockwave(base_range: 36, range_per_level: 10, stacks: 1, speed: 400))` | Better range |
| Rare | Devastating | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(Shockwave(base_range: 48, range_per_level: 14, stacks: 1, speed: 400))` | Large AoE on precision play |

`max_taken: 3`

### Tether
Chain bolt on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Short | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(ChainBolt(tether_distance: 80))` | Short tether |
| Uncommon | Extended | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(ChainBolt(tether_distance: 120))` | Medium tether |
| Rare | Long | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(ChainBolt(tether_distance: 160))` | Long tether — wide coverage |

`max_taken: 2`

### Aftershock
Shockwave on bolt-wall bounce.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → When(Impacted(Wall)) → Do(Shockwave(base_range: 16, range_per_level: 4, stacks: 1, speed: 400))` | Small wall shockwave |
| Uncommon | Rumbling | `On(Bolt) → When(Impacted(Wall)) → Do(Shockwave(base_range: 24, range_per_level: 6, stacks: 1, speed: 400))` | Medium |
| Rare | Thundering | `On(Bolt) → When(Impacted(Wall)) → Do(Shockwave(base_range: 32, range_per_level: 10, stacks: 1, speed: 400))` | Large wall shockwave |

`max_taken: 3`

### Reflex
Spawn bolts on perfect bump. Uncommon and rare only — no common variant.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Uncommon | Quick | `On(Breaker) → When(PerfectBump) → Do(SpawnBolts())` | Extra bolt on precision |
| Rare | Lightning | `On(Breaker) → When(PerfectBump) → Do(SpawnBolts()), Do(SpeedBoost(multiplier: 1.2))` | Bolt spawn + speed burst |

`max_taken: 1`

### Splinter
Spawn tiny temporary bolts on cell destruction. Parent bolt shrinks as a trade-off — higher rarity = more splinters, less shrinkage.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → When(DestroyedCell) → Do(SpawnBolts(count: 1, lifespan: 2.0)), Do(SizeBoost(0.5))` | 1 splinter, 0.5x size |
| Uncommon | Spreading | `On(Bolt) → When(DestroyedCell) → Do(SpawnBolts(count: 2, lifespan: 2.5)), Do(SizeBoost(0.55))` | 2 splinters, 0.55x size |
| Rare | Devastating | `On(Bolt) → When(DestroyedCell) → Do(SpawnBolts(count: 3, lifespan: 3.0)), Do(SizeBoost(0.6))` | 3 splinters, 0.6x size — evolution ingredient for Split Decision |

`max_taken: 2`

## Legendaries

All legendaries are `max_taken: 1`. Template has only the `legendary` slot filled.

### Ricochet Protocol
Wall-bank damage boost until next cell hit.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(Impacted(Wall)) → Until(Impacted(Cell)) → Do(DamageBoost(3.0))` | Wall bounce grants 3x damage until next cell hit. Rewards precise wall-bank shots. |

### Glass Cannon
Doubled damage, smaller bolt.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(DamageBoost(2.0)), Do(SizeBoost(0.7))` | 2x damage but 0.7x bolt size. High risk / high reward. |

### Desperation
Breaker speeds up when a bolt is lost.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Breaker) → When(BoltLost) → Do(SpeedBoost(multiplier: 2.0))` | 2x breaker speed on bolt loss. Snowballing risk/reward. |

### Deadline
Timer pressure = bolt speed.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(NodeTimerThreshold(0.25)) → Do(SpeedBoost(multiplier: 2.0))` | When timer drops below 25%, bolt speed 2x. Rewards playing on the edge. |

### Whiplash
Whiff redemption — miss a bump, but the next cell impact gets bonus damage + shockwave.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(BumpWhiff) → When(Impacted(Cell)) → Once([Do(DamageBoost(2.5))]), Do(Shockwave(base_range: 64, speed: 500))` | Turns whiffs into comebacks. DamageBoost fires once via `Once`, shockwave fires every time. |

### Singularity
Small bolt, big damage, fast.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(SizeBoost(0.6)), Do(DamageBoost(2.5)), Do(SpeedBoost(multiplier: 1.4))` | Tiny bolt that hits like a truck. Hard to control, massive payoff. |

### Gauntlet
Large bolt, fast, weak hits.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(SizeBoost(1.5)), Do(SpeedBoost(multiplier: 1.4)), Do(DamageBoost(0.5))` | 1.5x bolt size, 40% faster, but 0.5x damage. Safety-focused build-around. |

### Chain Reaction
Recursive destruction — nested cell destruction triggers spawn bolts.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(DestroyedCell) → When(DestroyedCell) → Do(SpawnBolts())` | Cells destroyed by effects (shockwaves, chain lightning) from a cell destruction trigger spawn bolts. Requires an external destruction source to start the chain. |

### Feedback Loop
Perfect bump → cell impact → cell destruction → timed speed burst.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → When(DestroyedCell) → Until(TimeExpires(3.0)) → Do(SpeedBoost(multiplier: 1.5))` | Deep trigger chain rewards a full precision sequence with a 3s speed burst. |

### Parry
Perfect bump grants breaker immunity + all bolts emit shockwaves.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Breaker) → When(PerfectBump) → Do(Shield(duration: 5.0))` + `On(AllBolts) → When(PerfectBump) → Do(Shockwave(base_range: 64, speed: 500))` | Two On targets: perfect bump spawns a timed floor wall, every bolt shockwaves. Rewards precision with brief invulnerability + area damage burst. |

### Powder Keg
Cells hit by the bolt explode on death. Uses `Explode` effect *(not yet implemented)*.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(Impacted(Cell)) → On(Cell) → When(Died) → Do(Explode(range: 48, damage_mult: 1.0))` | Nested On: targets the impacted cell, arms a Died trigger on it. When that cell dies, instant area damage. Chain explosions possible if Explode kills adjacent cells that also have armed Died triggers. |

### Tempo
Speed ramps on consecutive bumps, whiff removes all boosts.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(AllBolts) → When(Bumped) → Until(BumpWhiff) → Do(SpeedBoost(multiplier: 1.2))` | Each bump adds a 1.2x speed layer to the bumped bolt. Whiff strips all layers from all bolts. High-skill momentum legendary — consecutive perfect play builds devastating speed. |

## Evolutions

See `docs/design/evolutions.md` for evolution design principles. Evolution RON files are in `assets/chips/evolutions/`.

### Entropy Engine
**Ingredients**: Cascade x2 + Flux x2

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(EntropyEngine(max_effects: 3, pool: [(0.3, SpawnBolts()), (0.25, Shockwave(base_range: 48, speed: 400)), (0.25, ChainBolt(tether_distance: 120)), (0.20, SpeedBoost(1.3))]))`

Cell destruction rolls from weighted pool (max 3 active effects). Cascade provides the trigger domain, Flux provides the randomness mechanic, evolution adds the effect cap.

### Nova Lance
**Ingredients**: Damage Boost x2 + Bolt Speed x2

**Effect**: `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(Shockwave(base_range: 128, speed: 600))`

Perfect bumps unleash devastating shockwaves on cell impact.

### Voltchain
**Ingredients**: Chain Reaction x1 + Damage Boost x2

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(ChainLightning(arcs: 3, range: 96, damage_mult: 0.5))`

Destroying cells unleashes chain lightning to nearby targets.

### Phantom Breaker
**Ingredients**: Wide Breaker x2 + Bump Force x2

**Effect**: `On(Breaker) → When(Bump) → Do(SpawnPhantom(duration: 5.0, max_active: 1))`

Successful bumps summon a phantom breaker that mirrors your moves.

### Supernova
**Ingredients**: Piercing Shot x3 + Surge x1

**Effect**: `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → When(DestroyedCell) → Do(SpawnBolts(count: 2, inherit: true)), Do(Shockwave(base_range: 96, speed: 400))`

Perfect bumps trigger chain explosions — cells destroyed spawn bolts and shockwaves.

### Dead Man's Hand
**Ingredients**: Damage Boost x3 + Last Stand x1

**Effect**: `On(Breaker) → When(BoltLost) → Do(Shockwave(base_range: 128, speed: 500)), Do(SpeedBoost(multiplier: 1.5))`

Losing a bolt triggers a shockwave and boosts breaker speed.

### Railgun
**Ingredients**: Piercing Shot x3 + Bolt Speed x4

**Effect**: `On(Bolt) → When(PerfectBumped) → Do(PiercingBeam(damage_mult: 3.0, width: 30.0))`

Perfect bumps fire a devastating piercing beam through all cells in the bolt's path.

### Gravity Well
**Ingredients**: Bolt Size x2 + Magnetism x2

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(GravityWell(strength: 500, duration: 3.0, radius: 128, max: 2))`

Destroying cells creates gravity wells that pull bolts toward the destruction point.

### Second Wind
**Ingredients**: Wide Breaker x3 + Breaker Speed x3

**Effect**: `On(Breaker) → When(BoltLost) → Do(SecondWind)`

Bolt loss grants temporary invulnerability — cheat death.

### Split Decision
**Ingredients**: Splinter x2 + Piercing Shot x2

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(SpawnBolts(count: 2, inherit: true))`

Destroyed cells spawn 2 permanent bolts that inherit the parent's effects. Splinter provides the destruction-spawning mechanic, Piercing provides multi-cell reach, evolution removes the downsides (no shrink, no lifespan, inherits effects).
