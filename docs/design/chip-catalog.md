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
| Rare | Brutal | `On(Bolt) → Do(Piercing(3)), Do(DamageBoost(0.1))` | Opens DamageBoost synergy path |

`max_taken: 3`

### Damage Boost
Fractional bonus damage per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Bolt) → Do(DamageBoost(0.1))` | Pure damage |
| Uncommon | Potent | `On(Bolt) → Do(DamageBoost(0.2))` | Meaningful scaling |
| Rare | Savage | `On(Bolt) → Do(DamageBoost(0.35))` | High-value target for damage builds |

`max_taken: 5`

### Bolt Speed
Percentage-based bolt speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `On(Bolt) → Do(SpeedBoost(multiplier: 1.1))` | 10% speed boost |
| Uncommon | Swift | `On(Bolt) → Do(SpeedBoost(multiplier: 1.2))` | 20% speed boost |
| Rare | Blazing | `On(Bolt) → Do(SpeedBoost(multiplier: 1.3)), Do(DamageBoost(0.05))` | 30% speed + damage synergy |

`max_taken: 3`

### Chain Hit
Bolt chains to additional cells on hit.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Linked | `On(Bolt) → Do(ChainHit(1))` | Single chain |
| Uncommon | Branching | `On(Bolt) → Do(ChainHit(2))` | Multi-chain |
| Rare | Arcing | `On(Bolt) → Do(ChainHit(3)), Do(DamageBoost(0.05))` | Chains + damage |

`max_taken: 3`

### Bolt Size
Increases bolt radius by a fraction per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Broad | `On(Bolt) → Do(SizeBoost(0.15))` | Wider hits |
| Uncommon | Expanded | `On(Bolt) → Do(SizeBoost(0.25))` | Noticeably larger |
| Rare | Massive | `On(Bolt) → Do(SizeBoost(0.4))` | Huge bolt — hard to miss cells |

`max_taken: 3`

### Magnetism
Bolt attracts toward nearest entity of the given type. See [effects/attraction.md](effects/attraction.md) for Attraction mechanics.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Faint | `On(Bolt) → Do(Attraction(Cell, 0.5))` | Slight pull toward cells |
| Uncommon | Steady | `On(Bolt) → Do(Attraction(Cell, 1.0))` | Noticeable attraction |
| Rare | Powerful | `On(Bolt) → Do(Attraction(Cell, 1.5)), Do(SizeBoost(0.1))` | Strong pull + size synergy |

`max_taken: 3`

### Breaker Speed
Percentage-based breaker speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Quick | `On(Breaker) → Do(SpeedBoost(multiplier: 1.1))` | 10% speed boost |
| Uncommon | Agile | `On(Breaker) → Do(SpeedBoost(multiplier: 1.2))` | 20% speed boost |
| Rare | Lightning | `On(Breaker) → Do(SpeedBoost(multiplier: 1.3)), Do(BumpForce(5.0))` | 30% speed + bump force synergy |

`max_taken: 3`

### Bump Force
Flat bump force increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Firm | `On(Breaker) → Do(BumpForce(10.0))` | Slight force boost |
| Uncommon | Strong | `On(Breaker) → Do(BumpForce(20.0))` | Noticeable |
| Rare | Crushing | `On(Breaker) → Do(BumpForce(35.0))` | High force — enables speed-through-force builds |

`max_taken: 3`

### Tilt Control
Flat tilt control sensitivity increase.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Subtle | `On(Bolt) → Do(TiltControl(0.1))` | Minor control |
| Uncommon | Precise | `On(Bolt) → Do(TiltControl(0.2))` | Better aim |
| Rare | Masterful | `On(Bolt) → Do(TiltControl(0.35))` | Precise aim + feels responsive |

`max_taken: 3`

### Wide Breaker
Flat breaker width increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `On(Breaker) → Do(SizeBoost(10.0))` | Slightly wider |
| Uncommon | Sturdy | `On(Breaker) → Do(SizeBoost(20.0))` | Noticeably wider |
| Rare | Massive | `On(Breaker) → Do(SizeBoost(30.0))` | Very wide — safety at the cost of a chip slot |

`max_taken: 3`

## Named Chips

### Amp
Ramping damage bonus that stacks on cell hits.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Bolt) → Do(RampingDamage(bonus_per_hit: 0.02))` | Mild ramp |
| Uncommon | Potent | `On(Bolt) → Do(RampingDamage(bonus_per_hit: 0.04))` | Decent ramp |
| Rare | Savage | `On(Bolt) → Do(RampingDamage(bonus_per_hit: 0.06))` | High ramp, rewards sustained combos |

`max_taken: 2`

### Augment
Breaker width increase + bump force boost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `On(Breaker) → Do(SizeBoost(6.0)), Do(BumpForce(8.0))` | Mild utility |
| Uncommon | Sturdy | `On(Breaker) → Do(SizeBoost(10.0)), Do(BumpForce(15.0))` | Noticeable |
| Rare | Fortified | `On(Breaker) → Do(SizeBoost(16.0)), Do(BumpForce(25.0)), Do(SpeedBoost(multiplier: 1.15))` | Width + force + speed |

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
| Common | Minor | `On(Breaker) → When(OnBump) → Do(RandomEffect([(0.5, SpeedBoost(1.1)), (0.5, Shockwave(base_range: 24))]))` | 2-effect pool |
| Uncommon | Volatile | `On(Breaker) → When(OnBump) → Do(RandomEffect([(0.35, SpeedBoost(1.15)), (0.35, Shockwave(base_range: 32)), (0.30, ChainBolt(tether: 100))]))` | 3-effect pool |
| Rare | Critical | `On(Breaker) → When(OnBump) → Do(RandomEffect([(0.3, SpeedBoost(1.2)), (0.25, Shockwave(base_range: 40)), (0.25, ChainBolt(tether: 120)), (0.2, SpawnBolts())]))` | 4-effect pool — SpawnBolts opens multi-bolt synergy |

`max_taken: 2`

### Last Stand
Bolt-lost redemption — breaker speeds up when a bolt is lost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `On(Breaker) → When(OnBoltLost) → Do(SpeedBoost(multiplier: 1.15))` | Mild comfort |
| Uncommon | Strong | `On(Breaker) → When(OnBoltLost) → Do(SpeedBoost(multiplier: 1.3))` | Meaningful recovery |
| Rare | Desperate | `On(Breaker) → When(OnBoltLost) → Do(SpeedBoost(multiplier: 1.5))` | Major boost — rewards surviving bolt loss |

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
| Uncommon | Quick | `On(Breaker) → When(OnPerfectBump) → Do(SpawnBolts())` | Extra bolt on precision |
| Rare | Lightning | `On(Breaker) → When(OnPerfectBump) → Do(SpawnBolts()), Do(SpeedBoost(multiplier: 1.2))` | Bolt spawn + speed burst |

`max_taken: 1`

## Legendaries

All legendaries are `max_taken: 1`. Template has only the `legendary` slot filled.

### Ricochet Protocol
Wall-bank damage boost until next cell hit.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(Impacted(Wall)) → Until(Impacted(Cell)) → Do(DamageBoost(2.0))` | Wall bounce grants +200% damage until next cell hit. Rewards precise wall-bank shots. |

### Glass Cannon
Doubled damage, smaller bolt.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(DamageBoost(1.0)), Do(SizeBoost(-0.3))` | +100% damage but 30% smaller bolt. High risk / high reward. |

### Desperation
Breaker speeds up when a bolt is lost.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Breaker) → When(OnBoltLost) → Do(SpeedBoost(multiplier: 2.0))` | 2x breaker speed on bolt loss. Snowballing risk/reward. |

### Deadline
Timer pressure = bolt speed.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(OnNodeTimerThreshold(0.25)) → Do(SpeedBoost(multiplier: 2.0))` | When timer drops below 25%, bolt speed 2x. Rewards playing on the edge. |

### Whiplash
Whiff redemption — miss a bump, but the next cell impact gets bonus damage + shockwave.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → When(OnBumpWhiff) → When(Impacted(Cell)) → Once([Do(DamageBoost(1.5))]), Do(Shockwave(base_range: 64, speed: 500))` | Turns whiffs into comebacks. DamageBoost fires once via `Once`, shockwave fires every time. |

### Singularity
Small bolt, big damage, fast.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(SizeBoost(-0.4)), Do(DamageBoost(1.5)), Do(SpeedBoost(multiplier: 1.4))` | Tiny bolt that hits like a truck. Hard to control, massive payoff. |

### Gauntlet
Large bolt, fast, weak hits.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `On(Bolt) → Do(SizeBoost(0.5)), Do(SpeedBoost(multiplier: 1.4)), Do(DamageBoost(-0.5))` | 50% larger bolt, 40% faster, but halved damage. Safety-focused build-around. |

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

## Evolutions

See `docs/design/evolutions.md` for evolution design principles. Evolution RON files are in `assets/chips/evolution/`.

### Entropy Engine
**Ingredients**: Cascade x1 + Flux x1

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(EntropyEngine(threshold: 5, pool: [(0.3, SpawnBolts()), (0.25, Shockwave(base_range: 48, speed: 400)), (0.25, ChainBolt(tether: 120)), (0.20, SpeedBoost(1.3))]))`

Every 5th cell destroyed, roll from weighted pool. Cascade provides the trigger domain, Flux provides the randomness mechanic, evolution adds the counter gate.

### Nova Lance
**Ingredients**: Damage Boost x2 + Bolt Speed x2

**Effect**: `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → Do(Shockwave(base_range: 128, speed: 600))`

Perfect bumps unleash devastating shockwaves on cell impact.

### Voltchain
**Ingredients**: Chain Hit x2 + Damage Boost x2

**Effect**: `On(Bolt) → When(DestroyedCell) → Do(ChainLightning(arcs: 3, range: 96, damage_mult: 0.5))`

Destroying cells unleashes chain lightning to nearby targets.

### Phantom Breaker
**Ingredients**: Wide Breaker x2 + Bump Force x2

**Effect**: `On(Breaker) → When(OnBump) → Do(SpawnPhantom(duration: 5.0, max_active: 1))`

Successful bumps summon a phantom breaker that mirrors your moves.

### Supernova
**Ingredients**: Piercing Shot x3 + Surge x1

**Effect**: `On(Bolt) → When(PerfectBumped) → When(Impacted(Cell)) → When(DestroyedCell) → Do(MultiBolt(base_count: 2, count_per_level: 0, stacks: 1)), Do(Shockwave(base_range: 96, speed: 400))`

Perfect bumps trigger chain explosions — cells destroyed spawn bolts and shockwaves.

### Dead Man's Hand
**Ingredients**: Damage Boost x3 + Last Stand x1

**Effect**: `On(Breaker) → When(OnBoltLost) → Do(Shockwave(base_range: 128, speed: 500)), Do(SpeedBoost(multiplier: 1.5))`

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

**Effect**: `On(Breaker) → When(OnBoltLost) → Do(SecondWind(invuln_secs: 3.0))`

Bolt loss grants temporary invulnerability — cheat death.
