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
| Common | Basic | `Piercing(1)` | Pure utility |
| Uncommon | Keen | `Piercing(2)` | Enables multi-cell combos |
| Rare | Brutal | `Piercing(3), DamageBoost(0.1)` | Opens DamageBoost synergy path |

`max_taken: 3`

### Damage Boost
Fractional bonus damage per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `DamageBoost(0.1)` | Pure damage |
| Uncommon | Potent | `DamageBoost(0.2)` | Meaningful scaling |
| Rare | Savage | `DamageBoost(0.35)` | High-value target for damage builds |

`max_taken: 5`

### Bolt Speed Boost
Percentage-based bolt speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Slight | `SpeedBoost(target: Bolt, multiplier: 1.1)` | 10% speed boost |
| Uncommon | Swift | `SpeedBoost(target: Bolt, multiplier: 1.2)` | 20% speed boost |
| Rare | Blazing | `SpeedBoost(target: Bolt, multiplier: 1.3), DamageBoost(0.05)` | 30% speed + damage synergy |

`max_taken: 3`

### Chain Hit
Bolt chains to additional cells on hit.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Linked | `ChainHit(1)` | Single chain |
| Uncommon | Branching | `ChainHit(2)` | Multi-chain |
| Rare | Arcing | `ChainHit(3), DamageBoost(0.05)` | Chains + damage |

`max_taken: 3`

### Bolt Size Boost
Increases bolt radius by a fraction per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Broad | `SizeBoost(Bolt, 0.15)` | Wider hits |
| Uncommon | Expanded | `SizeBoost(Bolt, 0.25)` | Noticeably larger |
| Rare | Massive | `SizeBoost(Bolt, 0.4)` | Huge bolt — hard to miss cells |

`max_taken: 3`

### Attraction (Magnetism)
Bolt attracts nearby cells.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Faint | `Attraction(0.5)` | Slight pull |
| Uncommon | Steady | `Attraction(1.0)` | Noticeable attraction |
| Rare | Powerful | `Attraction(1.5), SizeBoost(Bolt, 0.1)` | Strong pull + size synergy |

`max_taken: 3`

### Breaker Speed Boost
Percentage-based breaker speed increase per stack. Stacks multiplicatively.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Quick | `SpeedBoost(target: Breaker, multiplier: 1.1)` | 10% speed boost |
| Uncommon | Agile | `SpeedBoost(target: Breaker, multiplier: 1.2)` | 20% speed boost |
| Rare | Lightning | `SpeedBoost(target: Breaker, multiplier: 1.3), BumpForce(5.0)` | 30% speed + bump force synergy |

`max_taken: 3`

### Bump Force
Flat bump force increase per stack.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Firm | `BumpForce(10.0)` | Slight force boost |
| Uncommon | Strong | `BumpForce(20.0)` | Noticeable |
| Rare | Crushing | `BumpForce(35.0)` | High force — enables speed-through-force builds |

`max_taken: 3`

### Tilt Control
Flat tilt control sensitivity increase.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Subtle | `TiltControl(0.1)` | Minor control |
| Uncommon | Precise | `TiltControl(0.2)` | Better aim |
| Rare | Masterful | `TiltControl(0.35)` | Precise aim + feels responsive |

`max_taken: 3`

## Named Chips

### Amp
Ramping damage bonus that stacks on cell hits, resets on non-bump breaker impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `OnSelected([RampingDamage(bonus_per_hit: 0.02, max_bonus: 0.2)])` | Mild ramp |
| Uncommon | Potent | `OnSelected([RampingDamage(bonus_per_hit: 0.04, max_bonus: 0.4)])` | Decent ramp |
| Rare | Savage | `OnSelected([RampingDamage(bonus_per_hit: 0.06, max_bonus: 0.6)])` | High ramp, rewards sustained combos |

`max_taken: 2`

### Augment
Breaker width increase + bump force boost.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `OnSelected([SizeBoost(Breaker, 6.0), BumpForce(8.0)])` | Mild utility |
| Uncommon | Sturdy | `OnSelected([SizeBoost(Breaker, 10.0), BumpForce(15.0)])` | Noticeable |
| Rare | Fortified | `OnSelected([SizeBoost(Breaker, 16.0), BumpForce(25.0), SpeedBoost(Breaker, 20.0)])` | Width + force + speed |

`max_taken: 2`

### Overclock
Timed speed burst after perfect bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `OnPerfectBump(TimedSpeedBurst(speed_mult: 1.3, duration_secs: 2.0))` | Short burst |
| Uncommon | Charged | `OnPerfectBump(TimedSpeedBurst(speed_mult: 1.5, duration_secs: 3.0))` | Longer, stronger |
| Rare | Supercharged | `OnPerfectBump(TimedSpeedBurst(speed_mult: 1.8, duration_secs: 4.0))` | Major burst |

`max_taken: 2`

### Flux
Randomness/instability — fires random effect from weighted pool on bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `OnBump([RandomEffect([(0.5, SpeedBoost(Bolt, 1.1)), (0.5, OnImpact(Cell, [Shockwave(base_range: 24, range_per_level: 0, stacks: 1)]))])])` | 2-effect pool |
| Uncommon | Volatile | `OnBump([RandomEffect([(0.35, SpeedBoost(Bolt, 1.15)), (0.35, OnImpact(Cell, [Shockwave(base_range: 32, range_per_level: 0, stacks: 1)])), (0.30, OnImpact(Cell, [ChainLightning(range: 64, jumps: 2)]))])])` | 3-effect pool |
| Rare | Critical | `OnBump([RandomEffect([(0.3, SpeedBoost(Bolt, 1.2)), (0.25, OnImpact(Cell, [Shockwave(base_range: 40, range_per_level: 0, stacks: 1)])), (0.25, OnImpact(Cell, [ChainLightning(range: 80, jumps: 3)])), (0.2, SpawnBolt)])])` | 4-effect pool — SpawnBolt opens multi-bolt synergy |

`max_taken: 2`

## Triggered Chips (Existing Overclocks)

### Surge
Speed boost on perfect bump.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Basic | `OnPerfectBump(SpeedBoost(Bolt, 1.2))` | Mild boost |
| Uncommon | Strong | `OnPerfectBump(SpeedBoost(Bolt, 1.35))` | Noticeable |
| Rare | Extreme | `OnPerfectBump(SpeedBoost(Bolt, 1.5))` | Major speed — synergizes with damage builds |

`max_taken: 3`

### Cascade
Shockwave on cell destruction.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `OnCellDestroyed(Shockwave(base_range: 20, range_per_level: 5, stacks: 1))` | Small wave |
| Uncommon | Spreading | `OnCellDestroyed(Shockwave(base_range: 30, range_per_level: 8, stacks: 1))` | Medium wave |
| Rare | Devastating | `OnCellDestroyed(Shockwave(base_range: 40, range_per_level: 12, stacks: 1))` | Large wave — evolution ingredient for Entropy Engine |

`max_taken: 3`

### Impact
Shockwave on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `OnPerfectBump(OnImpact(Cell, Shockwave(base_range: 24, range_per_level: 6, stacks: 1)))` | Conditional shockwave |
| Uncommon | Strong | `OnPerfectBump(OnImpact(Cell, Shockwave(base_range: 36, range_per_level: 10, stacks: 1)))` | Better range |
| Rare | Devastating | `OnPerfectBump(OnImpact(Cell, Shockwave(base_range: 48, range_per_level: 14, stacks: 1)))` | Large AoE on precision play |

`max_taken: 3`

### Tether
Chain bolt on perfect bump cell impact.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Short | `OnPerfectBump(OnImpact(Cell, ChainBolt(tether_distance: 80)))` | Short tether |
| Uncommon | Extended | `OnPerfectBump(OnImpact(Cell, ChainBolt(tether_distance: 120)))` | Medium tether |
| Rare | Long | `OnPerfectBump(OnImpact(Cell, ChainBolt(tether_distance: 160)))` | Long tether — wide coverage |

`max_taken: 2`

### Aftershock
Shockwave on bolt-wall bounce.

| Rarity | Prefix | Effects | Synergy Notes |
|--------|--------|---------|---------------|
| Common | Minor | `OnImpact(Wall, Shockwave(base_range: 16, range_per_level: 4, stacks: 1))` | Small wall shockwave |
| Uncommon | Rumbling | `OnImpact(Wall, Shockwave(base_range: 24, range_per_level: 6, stacks: 1))` | Medium |
| Rare | Thundering | `OnImpact(Wall, Shockwave(base_range: 32, range_per_level: 10, stacks: 1))` | Large wall shockwave |

`max_taken: 3`

## Legendaries

All legendaries are `max_taken: 1`. Template has only the `legendary` slot filled.

### Ricochet Protocol
Wall-bank 3x damage.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `OnImpact(Wall, OnImpact(Cell, DamageBoost(2.0)))` | Rewards precise wall-bank shots. Build-around: pair with wall-impact chips. |

### Glass Cannon
2x damage, 30% narrow breaker.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | `OnSelected([DamageBoost(1.0), SizeBoost(Breaker, -0.3)])` | High risk / high reward. Smaller breaker = harder to hit. |

### Deadline
Timer pressure = bolt speed + trail.

| Rarity | Effects | Design Notes |
|--------|---------|-------------|
| Legendary | Conditional: when timer < 25%, bolt speed 2x + visual trail | Rewards playing on the edge. Needs custom handler — `TimePressureBoost`. |

## Evolutions

See `docs/design/evolutions.md` for evolution design principles.

### Entropy Engine
**Ingredients**: Cascade (Rare) + Flux (Rare)

**Effect**: `OnCellDestroyed([EntropyEngine(5, [(0.3, SpawnBolt), (0.25, Shockwave(base_range: 48, range_per_level: 0, stacks: 1)), (0.25, ChainLightning(range: 80, jumps: 3)), (0.20, SpeedBoost(Bolt, 1.3))])])`

Every 5th cell destroyed, roll from weighted pool. Cascade provides the trigger domain, Flux provides the randomness mechanic, evolution adds the counter gate.

### (Existing Evolutions)
The 8 existing evolutions (Nova Lance, Voltchain, Phantom Breaker, Supernova, Dead Man's Hand, Railgun, Gravity Well, Second Wind) retain their current recipes and effects.
