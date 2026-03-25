# Chip Catalog

Authoring reference for all chip templates and their rarity variants. Each entry defines the chip concept, naming convention, per-rarity effects, and synergy notes.

See `docs/design/decisions/chip-template-system.md` for the template format and `docs/design/decisions/chip-rarity-rework.md` for the rarity philosophy.

## Naming Convention

| Rarity | Prefix Style |
|--------|-------------|
| Common | Weak adjective (Basic, Minor, Slight) |
| Uncommon | Moderate adjective (Keen, Potent, Sturdy) |
| Rare | Strong adjective (Brutal, Savage, Lethal) |
| Legendary | Unique proper name |

## Passive Chips (OnSelected)

### Piercing
Bolt passes through cells before stopping.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `Do(Piercing(1))` | Pure utility |
| Uncommon | Keen | `Do(Piercing(2))` | Enables multi-cell combos |
| Rare | Brutal | `Do(Piercing(3)), Do(DamageBoost(0.1))` | Opens DamageBoost synergy path |

`max_taken: 3`

### Damage Boost
Fractional bonus damage per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `Do(DamageBoost(0.1))` | Pure damage |
| Uncommon | Potent | `Do(DamageBoost(0.2))` | Meaningful scaling |
| Rare | Savage | `Do(DamageBoost(0.35))` | High-value target for damage builds |

`max_taken: 5`

### Bolt Speed Boost
Percentage-based bolt speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `Do(SpeedBoost(target: Bolt, multiplier: 1.1))` | 10% speed boost |
| Uncommon | Swift | `Do(SpeedBoost(target: Bolt, multiplier: 1.2))` | 20% speed boost |
| Rare | Blazing | `Do(SpeedBoost(target: Bolt, multiplier: 1.3)), Do(DamageBoost(0.05))` | 30% speed + damage synergy |

`max_taken: 3`

### Chain Hit
Bolt chains to additional cells on hit.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Linked | `Do(ChainHit(1))` | Single chain |
| Uncommon | Branching | `Do(ChainHit(2))` | Multi-chain |
| Rare | Arcing | `Do(ChainHit(3)), Do(DamageBoost(0.05))` | Chains + damage |

`max_taken: 3`

### Bolt Size Boost
Increases bolt radius by a fraction per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Broad | `Do(SizeBoost(Bolt, 0.15))` | Wider hits |
| Uncommon | Expanded | `Do(SizeBoost(Bolt, 0.25))` | Noticeably larger |
| Rare | Massive | `Do(SizeBoost(Bolt, 0.4))` | Huge bolt — hard to miss cells |

`max_taken: 3`

### Attraction (Magnetism)
Bolt attracts toward nearest entity of the given type. See `triggers-and-effects.md` for Attraction mechanics.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Faint | `Do(Attraction(Cell, 0.5))` | Slight pull toward cells |
| Uncommon | Steady | `Do(Attraction(Cell, 1.0))` | Noticeable attraction |
| Rare | Powerful | `Do(Attraction(Cell, 1.5)), Do(SizeBoost(Bolt, 0.1))` | Strong pull + size synergy |

`max_taken: 3`

### Breaker Speed Boost
Percentage-based breaker speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Quick | `Do(SpeedBoost(target: Breaker, multiplier: 1.1))` | 10% speed boost |
| Uncommon | Agile | `Do(SpeedBoost(target: Breaker, multiplier: 1.2))` | 20% speed boost |
| Rare | Lightning | `Do(SpeedBoost(target: Breaker, multiplier: 1.3)), Do(BumpForce(5.0))` | 30% speed + bump force synergy |

`max_taken: 3`

### Bump Force
Flat bump force increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Firm | `Do(BumpForce(10.0))` | Slight force boost |
| Uncommon | Strong | `Do(BumpForce(20.0))` | Noticeable |
| Rare | Crushing | `Do(BumpForce(35.0))` | High force — enables speed-through-force builds |

`max_taken: 3`

### Tilt Control
Flat tilt control sensitivity increase.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Subtle | `Do(TiltControl(0.1))` | Minor control |
| Uncommon | Precise | `Do(TiltControl(0.2))` | Better aim |
| Rare | Masterful | `Do(TiltControl(0.35))` | Precise aim + feels responsive |

`max_taken: 3`

## Named Chips

### Amp
Ramping damage bonus that stacks on cell hits, resets on non-bump breaker impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `When(OnSelected, [Do(RampingDamage(bonus_per_hit: 0.02, max_bonus: 0.2))])` | Mild ramp |
| Uncommon | Potent | `When(OnSelected, [Do(RampingDamage(bonus_per_hit: 0.04, max_bonus: 0.4))])` | Decent ramp |
| Rare | Savage | `When(OnSelected, [Do(RampingDamage(bonus_per_hit: 0.06, max_bonus: 0.6))])` | High ramp, rewards sustained combos |

`max_taken: 2`

### Augment
Breaker width increase + bump force boost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `When(OnSelected, [Do(SizeBoost(Breaker, 6.0)), Do(BumpForce(8.0))])` | Mild utility |
| Uncommon | Sturdy | `When(OnSelected, [Do(SizeBoost(Breaker, 10.0)), Do(BumpForce(15.0))])` | Noticeable |
| Rare | Fortified | `When(OnSelected, [Do(SizeBoost(Breaker, 16.0)), Do(BumpForce(25.0)), Do(SpeedBoost(Breaker, 1.2))])` | Width + force + speed |

`max_taken: 2`

### Overclock
Timed speed burst after perfect bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `When(OnPerfectBump, [Do(TimedSpeedBurst(speed_mult: 1.3, duration_secs: 2.0))])` | Short burst |
| Uncommon | Charged | `When(OnPerfectBump, [Do(TimedSpeedBurst(speed_mult: 1.5, duration_secs: 3.0))])` | Longer, stronger |
| Rare | Supercharged | `When(OnPerfectBump, [Do(TimedSpeedBurst(speed_mult: 1.8, duration_secs: 4.0))])` | Major burst |

`max_taken: 2`

### Flux
Randomness/instability — fires random effect from weighted pool on bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `When(OnBump, [Do(RandomEffect([(0.5, Do(SpeedBoost(Bolt, 1.1))), (0.5, When(OnImpact(Cell), [Do(Shockwave(base_range: 24, ...))]))]))])` | 2-effect pool |
| Uncommon | Volatile | `When(OnBump, [Do(RandomEffect([(0.35, Do(SpeedBoost(Bolt, 1.15))), (0.35, When(OnImpact(Cell), [Do(Shockwave(base_range: 32, ...))])), (0.30, When(OnImpact(Cell), [Do(ChainLightning(range: 64, jumps: 2))]))]))])` | 3-effect pool |
| Rare | Critical | `When(OnBump, [Do(RandomEffect([(0.3, Do(SpeedBoost(Bolt, 1.2))), (0.25, When(OnImpact(Cell), [Do(Shockwave(base_range: 40, ...))])), (0.25, When(OnImpact(Cell), [Do(ChainLightning(range: 80, jumps: 3))])), (0.2, Do(SpawnBolts {}))]))])` | 4-effect pool — SpawnBolts opens multi-bolt synergy |

`max_taken: 2`

## Triggered Chips

### Surge
Speed boost on perfect bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `When(OnPerfectBump, [Do(SpeedBoost(Bolt, 1.2))])` | Mild boost |
| Uncommon | Strong | `When(OnPerfectBump, [Do(SpeedBoost(Bolt, 1.35))])` | Noticeable |
| Rare | Extreme | `When(OnPerfectBump, [Do(SpeedBoost(Bolt, 1.5))])` | Major speed — synergizes with damage builds |

`max_taken: 3`

### Cascade
Shockwave on cell destruction.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `When(OnCellDestroyed, [Do(Shockwave(base_range: 20, range_per_level: 5, stacks: 1))])` | Small wave |
| Uncommon | Spreading | `When(OnCellDestroyed, [Do(Shockwave(base_range: 30, range_per_level: 8, stacks: 1))])` | Medium wave |
| Rare | Devastating | `When(OnCellDestroyed, [Do(Shockwave(base_range: 40, range_per_level: 12, stacks: 1))])` | Large wave — evolution ingredient for Entropy Engine |

`max_taken: 3`

### Impact
Shockwave on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(Shockwave(base_range: 24, range_per_level: 6, stacks: 1))])])` | Conditional shockwave |
| Uncommon | Strong | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(Shockwave(base_range: 36, range_per_level: 10, stacks: 1))])])` | Better range |
| Rare | Devastating | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(Shockwave(base_range: 48, range_per_level: 14, stacks: 1))])])` | Large AoE on precision play |

`max_taken: 3`

### Tether
Chain bolt on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Short | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(ChainBolt(tether_distance: 80))])])` | Short tether |
| Uncommon | Extended | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(ChainBolt(tether_distance: 120))])])` | Medium tether |
| Rare | Long | `When(OnPerfectBump, [When(OnImpact(Cell), [Do(ChainBolt(tether_distance: 160))])])` | Long tether — wide coverage |

`max_taken: 2`

### Aftershock
Shockwave on bolt-wall bounce.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `When(OnImpact(Wall), [Do(Shockwave(base_range: 16, range_per_level: 4, stacks: 1))])` | Small wall shockwave |
| Uncommon | Rumbling | `When(OnImpact(Wall), [Do(Shockwave(base_range: 24, range_per_level: 6, stacks: 1))])` | Medium |
| Rare | Thundering | `When(OnImpact(Wall), [Do(Shockwave(base_range: 32, range_per_level: 10, stacks: 1))])` | Large wall shockwave |

`max_taken: 3`

### Splinter
Spawn temporary bolts on cell destruction. Spawned bolts are small, temporary, and do not inherit effects.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `When(OnCellDestroyed, [Do(SpawnBolts { count: 1, lifespan: Some(2.0) }), Do(SizeBoost(Bolt, -0.3))])` | Single small temporary bolt |
| Uncommon | Spreading | `When(OnCellDestroyed, [Do(SpawnBolts { count: 2, lifespan: Some(2.5) }), Do(SizeBoost(Bolt, -0.25))])` | Two temporary bolts |
| Rare | Devastating | `When(OnCellDestroyed, [Do(SpawnBolts { count: 3, lifespan: Some(3.0) }), Do(SizeBoost(Bolt, -0.2))])` | Three temporary bolts |

`max_taken: 2`

## Legendaries

All legendaries are `max_taken: 1`. Template has only the `legendary` slot filled. 13 total.

### Ricochet Protocol
Wall-bank damage boost.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnImpact(Wall), [Until(until: OnImpact(Breaker), then: [Do(DamageBoost(2.0))])])` | Wall bounce grants 2x damage until bolt returns to breaker. Rewards precise wall-bank shots. Build-around: pair with wall-impact chips. |

### Glass Cannon
2x damage, 30% narrow breaker.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnSelected, [Do(DamageBoost(2.0)), Do(SizeBoost(Breaker, -0.3))])` | High risk / high reward. 2x damage but 30% narrower breaker. |

### Desperation
All bolts speed up dramatically when a bolt is lost.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnBoltLost, [Do(SpeedBoost(AllBolts, 2.0))])` | Permanent 2x speed to all bolts on bolt loss. Snowballing risk/reward. |

### Deadline
Timer pressure = bolt speed.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnSelected, [Do(TimePressureBoost(speed_mult: 2.0, threshold_pct: 0.25))])` | When timer < 25%, bolt speed 2x. Rewards playing on the edge. |

### Whiplash
Whiff redemption — miss a bump, but the next cell impact gets bonus damage + shockwave.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnBumpWhiff, [When(OnImpact(Cell), [Until(until: OnImpact(Breaker), then: [Do(DamageBoost(1.5)), Do(Shockwave(base_range: 64.0, ...))])])])` | Turns whiffs into comebacks. The damage boost lasts until the bolt returns to the breaker. |

### Singularity
Small bolt, big damage, fast.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnSelected, [Do(SizeBoost(Bolt, -0.4)), Do(DamageBoost(1.5)), Do(SpeedBoost(Bolt, 1.4))])` | Tiny bolt that hits like a truck. Hard to control, massive payoff. Inverse Glass Cannon — bolt risk instead of breaker risk. |

### Gauntlet
Wide breaker, fast breaker, weak hits.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnSelected, [Do(SizeBoost(Breaker, 0.5)), Do(SpeedBoost(Breaker, 1.4)), Do(DamageBoost(-0.5))])` | 50% wider breaker, 40% faster movement, but halved damage. Safety-focused build-around. |

### Split Decision
Spawn bolts on cell destruction, bolts inherit effects.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnCellDestroyed, [Do(SpawnBolts { count: 2, inherit: true })])` | Destroyed cells spawn 2 permanent bolts that inherit the parent's effects. Chain reaction potential. |

### Event Horizon
Gravity well on cell destruction.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnCellDestroyed, [Do(GravityWell(strength: 500.0, duration: 3.0, radius: 128.0, max: 2))])` | Each cell death creates a gravity well pulling bolts toward the destruction point. Clusters damage. |

### Parry
Perfect bump grants temporary damage immunity + shockwave.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnPerfectBump, [Until(until: TimeExpires(2.0), then: [Do(Shield(base_duration: 2.0, ...))]), Do(Shockwave(base_range: 64.0, ...))])` | Rewards precision with a brief invulnerability window + shockwave burst. |

### Second Strike
Perfect bumps fire a piercing beam.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnPerfectBump, [Do(PiercingBeam(damage_mult: 2.0, width: 30.0))])` | Railgun-lite on every perfect bump. Pure offensive legendary for precision players. |

### Powder Keg
Cells explode in chain shockwaves on destruction.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnCellDestroyed, [Do(Shockwave(base_range: 48.0, ...)), When(OnCellDestroyed, [Do(Shockwave(base_range: 32.0, ...))])])` | Destruction cascades — each destroyed cell shockwaves, and cells destroyed by those shockwaves trigger smaller secondary shockwaves. |

### Tempo
Speed ramps on consecutive bumps, resets on whiff.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `When(OnBump, [Do(TimedSpeedBurst(speed_mult: 1.2, duration_secs: 5.0))]), When(OnBumpWhiff, [Do(SpeedBoost(AllBolts, 0.5))])` | Consecutive bumps stack speed bursts. A whiff cuts all bolt speed in half. High-skill momentum legendary. |

## Evolutions

See `docs/design/evolutions.md` for evolution design principles.

### Entropy Engine
**Ingredients**: Cascade + Flux

**Effect**: `When(OnCellDestroyed, [Do(EntropyEngine(5, [(0.3, Do(SpawnBolts {})), (0.25, Do(Shockwave(base_range: 48, ...))), (0.25, Do(ChainLightning(range: 80, jumps: 3))), (0.20, Do(SpeedBoost(Bolt, 1.3)))]))])`

Every 5th cell destroyed, roll from weighted pool. Cascade provides the trigger domain, Flux provides the randomness mechanic, evolution adds the counter gate.

### Chain Reaction
**Ingredients**: Cascade x3 + Splinter x2 + Piercing x3

**Effect**: `When(OnCellDestroyed, [Do(SpawnBolts { count: 2, lifespan: Some(3.0), inherit: true })])`

Recursive bolt spawning with effect inheritance. Destroyed cells spawn temporary bolts that inherit the parent's effects (including piercing and cascading shockwaves), creating chain reactions that spread across the field.

### Feedback Loop
**Ingredients**: TBD

**Effect**: 3 perfect bumps trigger bolt spawn + shockwave.

`When(OnPerfectBump, [Do(EntropyEngine(3, [(0.5, Do(SpawnBolts { count: 2 })), (0.5, Do(Shockwave(base_range: 96.0, ...)))]))])`

Counter-gated burst: every 3rd perfect bump fires a bolt swarm and shockwave. Rewards consistent precision over time.

### Nova Lance
**Ingredients**: Damage Boost x2 + Bolt Speed x2

**Effect**: `When(OnPerfectBump, [When(OnImpact(Cell), [Do(Shockwave(base_range: 128.0, ...))])])`

Perfect bumps unleash devastating shockwaves on cell impact.

### Voltchain
**Ingredients**: Chain Hit x2 + Damage Boost x2

**Effect**: `When(OnCellDestroyed, [Do(ChainLightning(arcs: 3, range: 96.0, damage_mult: 0.5))])`

Destroying cells unleashes chain lightning to nearby targets.

### Phantom Breaker
**Ingredients**: Wide Breaker x2 + Bump Force x2

**Effect**: `When(OnBump, [Do(SpawnPhantom(duration: 5.0, max_active: 1))])`

Successful bumps summon a phantom breaker that mirrors your moves.

### Supernova
**Ingredients**: Piercing Shot x3 + Surge x1

**Effect**: `When(OnPerfectBump, [When(OnImpact(Cell), [When(OnCellDestroyed, [Do(MultiBolt(base_count: 2, count_per_level: 0, stacks: 1)), Do(Shockwave(base_range: 96.0, ...))])])])`

Perfect bumps trigger chain explosions — cells destroyed spawn bolts and shockwaves.

### Dead Man's Hand
**Ingredients**: Damage Boost x3 + Last Stand x1

**Effect**: `When(OnBoltLost, [Do(Shockwave(base_range: 128.0, ...)), Do(SpeedBoost(AllBolts, 1.5))])`

Losing a bolt triggers a shockwave and boosts all remaining bolts.

**Redesign note (C7)**: Planned rework to Pulse effect — all bolts shockwave on bolt lost, rather than a single point-source shockwave. Creates a distributed damage response.

### Railgun
**Ingredients**: Piercing Shot x3 + Bolt Speed x4

**Effect**: `When(OnPerfectBump, [Do(PiercingBeam(damage_mult: 3.0, width: 30.0))])`

Perfect bumps fire a devastating piercing beam through all cells in the bolt's path.

### Gravity Well
**Ingredients**: Bolt Size x2 + Magnetism x2

**Effect**: `When(OnCellDestroyed, [Do(GravityWell(strength: 500.0, duration: 3.0, radius: 128.0, max: 2))])`

Destroying cells creates gravity wells that pull bolts toward the destruction point.

### Second Wind
**Ingredients**: Wide Breaker x3 + Breaker Speed x3

**Effect**: `Once([Do(SecondWind(invuln_secs: 3.0))])`

Invisible wall that bounces the bolt once per node. Applied to the breaker's `EffectChains`. Consumed after first use via `Once` — fires exactly once, then the node is permanently removed from the chain.

**Redesign note (C7)**: Now uses `Once` instead of a custom one-shot mechanism. The invisible wall is placed on the breaker's `EffectChains`, not as a global resource.
