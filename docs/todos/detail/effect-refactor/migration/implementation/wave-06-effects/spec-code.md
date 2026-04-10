## Implementation Spec: Effect — All 30 Effects + Tick Systems + Conditions

### Prerequisites

This wave depends on the completion of:
- **Wave 2 (Scaffold)**: All types, traits, components, resources, stubs, and plugin wiring must exist. In particular, `EffectSystems::Bridge`, `EffectSystems::Tick`, `EffectSystems::Conditions`, and `EffectSystems::Reset` system sets must exist in `src/effect/sets.rs`.
- **Wave 3 (RON Assets)**: RON asset files must use new syntax.
- **Wave 4 (Functions)**: `EffectStack<T>` methods (push, remove, aggregate), walking algorithm, dispatch functions, command extensions, and passive effect trait definitions (`Fireable`, `Reversible`, `PassiveEffect`) must be implemented.
- **Wave 5 (Triggers)**: All trigger bridge systems and game systems must be implemented. This includes `tick_effect_timers` and `check_node_timer_thresholds` (which are wave 5 scope, NOT wave 6).

Do NOT start wave 6 until waves 2-5 are complete.

### Domain
`src/effect/`

### Failing Tests
- `src/effect/effects/speed_boost/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/size_boost/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/damage_boost/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/bump_force/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/quick_stop/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/vulnerable/tests.rs` — passive fire/reverse/aggregate tests
- `src/effect/effects/piercing/tests.rs` — passive fire/reverse/aggregate + PiercingRemaining tests
- `src/effect/effects/ramping_damage/tests.rs` — passive fire/reverse/aggregate + RampingDamageAccumulator tests
- `src/effect/effects/lose_life/tests.rs` — fire sends DamageDealt<Breaker> test
- `src/effect/effects/time_penalty/tests.rs` — fire sends ApplyTimePenalty test
- `src/effect/effects/die/tests.rs` — fire sends KillYourself<T> test
- `src/effect/effects/spawn_bolts/tests.rs` — fire spawns bolts tests
- `src/effect/effects/chain_bolt/tests.rs` — fire spawns tethered bolt test
- `src/effect/effects/mirror_protocol/tests.rs` — fire spawns mirrored bolt tests
- `src/effect/effects/random_effect/tests.rs` — fire delegates to random selection tests
- `src/effect/effects/explode/tests.rs` — fire damages cells in range test
- `src/effect/effects/piercing_beam/tests.rs` — fire damages cells in beam test
- `src/effect/effects/shockwave/tests.rs` — fire spawns shockwave entity + tick/sync/damage/despawn tests
- `src/effect/effects/chain_lightning/tests.rs` — fire spawns chain entity + tick state machine tests
- `src/effect/effects/anchor/tests.rs` — fire/reverse + tick plant/unplant state machine tests
- `src/effect/effects/attraction/tests.rs` — fire/reverse + apply attraction force tests
- `src/effect/effects/pulse/tests.rs` — fire/reverse + tick periodic emission tests
- `src/effect/effects/shield/tests.rs` — fire/reverse + tick countdown + reflection cost tests
- `src/effect/effects/second_wind/tests.rs` — fire/reverse tests
- `src/effect/effects/flash_step/tests.rs` — fire/reverse toggle tests
- `src/effect/effects/circuit_breaker/tests.rs` — fire counter + reward tests
- `src/effect/effects/entropy_engine/tests.rs` — fire counter + random pool tests
- `src/effect/effects/gravity_well/tests.rs` — fire spawns well entity + tick force + despawn tests
- `src/effect/effects/phantom_bolt/tests.rs` — fire spawns phantom entity + tick lifetime + despawn tests (note: directory name is `phantom_bolt` but the config struct is `SpawnPhantomConfig`)
- `src/effect/effects/tether_beam/tests.rs` — fire spawns beam entity + tick damage + cleanup tests
- `src/effect/effects/ramping_damage/tests.rs` — reset accumulator on node start test (shares file with passive tests)
- `src/effect/conditions/evaluate_conditions/tests.rs` — condition transition fire/reverse tests
- `src/effect/conditions/node_active.rs` — NodeActive evaluator tests (inline)
- `src/effect/conditions/shield_active.rs` — ShieldActive evaluator tests (inline)
- `src/effect/conditions/combo_active.rs` — ComboActive evaluator tests (inline)
- `src/effect/dispatch/fire_dispatch.rs` — 30 match arm dispatch tests
- `src/effect/dispatch/reverse_dispatch.rs` — 16 match arm dispatch tests

### What to Implement

#### Trait Definitions (if not already present from prior waves)

- `Fireable` trait: `fn fire(&self, entity: Entity, source: &str, world: &mut World)` + `fn register(app: &mut App) {}` default
- `Reversible` trait: extends `Fireable`, `fn reverse(&self, entity: Entity, source: &str, world: &mut World)`
- `PassiveEffect` trait: extends `Fireable + Reversible + Sized + Clone + PartialEq + Eq`, `fn aggregate(entries: &[(String, Self)]) -> f32`

#### EffectStack<T> (if not already present from prior waves)

- `EffectStack<T: PassiveEffect>`: Component with `entries: Vec<(String, T)>`, methods `push`, `remove`, `aggregate`

---

#### GROUP A: Passive Effects (8 configs)

Each passive effect follows the identical pattern. Implement `Fireable`, `Reversible`, and `PassiveEffect` for each.

**1. SpeedBoostConfig**
- File: `src/effect/effects/speed_boost/config.rs`
- Struct: `SpeedBoostConfig { multiplier: OrderedFloat<f32> }`
- Derives: `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`
- `Fireable::fire`: get or insert `EffectStack<SpeedBoostConfig>`, call `stack.push(source.to_string(), self.clone())`
- `Reversible::reverse`: get `EffectStack<SpeedBoostConfig>`, call `stack.remove(source, self)`
- `PassiveEffect::aggregate`: product of all `multiplier.0` values; identity = 1.0
- No `register` override (default no-op)

**2. SizeBoostConfig**
- File: `src/effect/effects/size_boost/config.rs`
- Struct: `SizeBoostConfig { multiplier: OrderedFloat<f32> }`
- Same pattern as SpeedBoostConfig. Aggregate = product. Identity = 1.0.

**3. DamageBoostConfig**
- File: `src/effect/effects/damage_boost/config.rs`
- Struct: `DamageBoostConfig { multiplier: OrderedFloat<f32> }`
- Same pattern. Aggregate = product. Identity = 1.0.

**4. BumpForceConfig**
- File: `src/effect/effects/bump_force/config.rs`
- Struct: `BumpForceConfig { multiplier: OrderedFloat<f32> }`
- Same pattern. Aggregate = product. Identity = 1.0.

**5. QuickStopConfig**
- File: `src/effect/effects/quick_stop/config.rs`
- Struct: `QuickStopConfig { multiplier: OrderedFloat<f32> }`
- Same pattern. Aggregate = product. Identity = 1.0.

**6. VulnerableConfig**
- File: `src/effect/effects/vulnerable/config.rs`
- Struct: `VulnerableConfig { multiplier: OrderedFloat<f32> }`
- Same pattern. Aggregate = product. Identity = 1.0.

**7. PiercingConfig**
- File: `src/effect/effects/piercing/config.rs`
- Struct: `PiercingConfig { charges: u32 }`
- `Fireable::fire`: get or insert `EffectStack<PiercingConfig>`, push `(source, config)`. Calculate new aggregate (sum of charges). If `PiercingRemaining` not present, insert `PiercingRemaining(aggregate)`.
- `Reversible::reverse`: get `EffectStack<PiercingConfig>`, remove matching entry. Recalculate aggregate. If aggregate == 0, remove `PiercingRemaining`.
- `PassiveEffect::aggregate`: sum of all `charges` values; identity = 0
- Component: `PiercingRemaining(pub u32)` in `src/effect/effects/piercing/components.rs`

**8. RampingDamageConfig**
- File: `src/effect/effects/ramping_damage/config.rs`
- Struct: `RampingDamageConfig { increment: OrderedFloat<f32> }`
- `Fireable::fire`: get or insert `EffectStack<RampingDamageConfig>`, push `(source, config)`. If `RampingDamageAccumulator` not present, insert `RampingDamageAccumulator(OrderedFloat(0.0))`.
- `Reversible::reverse`: get stack, remove entry. If stack empty, remove `RampingDamageAccumulator`.
- `PassiveEffect::aggregate`: sum of all `increment.0` values; identity = 0.0
- Component: `RampingDamageAccumulator(pub OrderedFloat<f32>)` in `src/effect/effects/ramping_damage/components.rs`

---

#### GROUP B: Message/Meta Effects (4 configs, fire-and-forget, NOT reversible)

**9. LoseLifeConfig**
- File: `src/effect/effects/lose_life/config.rs`
- Struct: `LoseLifeConfig {}` (empty)
- Derives: `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`
- `Fireable::fire`: send `DamageDealt<Breaker>` message with `dealer: None`, `target: entity`, `amount: 1.0`, `source_chip: Some(source.to_string())`
- Does NOT implement `Reversible`
- No `register` override

**10. TimePenaltyConfig**
- File: `src/effect/effects/time_penalty/config.rs`
- Struct: `TimePenaltyConfig { seconds: OrderedFloat<f32> }`
- `Fireable::fire`: send `ApplyTimePenalty { seconds: self.seconds.0 }` message
- Does NOT implement `Reversible`
- No `register` override

**11. DieConfig**
- File: `src/effect/effects/die/config.rs`
- Struct: `DieConfig {}` (empty)
- `Fireable::fire`: inspect entity components to determine `GameEntity` type (Bolt, Cell, Wall, Breaker). Send appropriate `KillYourself<T>` message. If no game entity component, do nothing.
- Does NOT implement `Reversible`
- No `register` override

**12. RandomEffectConfig**
- File: `src/effect/effects/random_effect/config.rs`
- Struct: `RandomEffectConfig { pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`
- `Fireable::fire`: weighted random selection from pool. Call selected effect's `config.fire(entity, source, world)`. If pool empty, do nothing.
- Does NOT implement `Reversible`
- No `register` override

---

#### GROUP C: Toggle Effect (1 config)

**13. FlashStepConfig**
- File: `src/effect/effects/flash_step/config.rs`
- Struct: `FlashStepConfig {}` (empty)
- `Fireable::fire`: insert `FlashStepActive` marker on entity. If already present, do nothing.
- `Reversible::reverse`: remove `FlashStepActive` from entity. If not present, do nothing.
- Component: `FlashStepActive` (marker, `#[derive(Component)]`) in `src/effect/effects/flash_step/components.rs`
- No `register` override

---

#### GROUP D: Protector Effects (3 configs, reversible, spawn protective entities)

**14. ShieldConfig**
- File: `src/effect/effects/shield/config.rs`
- Struct: `ShieldConfig { duration: OrderedFloat<f32>, reflection_cost: OrderedFloat<f32> }`
- `Fireable::fire`: spawn a `ShieldWall` entity with `ShieldDuration(duration.0)`, `ShieldReflectionCost(reflection_cost.0)`, `ShieldOwner(entity)`, `CleanupOnExit<NodeState>`, positioned at bottom of playfield
- `Reversible::reverse`: query all `ShieldWall` entities with `ShieldOwner` matching entity, despawn them
- `register`: register `tick_shield_duration` in `EffectSystems::Tick`
- Components in `src/effect/effects/shield/components.rs`:
  - `ShieldWall` — marker
  - `ShieldOwner(pub Entity)`
  - `ShieldDuration(pub f32)`
  - `ShieldReflectionCost(pub f32)`

**15. SecondWindConfig**
- File: `src/effect/effects/second_wind/config.rs`
- Struct: `SecondWindConfig {}` (empty)
- `Fireable::fire`: spawn an invisible one-shot wall entity at bottom of playfield with `SecondWindWall`, `SecondWindOwner(entity)`, `CleanupOnExit<NodeState>`
- `Reversible::reverse`: query all `SecondWindWall` entities with `SecondWindOwner` matching entity, despawn them
- No `register` override (wall collision handles the bounce, Die handles despawn)
- Components in `src/effect/effects/second_wind/components.rs`:
  - `SecondWindWall` — marker
  - `SecondWindOwner(pub Entity)`

**16. PulseConfig**
- File: `src/effect/effects/pulse/config.rs`
- Struct: `PulseConfig { base_range: OrderedFloat<f32>, range_per_level: OrderedFloat<f32>, stacks: u32, speed: OrderedFloat<f32>, interval: OrderedFloat<f32> }`
- `Fireable::fire`: insert `PulseEmitter` on entity with config values. Initialize timer to `interval.0`.
- `Reversible::reverse`: remove `PulseEmitter` from entity. Active shockwaves already spawned continue to completion.
- `register`: register `tick_pulse` in `EffectSystems::Tick`
- Component in `src/effect/effects/pulse/components.rs`:
  - `PulseEmitter { base_range: f32, range_per_level: f32, stacks: u32, speed: f32, interval: f32, timer: f32 }`

---

#### GROUP E: Stateful Effects (4 configs, reversible, internal state tracking)

**17. AnchorConfig**
- File: `src/effect/effects/anchor/config.rs`
- Struct: `AnchorConfig { bump_force_multiplier: OrderedFloat<f32>, perfect_window_multiplier: OrderedFloat<f32>, plant_delay: OrderedFloat<f32> }`
- `Fireable::fire`: insert `AnchorActive { source: source.to_string(), bump_force_multiplier: self.bump_force_multiplier.0, perfect_window_multiplier: self.perfect_window_multiplier.0, plant_delay: self.plant_delay.0 }` on entity
- `Reversible::reverse`: remove `AnchorActive`, `AnchorTimer`, `AnchorPlanted` from entity. If `AnchorPlanted` was present, also remove the bump force boost from `EffectStack<BumpForceConfig>` using the source string stored in `AnchorActive.source`.
- `register`: register `tick_anchor` in `EffectSystems::Tick`
- Components in `src/effect/effects/anchor/components.rs`:
  - `AnchorActive { source: String, bump_force_multiplier: f32, perfect_window_multiplier: f32, plant_delay: f32 }` — the `source` field stores the chip source string so that `tick_anchor` can push/remove `BumpForceConfig` entries with the correct source key
  - `AnchorTimer(pub f32)`
  - `AnchorPlanted` — marker

**18. AttractionConfig**
- File: `src/effect/effects/attraction/config.rs`
- Struct: `AttractionConfig { attraction_type: AttractionType, force: OrderedFloat<f32>, max_force: Option<OrderedFloat<f32>> }`
- `Fireable::fire`: get or insert `ActiveAttractions` on entity. Push `AttractionEntry { source: source.to_string(), attraction_type: self.attraction_type, force: self.force.0, max_force: self.max_force.map(|f| f.0) }`.
- `Reversible::reverse`: find and remove the matching `AttractionEntry` by `(source, config match)`. If Vec empty, remove `ActiveAttractions`.
- `register`: register `apply_attraction` in `EffectSystems::Tick`
- Components in `src/effect/effects/attraction/components.rs`:
  - `ActiveAttractions(pub Vec<AttractionEntry>)` — NOT an EffectStack, custom vec storage
  - `AttractionEntry { source: String, attraction_type: AttractionType, force: f32, max_force: Option<f32> }`

**19. CircuitBreakerConfig**
- File: `src/effect/effects/circuit_breaker/config.rs`
- Struct: `CircuitBreakerConfig { bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: OrderedFloat<f32>, shockwave_speed: OrderedFloat<f32> }`
- `Fireable::fire`: designed to be inside `When(PerfectBumped, ...)`. If `CircuitBreakerCounter` present, decrement `remaining`. If reaches 0: fire reward (SpawnBoltsConfig + ShockwaveConfig fire calls), reset to `bumps_required`. If absent: insert with `remaining = bumps_required - 1` (first bump counts). If `bumps_required == 1`: fire reward immediately and reset.
- `Reversible::reverse`: remove `CircuitBreakerCounter` from entity
- No `register` override (all logic in fire/reverse)
- Component in `src/effect/effects/circuit_breaker/components.rs`:
  - `CircuitBreakerCounter { remaining: u32, bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: f32, shockwave_speed: f32 }`

**20. EntropyConfig**
- File: `src/effect/effects/entropy_engine/config.rs`
- Struct: `EntropyConfig { max_effects: u32, pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`
- `Fireable::fire`: designed to be inside `When(Killed(Cell), ...)`. If `EntropyCounter` present, increment `count` (capped at `max_effects`). If absent, insert with `count = 1`. For `count` times: pick weighted random from pool and call its fire. EntropyEngine resets its counter internally when fired — the counter resets to 0 after firing all effects.
- `Reversible::reverse`: remove `EntropyCounter` from entity
- No `register` override — EntropyEngine resets internally when fired, so no separate reset system is needed
- Component in `src/effect/effects/entropy_engine/components.rs`:
  - `EntropyCounter { count: u32, max_effects: u32, pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`

---

#### GROUP F: Spawner/Area Effects (10 configs, NOT reversible)

**21. ShockwaveConfig**
- File: `src/effect/effects/shockwave/config.rs`
- Struct: `ShockwaveConfig { base_range: OrderedFloat<f32>, range_per_level: OrderedFloat<f32>, stacks: u32, speed: OrderedFloat<f32> }`
- `Fireable::fire`:
  1. Calculate effective range: `base_range.0 + range_per_level.0 * (stacks - 1) as f32`
  2. Read source entity position
  3. Snapshot damage multiplier from `EffectStack<DamageBoostConfig>.aggregate()` and base damage from `BoltBaseDamage` component
  4. Spawn entity with: `ShockwaveSource`, `ShockwaveRadius(0.0)`, `ShockwaveMaxRadius(effective_range)`, `ShockwaveSpeed(speed.0)`, `ShockwaveBaseDamage(base_damage)`, `ShockwaveDamageMultiplier(snapshotted_multiplier)`, `EffectSourceChip(source_chip_attribution)`, `ShockwaveDamaged(HashSet::new())`, `CleanupOnExit<NodeState>`
- `register`: register the 4 chained shockwave systems in `EffectSystems::Tick`
- Components in `src/effect/effects/shockwave/components.rs`:
  - `ShockwaveSource` — marker
  - `ShockwaveRadius(pub f32)`
  - `ShockwaveMaxRadius(pub f32)`
  - `ShockwaveSpeed(pub f32)`
  - `ShockwaveDamaged(pub HashSet<Entity>)`
  - `ShockwaveBaseDamage(pub f32)`
  - `ShockwaveDamageMultiplier(pub f32)`

**22. ExplodeConfig**
- File: `src/effect/effects/explode/config.rs`
- Struct: `ExplodeConfig { range: OrderedFloat<f32>, damage: OrderedFloat<f32> }`
- `Fireable::fire`: read source entity position. Query quadtree for cells within `range.0`. Send `DamageDealt<Cell>` with `damage.0` as flat damage per cell. Spawn short-lived VFX entity.
- Does NOT use damage boosts — damage is flat from config
- No `register` override (instant, fully resolved in fire)

**23. ChainLightningConfig**
- File: `src/effect/effects/chain_lightning/config.rs`
- Struct: `ChainLightningConfig { arcs: u32, range: OrderedFloat<f32>, damage_mult: OrderedFloat<f32>, arc_speed: OrderedFloat<f32> }`
- `Fireable::fire`:
  1. Read source entity position
  2. Snapshot damage: `damage_mult.0 * BoltBaseDamage * EffectStack<DamageBoostConfig>.aggregate()`
  3. Query quadtree for cells within `range.0`
  4. Pick random target, send `DamageDealt<Cell>` for first target immediately
  5. If `arcs > 1`: spawn `ChainLightningChain` entity with `remaining_jumps: arcs - 1`, snapshotted damage, `hit_set` with first target, `state: ChainState::Idle`, `range: range.0`, `arc_speed: arc_speed.0`, `source_pos` = first target position, `CleanupOnExit<NodeState>`
- `register`: register `tick_chain_lightning` in `EffectSystems::Tick`
- Components in `src/effect/effects/chain_lightning/components.rs`:
  - `ChainLightningChain { remaining_jumps: u32, damage: f32, hit_set: HashSet<Entity>, state: ChainState, range: f32, arc_speed: f32, source_pos: Vec2 }`
  - `ChainState` enum: `Idle`, `ArcTraveling { target: Entity, target_pos: Vec2, arc_entity: Entity, arc_pos: Vec2 }`

**24. PiercingBeamConfig**
- File: `src/effect/effects/piercing_beam/config.rs`
- Struct: `PiercingBeamConfig { damage_mult: OrderedFloat<f32>, width: OrderedFloat<f32> }`
- `Fireable::fire`: read source position and velocity direction. Calculate beam rectangle. Query quadtree for cells in beam. Send `DamageDealt<Cell>` per cell with `damage_mult.0 * BoltBaseDamage * EffectStack<DamageBoostConfig>.aggregate()`. Spawn short-lived VFX beam.
- No `register` override (instant)

**25. SpawnBoltsConfig**
- File: `src/effect/effects/spawn_bolts/config.rs`
- Struct: `SpawnBoltsConfig { count: u32, lifespan: Option<OrderedFloat<f32>>, inherit: bool }`
- `Fireable::fire`: read source entity position. Spawn `count` bolts at source position with random velocity directions. Mark each as `ExtraBolt`. If `lifespan` is Some, attach lifespan timer. If `inherit` is true, copy first primary bolt's `BoundEffects` onto each spawned bolt. Does NOT inherit `StagedEffects`.
- No `register` override

**26. SpawnPhantomConfig**
- File: `src/effect/effects/phantom_bolt/config.rs`
- Struct: `SpawnPhantomConfig { duration: OrderedFloat<f32>, max_active: u32 }`
- `Fireable::fire`: read source position. Query existing `PhantomBolt` entities with `PhantomOwner` matching source. If count >= `max_active`, despawn oldest by `PhantomLifetime`. Spawn phantom bolt with `PhantomBolt`, `PhantomLifetime(duration.0)`, `PhantomOwner(source_entity)`, infinite piercing, phantom visual, `CleanupOnExit<NodeState>`.
- `register`: register `tick_phantom_lifetime` in `EffectSystems::Tick`
- Components in `src/effect/effects/phantom_bolt/components.rs`:
  - `PhantomBolt` — marker
  - `PhantomLifetime(pub f32)`
  - `PhantomOwner(pub Entity)`

**27. ChainBoltConfig**
- File: `src/effect/effects/chain_bolt/config.rs`
- Struct: `ChainBoltConfig { tether_distance: OrderedFloat<f32> }`
- `Fireable::fire`: read source bolt position. Spawn new bolt at source position with random velocity. Mark as `ExtraBolt`. Create `DistanceConstraint` from `rantzsoft_physics2d` between source bolt and spawned bolt with `max_distance = tether_distance.0`.
- No `register` override

**28. MirrorConfig**
- File: `src/effect/effects/mirror_protocol/config.rs`
- Struct: `MirrorConfig { inherit: bool }`
- `Fireable::fire`: read source bolt position and velocity. Negate x-component for mirrored velocity. Spawn new bolt at source position with mirrored velocity. Mark as `ExtraBolt`. If `inherit` true, clone source bolt's `BoundEffects` onto mirrored bolt — cloned entries keep original source strings.
- No `register` override

**29. TetherBeamConfig**
- File: `src/effect/effects/tether_beam/config.rs`
- Struct: `TetherBeamConfig { damage_mult: OrderedFloat<f32>, chain: bool }`
- `Fireable::fire`:
  - If `chain == false`: spawn new bolt at source position with random velocity, mark `ExtraBolt`. Spawn tether beam entity with `TetherBeamSource { bolt_a: source, bolt_b: new_bolt }`, `TetherBeamDamage(damage_mult.0 * BoltBaseDamage)`, `CleanupOnExit<NodeState>`.
  - If `chain == true`: find nearest other bolt to source. If found, spawn tether beam entity connecting them. If not found, do nothing.
- `register`: register `tick_tether_beam_damage` and `cleanup_tether_beams` (chained) in `EffectSystems::Tick`
- Components in `src/effect/effects/tether_beam/components.rs`:
  - `TetherBeamSource { bolt_a: Entity, bolt_b: Entity }`
  - `TetherBeamDamage(pub f32)`

**30. GravityWellConfig**
- File: `src/effect/effects/gravity_well/config.rs`
- Struct: `GravityWellConfig { strength: OrderedFloat<f32>, duration: OrderedFloat<f32>, radius: OrderedFloat<f32>, max: u32 }`
- `Fireable::fire`: read source position. Query existing `GravityWellSource` entities with `GravityWellOwner` matching source. If count >= `max`, despawn oldest by `GravityWellLifetime`. Spawn entity with `GravityWellSource`, `GravityWellStrength(strength.0)`, `GravityWellRadius(radius.0)`, `GravityWellLifetime(duration.0)`, `GravityWellOwner(source_entity)`, `CleanupOnExit<NodeState>`.
- `register`: register `tick_gravity_wells` and `despawn_expired_wells` (chained) in `EffectSystems::Tick`
- Components in `src/effect/effects/gravity_well/components.rs`:
  - `GravityWellSource` — marker
  - `GravityWellStrength(pub f32)`
  - `GravityWellRadius(pub f32)`
  - `GravityWellLifetime(pub f32)`
  - `GravityWellOwner(pub Entity)`

---

#### GROUP G: Shared Component

**EffectSourceChip**
- File: `src/effect/components/effect_source_chip.rs`
- Struct: `EffectSourceChip(pub Option<String>)`
- `#[derive(Component)]`
- Added by spawner effect `fire()` methods to track damage attribution

---

#### GROUP H: Tick Systems

All tick systems run in `FixedUpdate`, in `EffectSystems::Tick`, with `run_if(in_state(NodeState::Playing))`. The `EffectSystems::Tick` set is ordered after `EffectSystems::Bridge` by the set-level ordering constraint established in wave 2. Individual tick systems do NOT need additional `.after()` calls beyond those required for chaining within the same effect (e.g., the 4 shockwave systems are chained together).

**tick_shockwave**
- File: `src/effect/effects/shockwave/systems.rs`
- Query: entities with `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveSpeed`
- Logic: increase `ShockwaveRadius.0` by `ShockwaveSpeed.0 * dt`
- Does NOT deal damage, despawn, or check cells

**sync_shockwave_visual**
- File: `src/effect/effects/shockwave/systems.rs`
- Query: entities with `ShockwaveSource`, `ShockwaveRadius`
- Logic: set `Scale2D` to match current `ShockwaveRadius`
- Schedule: chained after `tick_shockwave`

**apply_shockwave_damage**
- File: `src/effect/effects/shockwave/systems.rs`
- Query: entities with `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveDamaged`, `ShockwaveBaseDamage`, `ShockwaveDamageMultiplier`
- Logic: query quadtree for cells within radius. For each cell not in `ShockwaveDamaged`, send `DamageDealt<Cell>` with `ShockwaveBaseDamage.0 * ShockwaveDamageMultiplier.0`. Add cell to `ShockwaveDamaged` set.
- Schedule: chained after `sync_shockwave_visual`

**despawn_finished_shockwave**
- File: `src/effect/effects/shockwave/systems.rs`
- Query: entities with `ShockwaveSource`, `ShockwaveRadius`, `ShockwaveMaxRadius`
- Logic: if `ShockwaveRadius.0 >= ShockwaveMaxRadius.0`, despawn entity
- Schedule: chained after `apply_shockwave_damage`

**tick_chain_lightning**
- File: `src/effect/effects/chain_lightning/systems.rs`
- Query: entities with `ChainLightningChain`
- Logic: state machine per entity:
  - `Idle`: query quadtree for cells within `range` of `source_pos` not in `hit_set`. Pick random valid target. Spawn arc VFX entity. Transition to `ArcTraveling`. If no targets or `remaining_jumps == 0`: despawn chain entity and any active arc VFX.
  - `ArcTraveling`: advance `arc_pos` toward `target_pos` by `arc_speed * dt`. When reached: send `DamageDealt<Cell>`, add target to `hit_set`, set `source_pos` to `target_pos`, decrement `remaining_jumps`, despawn arc VFX, transition to `Idle`.

**tick_anchor**
- File: `src/effect/effects/anchor/systems.rs`
- Query: entities with `AnchorActive`
- Logic: state machine based on breaker velocity:
  1. Moving (nonzero velocity or dashing): remove `AnchorTimer` and `AnchorPlanted`. If was planted, remove bump force boost from `EffectStack<BumpForceConfig>` using `AnchorActive.source` as the source key.
  2. Stationary + no timer + not planted: insert `AnchorTimer(plant_delay)`.
  3. Stationary + timer active: decrement by `dt`. When reaches 0: remove timer, insert `AnchorPlanted`, push bump force boost to `EffectStack<BumpForceConfig>` using `AnchorActive.source` as the source key with `BumpForceConfig { multiplier: OrderedFloat(AnchorActive.bump_force_multiplier) }`.
  4. Stationary + planted: no-op.
- Does NOT modify perfect window directly — bump timing system reads `AnchorActive.perfect_window_multiplier` when `AnchorPlanted` present

**apply_attraction**
- File: `src/effect/effects/attraction/systems.rs`
- Query: entities with `ActiveAttractions`
- Logic: for each entry, query nearest entity matching `attraction_type` (via spatial queries from `rantzsoft_spatial2d`). Calculate steering force vector toward target. Apply to entity velocity. If `max_force` set, clamp per-tick delta.

**tick_pulse**
- File: `src/effect/effects/pulse/systems.rs`
- Query: entities with `PulseEmitter`
- Logic: decrement `timer` by `dt`. When timer <= 0: spawn shockwave at entity position using pulse config values and entity's current damage snapshot. Reset timer to `interval`.

**tick_shield_duration**
- File: `src/effect/effects/shield/systems.rs`
- Query: entities with `ShieldWall`, `ShieldDuration`, `ShieldReflectionCost`
- Logic: decrement `ShieldDuration.0` by `dt`. Read bolt-wall collision messages — for each bounce involving this shield wall, subtract `ShieldReflectionCost.0` from `ShieldDuration.0`. When `ShieldDuration.0 <= 0.0`, despawn entity.
- Schedule: in `EffectSystems::Tick` — no additional `.after()` needed beyond set-level ordering. The set runs after Bridge, which runs after collision systems.

**tick_phantom_lifetime**
- File: `src/effect/effects/phantom_bolt/systems.rs`
- Query: entities with `PhantomBolt`, `PhantomLifetime`
- Logic: decrement `PhantomLifetime.0` by `dt`. If `<= 0.0`, despawn entity.

**tick_tether_beam_damage**
- File: `src/effect/effects/tether_beam/systems.rs`
- Query: entities with `TetherBeamSource`, `TetherBeamDamage`
- Logic: read positions of both endpoint bolts. Calculate beam line segment. Query quadtree for cells intersecting line. Send `DamageDealt<Cell>` per intersecting cell.

**cleanup_tether_beams**
- File: `src/effect/effects/tether_beam/systems.rs`
- Query: entities with `TetherBeamSource`
- Logic: if either `bolt_a` or `bolt_b` entity has been despawned, despawn beam entity
- Schedule: chained after `tick_tether_beam_damage`

**tick_gravity_wells**
- File: `src/effect/effects/gravity_well/systems.rs`
- Query: entities with `GravityWellSource`, `GravityWellStrength`, `GravityWellRadius`, `GravityWellLifetime`
- Logic: for each well, query bolt entities within `GravityWellRadius.0`. Apply force toward well center scaled by `GravityWellStrength.0`. Decrement `GravityWellLifetime.0` by `dt`.

**despawn_expired_wells**
- File: `src/effect/effects/gravity_well/systems.rs`
- Query: entities with `GravityWellSource`, `GravityWellLifetime`
- Logic: if `GravityWellLifetime.0 <= 0.0`, despawn entity
- Schedule: chained after `tick_gravity_wells`

---

#### GROUP I: Reset Systems (EffectSystems::Reset)

**reset_ramping_damage**
- File: `src/effect/effects/ramping_damage/systems.rs`
- Query: entities with `RampingDamageAccumulator`
- Logic: set value to `OrderedFloat(0.0)`
- Schedule: `OnEnter(NodeState::Playing)`

Note: There is NO `reset_entropy_counter` system. EntropyEngine resets its counter internally when fired (counter resets to 0 after firing all effects in `EntropyConfig::fire()`). No separate reset system is needed.

---

#### GROUP J: Condition Evaluation System + Evaluators

**evaluate_conditions**
- File: `src/effect/conditions/evaluate_conditions.rs`
- Schedule: `FixedUpdate`, in `EffectSystems::Conditions`, after `EffectSystems::Tick`
- Logic:
  1. Evaluate each condition once per frame for the whole world:
     - `Condition::NodeActive` -> `is_node_active(world)`
     - `Condition::ShieldActive` -> `is_shield_active(world)`
     - `Condition::ComboActive(n)` -> `is_combo_active(world, n)` (per unique threshold)
  2. For each entity with `BoundEffects`, for each `BoundEntry` where `condition_active` is `Some(was_active)`:
     - Read the During entry's `Condition` from the tree
     - Look up current condition value from step 1
     - If `was_active == false` and now `true`: fire each scoped effect in the `ScopedTree`, set `condition_active = Some(true)`
     - If `was_active == true` and now `false`: reverse each scoped effect in the `ScopedTree`, set `condition_active = Some(false)`
     - If no change: do nothing
  3. Does NOT call `walk_effects`. Directly calls `fire_effect`/`reverse_effect`.
  4. Does NOT remove During entries. They are permanent.
  5. Does NOT evaluate non-During entries (`condition_active: None` are skipped).

**is_node_active**
- File: `src/effect/conditions/node_active.rs`
- Signature: `fn is_node_active(world: &World) -> bool`
- Logic: read `State<NodeState>` resource. Return `true` if `NodeState::Playing`, `false` otherwise.
- Pure read-only, no state modification.

**is_shield_active**
- File: `src/effect/conditions/shield_active.rs`
- Signature: `fn is_shield_active(world: &World) -> bool`
- Logic: return `true` if any entity with `ShieldWall` component exists. Return `false` if none.
- Pure read-only.

**is_combo_active**
- File: `src/effect/conditions/combo_active.rs`
- Signature: `fn is_combo_active(world: &World, threshold: u32) -> bool`
- Logic: read combo tracking resource. Return `true` if current consecutive perfect bump streak >= `threshold`.
- Pure read-only.

---

#### GROUP K: Dispatch Functions

**fire_dispatch**
- File: `src/effect/dispatch/fire_dispatch.rs`
- Function: takes an `EffectType`, entity, source, world. Match on all 30 variants, call `config.fire(entity, source, world)` for each.
- Every arm has the same shape: `EffectType::Variant(config) => config.fire(entity, source, world)`
- This is a single `match` statement with exactly 30 arms — one per `EffectType` variant. All arms are mechanical dispatch, no branching logic within any arm.

**reverse_dispatch**
- File: `src/effect/dispatch/reverse_dispatch.rs`
- Function: takes a `ReversibleEffectType`, entity, source, world. Match on all 16 variants, call `config.reverse(entity, source, world)` for each.
- Every arm has the same shape: `ReversibleEffectType::Variant(config) => config.reverse(entity, source, world)`
- This is a single `match` statement with exactly 16 arms — one per `ReversibleEffectType` variant. All arms are mechanical dispatch, no branching logic within any arm.

**Dispatch guidance**: `fire_dispatch` and `reverse_dispatch` are the central routing points. They must be exhaustive — every variant of their respective enums must have an arm. When adding a new effect, both files must be updated (fire for all effects, reverse only for reversible effects). The match statements are intentionally flat and repetitive — do not attempt to DRY them up with macros or indirection.

---

### Patterns to Follow

- All config structs derive `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`
- All f32 fields in config structs use `OrderedFloat<f32>` (for Eq matching in EffectStack removal)
- All runtime component f32 fields use plain `f32` (not OrderedFloat — only configs need Eq)
- All passive effects follow the identical fire/reverse/aggregate pattern via EffectStack
- All spawned effect entities include `CleanupOnExit<NodeState>` as a safety net for node teardown
- All tick systems gate on `run_if(in_state(NodeState::Playing))` — effects freeze during transitions
- Fire methods use exclusive `&mut World` access. They are called from the tree walker/dispatch, not from systems.
- Follow the folder-per-effect pattern: `src/effect/effects/<name>/` with `mod.rs`, `config.rs`, optionally `components.rs`, `systems.rs`
- Module `mod.rs` files are wiring-only: `pub(crate) mod config;` etc. + re-exports
- Tests go in `effects/<name>/tests.rs` (directory module pattern), NOT in `config.rs`

### RON Data
- No new RON files in this wave. All config values come from the effect tree definitions in `BoundEffects`/`StagedEffects`, which are loaded from existing chip/augment RON files.

### Schedule

**FixedUpdate, EffectSystems::Tick** (all with `run_if(in_state(NodeState::Playing))`):
- `tick_shockwave` -> `sync_shockwave_visual` -> `apply_shockwave_damage` -> `despawn_finished_shockwave` (chained)
- `tick_chain_lightning`
- `tick_anchor`
- `apply_attraction`
- `tick_pulse`
- `tick_shield_duration`
- `tick_phantom_lifetime`
- `tick_tether_beam_damage` -> `cleanup_tether_beams` (chained)
- `tick_gravity_wells` -> `despawn_expired_wells` (chained)

Note: `tick_effect_timers` and `check_node_timer_thresholds` are wave 5 scope and already implemented by this point. They are NOT part of this wave.

**FixedUpdate, EffectSystems::Conditions** (after EffectSystems::Tick):
- `evaluate_conditions`

**OnEnter(NodeState::Playing), EffectSystems::Reset**:
- `reset_ramping_damage`

### Wiring

Each effect's `register` method (called from `EffectPlugin::build`) handles its own system registration. The plugin calls `register` for every config type:

```rust
// In EffectPlugin::build (or equivalent)
SpeedBoostConfig::register(app);    // no-op
SizeBoostConfig::register(app);     // no-op
DamageBoostConfig::register(app);   // no-op
BumpForceConfig::register(app);     // no-op
QuickStopConfig::register(app);     // no-op
VulnerableConfig::register(app);    // no-op
PiercingConfig::register(app);      // no-op
RampingDamageConfig::register(app); // registers reset_ramping_damage
LoseLifeConfig::register(app);     // no-op
TimePenaltyConfig::register(app);  // no-op
DieConfig::register(app);          // no-op
RandomEffectConfig::register(app); // no-op
FlashStepConfig::register(app);    // no-op
ShieldConfig::register(app);       // registers tick_shield_duration
SecondWindConfig::register(app);   // no-op
PulseConfig::register(app);        // registers tick_pulse
AnchorConfig::register(app);       // registers tick_anchor
AttractionConfig::register(app);   // registers apply_attraction
CircuitBreakerConfig::register(app); // no-op
EntropyConfig::register(app);      // no-op (EntropyEngine resets internally when fired)
ShockwaveConfig::register(app);    // registers 4 chained shockwave systems
ExplodeConfig::register(app);      // no-op
ChainLightningConfig::register(app); // registers tick_chain_lightning
PiercingBeamConfig::register(app); // no-op
SpawnBoltsConfig::register(app);   // no-op
SpawnPhantomConfig::register(app); // registers tick_phantom_lifetime
ChainBoltConfig::register(app);    // no-op
MirrorConfig::register(app);       // no-op
TetherBeamConfig::register(app);   // registers tick_tether_beam_damage + cleanup_tether_beams
GravityWellConfig::register(app);  // registers tick_gravity_wells + despawn_expired_wells
```

The `evaluate_conditions` system is registered directly by the plugin (not via a config's `register`):
```rust
app.add_systems(FixedUpdate,
    evaluate_conditions
        .in_set(EffectSystems::Conditions)
        .after(EffectSystems::Tick)
);
```

Module declarations in `src/effect/effects/mod.rs`:
```rust
pub(crate) mod speed_boost;
pub(crate) mod size_boost;
pub(crate) mod damage_boost;
pub(crate) mod bump_force;
pub(crate) mod quick_stop;
pub(crate) mod vulnerable;
pub(crate) mod piercing;
pub(crate) mod ramping_damage;
pub(crate) mod lose_life;
pub(crate) mod time_penalty;
pub(crate) mod die;
pub(crate) mod spawn_bolts;
pub(crate) mod chain_bolt;
pub(crate) mod mirror_protocol;
pub(crate) mod random_effect;
pub(crate) mod explode;
pub(crate) mod piercing_beam;
pub(crate) mod shockwave;
pub(crate) mod chain_lightning;
pub(crate) mod anchor;
pub(crate) mod attraction;
pub(crate) mod pulse;
pub(crate) mod shield;
pub(crate) mod second_wind;
pub(crate) mod flash_step;
pub(crate) mod circuit_breaker;
pub(crate) mod entropy_engine;
pub(crate) mod gravity_well;
pub(crate) mod phantom_bolt;
pub(crate) mod tether_beam;
```

Module declarations in `src/effect/conditions/mod.rs`:
```rust
pub(crate) mod evaluate_conditions;
pub(crate) mod node_active;
pub(crate) mod shield_active;
pub(crate) mod combo_active;
```

### Constraints
- Do NOT modify: `src/effect/types/` (enums defined in prior waves), `src/effect/walking/` (tree walker from prior wave), `src/effect/storage/` (BoundEffects/StagedEffects from prior wave), `src/effect/commands/` (fire_effect/reverse_effect from prior wave), `src/effect/stacking/` (EffectStack from prior wave)
- Do NOT modify: any domain outside `src/effect/` (bolt, breaker, cells, run, etc.)
- Do NOT modify: `src/effect/triggers/` (bridges from Wave 5)
- Do NOT add: trigger bridge systems (those belong to Wave 5)
- Do NOT add: death pipeline handlers (those belong to Wave 7)
- Do NOT add: RON chip definitions (those belong to later integration waves)
- Do NOT add: `tick_effect_timers` or `check_node_timer_thresholds` (those belong to Wave 5)
- Do NOT add: `reset_entropy_counter` system (EntropyEngine resets internally when fired — no separate reset system exists)
- Do NOT implement: non-system helper functions that already exist from prior waves (fire_effect, reverse_effect, walk_effects)
- fire/reverse methods take `&mut World` — they are called from exclusive-access contexts, not from normal systems
- The `source` parameter in fire/reverse is `&str`, not `String` — the config clones it when needed (e.g., `source.to_string()` for EffectStack entries)
- All spawned entities (shockwaves, phantoms, gravity wells, shield walls, second-wind walls, chain lightning chains, tether beams) MUST include `CleanupOnExit<NodeState>` — this is the safety net for node teardown
- Damage formula for effects that use damage boosts: `(BoltBaseDamage + RampingDamageAccumulator) * EffectStack<DamageBoostConfig>.aggregate()` — but note that shockwave snapshots damage at spawn time, while piercing beam calculates it at fire time
