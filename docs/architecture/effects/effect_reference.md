# Per-Effect Reference

Quick reference for all 30 `EffectType` variants: config struct, fire behavior, reversal, runtime systems, and category. For the type system, traits, and dispatch mechanics, see `core_types.md`. For adding new effects, see `adding_effects.md`.

## Categories

| Category | Pattern | Reversal |
|---|---|---|
| **Passive stack** | `fire` pushes onto `EffectStack<Config>`, recalculation system applies | `reverse` pops matching entry, `reverse_all_by_source` retains-not-matching |
| **Toggle** | `fire` inserts marker component | `reverse` removes component |
| **Spawned entity** | `fire` spawns a child entity with lifecycle systems | `reverse` despawns child |
| **Fire-and-forget** | `fire` spawns a request or deals damage; no persistent state | Not reversible |
| **Stateful counter** | `fire` inserts a counter component; bump/event decrements | `reverse` removes counter |
| **Meta** | `fire` picks a sub-effect and fires it | Not reversible |

## Passive Stack Effects

These are `Reversible` and use `EffectStack<Config>`.

| Effect | Config | Fields | Stacking | Fire | Reverse |
|---|---|---|---|---|---|
| **SpeedBoost** | `SpeedBoostConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, recalc bolt speed | Pop matching entry |
| **SizeBoost** | `SizeBoostConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, recalc entity scale | Pop matching entry |
| **DamageBoost** | `DamageBoostConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, read at damage time | Pop matching entry |
| **BumpForce** | `BumpForceConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, bump system reads | Pop matching entry |
| **QuickStop** | `QuickStopConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, breaker decel reads | Pop matching entry |
| **Piercing** | `PiercingConfig` | `charges: u32` | Additive | Push charges → `EffectStack`, decrement on destroy | Pop matching entry |
| **Vulnerable** | `VulnerableConfig` | `multiplier: OrderedFloat<f32>` | Multiplicative | Push multiplier → `EffectStack`, damage calc reads | Pop matching entry |
| **RampingDamage** | `RampingDamageConfig` | `increment: OrderedFloat<f32>` | Per-source accumulation | Insert `RampingDamageState` component with accumulator | Remove component |

## Toggle Effects

`Reversible` — insert/remove a marker or config component.

| Effect | Config | Fields | Fire | Reverse |
|---|---|---|---|---|
| **FlashStep** | `FlashStepConfig` | (unit struct) | Insert `FlashStepActive` marker on breaker | Remove marker |
| **Anchor** | `AnchorConfig` | `bump_force_multiplier`, `perfect_window_multiplier`, `plant_delay` (all `OrderedFloat<f32>`) | Insert `AnchorActive` config on breaker; `tick_anchor` manages `AnchorTimer` → `AnchorPlanted` state machine | Remove `AnchorActive`, `AnchorTimer`, `AnchorPlanted` |
| **Attraction** | `AttractionConfig` | `attraction_type: AttractionType`, `force: OrderedFloat<f32>`, `max_force: Option<OrderedFloat<f32>>` | Insert `AttractionActive` on bolt; `apply_attraction` steers bolt each tick | Remove `AttractionActive` |

## Spawned Entity Effects

`Reversible` — spawn a child entity with lifecycle. `reverse` despawns it.

| Effect | Config | Key fields | Fire | Tick system | Reverse |
|---|---|---|---|---|---|
| **Pulse** | `PulseConfig` | `base_range`, `range_per_level`, `stacks`, `speed`, `interval` | Spawn `PulseEmitter` entity that periodically fires shockwave rings | `tick_pulse` (interval countdown → spawn ring) | Despawn emitter |
| **Shield** | `ShieldConfig` | `duration`, `reflection_cost` | Spawn `ShieldWall` floor wall + `ShieldWallTimer`; if wall exists, reset timer | `tick_shield_wall_timer` → despawn on expiry | Despawn all `ShieldWall` entities |
| **SecondWind** | `SecondWindConfig` | (unit struct) | Spawn a `SecondWindWall` safety wall at playfield bottom (single use — despawns on bolt bounce) | None (wall physics handles it) | Despawn wall |
| **GravityWell** | `GravityWellConfig` | `strength`, `duration`, `radius`, `max` | Spawn gravity well entity at entity's position; if `max` reached, evict oldest | `tick_gravity_well` (duration countdown → despawn); `apply_gravity_pull` (steer bolts toward well each tick) | Despawn well |
| **SpawnPhantom** | `SpawnPhantomConfig` | `duration`, `max_active` | Spawn phantom bolt with limited lifespan; if `max_active` reached, evict oldest | Phantom bolt lifespan tick → despawn | Despawn phantom bolt |
| **CircuitBreaker** | `CircuitBreakerConfig` | `bumps_required`, `spawn_count`, `inherit`, `shockwave_range`, `shockwave_speed` | Insert `CircuitBreakerCounter`; each `PerfectBumped` decrements; on zero → fire `SpawnBolts` + `Shockwave`, reset counter | `tick_circuit_breaker` (counts bumps) | Remove counter |
| **EntropyEngine** | `EntropyConfig` | `max_effects`, `pool: Vec<...>` | Insert `EntropyEngineState`; tracks kills; on kill → weighted random pick from pool → fire sub-effect | `tick_entropy` (watches death triggers) | Remove state; reset per-node in `Reset` set |
| **TetherBeam** | `TetherBeamConfig` | `damage_mult`, `chain: bool`, `width` | Standard mode: spawn 2 tethered bolts + beam entity. Chain mode: link all active bolts with beam entities | `tick_tether_beam` (damage cells intersecting beam lines each tick) | Despawn beam entities |

## Fire-and-Forget Effects

**Not** in `ReversibleEffectType`. Cannot appear in `ScopedTree::Fire`.

| Effect | Config | Key fields | Fire |
|---|---|---|---|
| **Shockwave** | `ShockwaveConfig` | `base_range`, `range_per_level`, `stacks`, `speed` | Spawn expanding ring entity; damages cells within ring; each cell hit once; despawns at max radius |
| **Explode** | `ExplodeConfig` | `range`, `damage` | Instant AoE damage to all cells within range of entity position |
| **ChainLightning** | `ChainLightningConfig` | `arcs`, `range`, `damage_mult`, `arc_speed` | Spawn arc chain that jumps between nearby cells; each arc damages one target; arcs travel at `arc_speed` |
| **PiercingBeam** | `PiercingBeamConfig` | `damage_mult`, `width` | Spawn beam request entity along bolt velocity direction; single-tick damage to all cells in the beam's line |
| **SpawnBolts** | `SpawnBoltsConfig` | `count`, `lifespan: Option<OrderedFloat<f32>>`, `inherit: bool` | Spawn N extra bolts; `inherit` copies parent's `BoundEffects`; `lifespan` sets per-bolt timer |
| **ChainBolt** | `ChainBoltConfig` | `tether_distance: OrderedFloat<f32>` | Spawn a bolt tethered to parent via `DistanceConstraint` (from `rantzsoft_physics2d`) |
| **MirrorProtocol** | `MirrorConfig` | `inherit: bool` | Duplicate bolt on last impact side using `LastImpact` + `ImpactSide`; the clone continues in the mirrored direction |
| **LoseLife** | `LoseLifeConfig` | (unit struct) | Decrements `LivesCount`; run domain handles game-over if zero |
| **TimePenalty** | `TimePenaltyConfig` | `seconds: OrderedFloat<f32>` | Sends `ApplyTimePenalty` message; node domain subtracts from timer |
| **Die** | `DieConfig` | (unit struct) | Sends `KillYourself<T>` for the target entity → enters death pipeline |
| **RandomEffect** | `RandomEffectConfig` | `pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>` | Weighted random pick from pool → fire the selected sub-effect via `fire_dispatch` |

## Effect → Category Mapping

For quick lookup, all 30 effects sorted by category:

| Category | Effects |
|---|---|
| Passive stack | SpeedBoost, SizeBoost, DamageBoost, BumpForce, QuickStop, Piercing, Vulnerable, RampingDamage |
| Toggle | FlashStep, Anchor, Attraction |
| Spawned entity | Pulse, Shield, SecondWind, GravityWell, SpawnPhantom, CircuitBreaker, EntropyEngine, TetherBeam |
| Fire-and-forget | Shockwave, Explode, ChainLightning, PiercingBeam, SpawnBolts, ChainBolt, MirrorProtocol, LoseLife, TimePenalty, Die, RandomEffect |

## Reversible ↔ Non-Reversible

`ReversibleEffectType` (16 variants — can appear in `ScopedTree::Fire`):
SpeedBoost, SizeBoost, DamageBoost, BumpForce, QuickStop, FlashStep, Piercing, Vulnerable, RampingDamage, Attraction, Anchor, Pulse, Shield, SecondWind, CircuitBreaker, EntropyEngine

Non-reversible (14 variants — `Tree::Fire` only, never in scoped position):
Shockwave, Explode, ChainLightning, PiercingBeam, SpawnBolts, SpawnPhantom, ChainBolt, MirrorProtocol, TetherBeam, GravityWell, LoseLife, TimePenalty, Die, RandomEffect
