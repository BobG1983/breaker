# Test Spec: Effect Domain -- Wave 6: Effects + Tick Systems + Conditions

## Prerequisites

The following waves MUST be complete before wave 6 begins:

- **Wave 1**: Delete old effect code (clean slate)
- **Wave 2**: Scaffold -- all config structs, component types, EffectStack<T>, EffectType/ReversibleEffectType enums, Fireable/Reversible/PassiveEffect traits, EffectSystems system sets, message types (ApplyTimePenalty, EffectTimerExpired), plugin skeleton
- **Wave 3**: RON asset pipeline -- deserialization of config structs from chip/augment RON
- **Wave 4**: Functions -- EffectStack methods (push, remove, aggregate), walk_effects tree walker, fire_effect/reverse_effect commands
- **Wave 5**: Triggers -- all bridge systems in EffectV3Systems::Bridge, tick_effect_timers in EffectV3Systems::Tick, TriggerContext types, Condition enum, BoundEffects/StagedEffects storage

All types from waves 2-5 are assumed to exist and be functional: EffectStack, Fireable, Reversible, PassiveEffect, all config structs, all tree types, all triggers.

## Domain
`src/effect_v3/`

---

## Section A: Passive Effects (fire/reverse/aggregate)

All 8 passive effects follow the same EffectStack pattern. Each config implements Fireable, Reversible, and PassiveEffect.

### A1: SpeedBoost

#### A1.1 **SpeedBoost fire pushes entry onto EffectStack**
- Given: Entity with no `EffectStack<SpeedBoostConfig>`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
- Then: Entity has `EffectStack<SpeedBoostConfig>` with 1 entry `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
- Edge case: Entity already has an empty `EffectStack<SpeedBoostConfig>` -- fire inserts into existing stack, does not panic

#### A1.2 **SpeedBoost fire inserts default stack when absent**
- Given: Entity with no `EffectStack<SpeedBoostConfig>`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_b", world)`
- Then: Entity has `EffectStack<SpeedBoostConfig>` component (was inserted by fire). Stack contains exactly 1 entry.

#### A1.3 **SpeedBoost multiple fires from different sources stack**
- Given: Entity with `EffectStack<SpeedBoostConfig>` containing `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_b", world)`
- Then: Stack contains 2 entries: `("chip_a", OrderedFloat(1.5))` and `("chip_b", OrderedFloat(2.0))`

#### A1.4 **SpeedBoost multiple fires from same source stack**
- Given: Entity with `EffectStack<SpeedBoostConfig>` containing `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
- Then: Stack contains 2 entries, both `("chip_a", OrderedFloat(1.5))` -- duplicate sources are allowed

#### A1.5 **SpeedBoost reverse removes matching entry**
- Given: Entity with `EffectStack<SpeedBoostConfig>` containing `[("chip_a", OrderedFloat(1.5)), ("chip_b", OrderedFloat(2.0))]`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
- Then: Stack contains 1 entry: `("chip_b", OrderedFloat(2.0))`

#### A1.6 **SpeedBoost reverse does nothing when no match**
- Given: Entity with `EffectStack<SpeedBoostConfig>` containing `[("chip_b", OrderedFloat(2.0))]`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
- Then: Stack still contains 1 entry: `("chip_b", OrderedFloat(2.0))` -- no panic, no change

#### A1.7 **SpeedBoost reverse does nothing when stack is absent**
- Given: Entity with no `EffectStack<SpeedBoostConfig>`
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.reverse(entity, "chip_a", world)`
- Then: No panic. Entity still has no `EffectStack<SpeedBoostConfig>`.

#### A1.8 **SpeedBoost aggregate is multiplicative**
- Given: `EffectStack<SpeedBoostConfig>` with entries `[("a", OrderedFloat(1.5)), ("b", OrderedFloat(2.0)), ("c", OrderedFloat(0.5))]`
- When: `stack.aggregate()` called
- Then: Returns `1.5 * 2.0 * 0.5 = 1.5`
- Edge case: Empty stack returns `1.0` (multiplicative identity)

### A2: SizeBoost

#### A2.1 **SizeBoost fire pushes entry onto EffectStack**
- Given: Entity with no `EffectStack<SizeBoostConfig>`
- When: `SizeBoostConfig { multiplier: OrderedFloat(1.3) }.fire(entity, "chip_x", world)`
- Then: Entity has `EffectStack<SizeBoostConfig>` with 1 entry `("chip_x", SizeBoostConfig { multiplier: OrderedFloat(1.3) })`

#### A2.2 **SizeBoost reverse removes matching entry**
- Given: Entity with `EffectStack<SizeBoostConfig>` containing `[("chip_x", OrderedFloat(1.3)), ("chip_y", OrderedFloat(0.8))]`
- When: `SizeBoostConfig { multiplier: OrderedFloat(1.3) }.reverse(entity, "chip_x", world)`
- Then: Stack contains 1 entry: `("chip_y", OrderedFloat(0.8))`

#### A2.3 **SizeBoost aggregate is multiplicative**
- Given: `EffectStack<SizeBoostConfig>` with entries `[("a", OrderedFloat(1.3)), ("b", OrderedFloat(0.7))]`
- When: `stack.aggregate()` called
- Then: Returns `1.3 * 0.7 = 0.91`
- Edge case: Empty stack returns `1.0`

### A3: DamageBoost

#### A3.1 **DamageBoost fire pushes entry onto EffectStack**
- Given: Entity with no `EffectStack<DamageBoostConfig>`
- When: `DamageBoostConfig { multiplier: OrderedFloat(2.5) }.fire(entity, "chip_dmg", world)`
- Then: Entity has `EffectStack<DamageBoostConfig>` with 1 entry `("chip_dmg", DamageBoostConfig { multiplier: OrderedFloat(2.5) })`

#### A3.2 **DamageBoost reverse removes matching entry**
- Given: Entity with `EffectStack<DamageBoostConfig>` containing `[("chip_dmg", OrderedFloat(2.5))]`
- When: `DamageBoostConfig { multiplier: OrderedFloat(2.5) }.reverse(entity, "chip_dmg", world)`
- Then: Stack is empty

#### A3.3 **DamageBoost aggregate is multiplicative**
- Given: `EffectStack<DamageBoostConfig>` with entries `[("a", OrderedFloat(2.0)), ("b", OrderedFloat(3.0))]`
- When: `stack.aggregate()` called
- Then: Returns `2.0 * 3.0 = 6.0`
- Edge case: Empty stack returns `1.0`

### A4: BumpForce

#### A4.1 **BumpForce fire pushes entry onto EffectStack**
- Given: Entity with no `EffectStack<BumpForceConfig>`
- When: `BumpForceConfig { multiplier: OrderedFloat(1.8) }.fire(entity, "chip_bump", world)`
- Then: Entity has `EffectStack<BumpForceConfig>` with 1 entry

#### A4.2 **BumpForce reverse removes matching entry**
- Given: Entity with `EffectStack<BumpForceConfig>` containing `[("chip_bump", OrderedFloat(1.8))]`
- When: `BumpForceConfig { multiplier: OrderedFloat(1.8) }.reverse(entity, "chip_bump", world)`
- Then: Stack is empty

#### A4.3 **BumpForce aggregate is multiplicative**
- Given: `EffectStack<BumpForceConfig>` with entries `[("a", OrderedFloat(1.8)), ("b", OrderedFloat(1.2))]`
- When: `stack.aggregate()` called
- Then: Returns `1.8 * 1.2 = 2.16`
- Edge case: Empty stack returns `1.0`

### A5: QuickStop

#### A5.1 **QuickStop fire pushes entry onto EffectStack**
- Given: Entity with no `EffectStack<QuickStopConfig>`
- When: `QuickStopConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_qs", world)`
- Then: Entity has `EffectStack<QuickStopConfig>` with 1 entry

#### A5.2 **QuickStop reverse removes matching entry**
- Given: Entity with `EffectStack<QuickStopConfig>` containing `[("chip_qs", OrderedFloat(2.0))]`
- When: `QuickStopConfig { multiplier: OrderedFloat(2.0) }.reverse(entity, "chip_qs", world)`
- Then: Stack is empty

#### A5.3 **QuickStop aggregate is multiplicative**
- Given: `EffectStack<QuickStopConfig>` with entries `[("a", OrderedFloat(2.0)), ("b", OrderedFloat(1.5))]`
- When: `stack.aggregate()` called
- Then: Returns `2.0 * 1.5 = 3.0`
- Edge case: Empty stack returns `1.0`

### A6: Vulnerable

#### A6.1 **Vulnerable fire pushes entry onto EffectStack**
- Given: Entity (cell) with no `EffectStack<VulnerableConfig>`
- When: `VulnerableConfig { multiplier: OrderedFloat(2.0) }.fire(entity, "chip_vuln", world)`
- Then: Entity has `EffectStack<VulnerableConfig>` with 1 entry

#### A6.2 **Vulnerable reverse removes matching entry**
- Given: Entity with `EffectStack<VulnerableConfig>` containing `[("chip_vuln", OrderedFloat(2.0))]`
- When: `VulnerableConfig { multiplier: OrderedFloat(2.0) }.reverse(entity, "chip_vuln", world)`
- Then: Stack is empty

#### A6.3 **Vulnerable aggregate is multiplicative**
- Given: `EffectStack<VulnerableConfig>` with entries `[("a", OrderedFloat(2.0)), ("b", OrderedFloat(1.5))]`
- When: `stack.aggregate()` called
- Then: Returns `2.0 * 1.5 = 3.0`
- Edge case: Empty stack returns `1.0`

### A7: Piercing

#### A7.1 **Piercing fire pushes entry onto EffectStack and inserts PiercingRemaining**
- Given: Entity with no `EffectStack<PiercingConfig>` and no `PiercingRemaining`
- When: `PiercingConfig { charges: 3 }.fire(entity, "chip_pierce", world)`
- Then: Entity has `EffectStack<PiercingConfig>` with 1 entry `("chip_pierce", PiercingConfig { charges: 3 })`. Entity has `PiercingRemaining(3)`.

#### A7.2 **Piercing fire adds to existing stack, PiercingRemaining reflects aggregate**
- Given: Entity with `EffectStack<PiercingConfig>` containing `[("chip_a", PiercingConfig { charges: 2 })]` and `PiercingRemaining(2)`
- When: `PiercingConfig { charges: 3 }.fire(entity, "chip_b", world)`
- Then: Stack contains 2 entries. `PiercingRemaining` exists (initial insertion value is aggregate = 5 on first insert; if already present, fire does not overwrite PiercingRemaining -- PiercingRemaining is only inserted if absent)
- Edge case: PiercingRemaining already present -- fire does NOT overwrite the value

#### A7.3 **Piercing reverse removes matching entry, removes PiercingRemaining when aggregate is 0**
- Given: Entity with `EffectStack<PiercingConfig>` containing `[("chip_pierce", PiercingConfig { charges: 3 })]` and `PiercingRemaining(1)` (partially consumed)
- When: `PiercingConfig { charges: 3 }.reverse(entity, "chip_pierce", world)`
- Then: Stack is empty. `PiercingRemaining` is removed from entity.

#### A7.4 **Piercing reverse with remaining sources does not remove PiercingRemaining**
- Given: Entity with `EffectStack<PiercingConfig>` containing `[("chip_a", PiercingConfig { charges: 2 }), ("chip_b", PiercingConfig { charges: 3 })]` and `PiercingRemaining(4)`
- When: `PiercingConfig { charges: 2 }.reverse(entity, "chip_a", world)`
- Then: Stack contains 1 entry `("chip_b", 3)`. `PiercingRemaining` is still present (aggregate = 3 > 0).

#### A7.5 **Piercing aggregate is additive**
- Given: `EffectStack<PiercingConfig>` with entries `[("a", charges: 2), ("b", charges: 3)]`
- When: `stack.aggregate()` called
- Then: Returns `5.0` (sum of charges: 2 + 3)
- Edge case: Empty stack returns `0.0` (additive identity)

### A8: RampingDamage

#### A8.1 **RampingDamage fire pushes entry and inserts RampingDamageAccumulator**
- Given: Entity with no `EffectStack<RampingDamageConfig>` and no `RampingDamageAccumulator`
- When: `RampingDamageConfig { increment: OrderedFloat(5.0) }.fire(entity, "chip_ramp", world)`
- Then: Entity has `EffectStack<RampingDamageConfig>` with 1 entry. Entity has `RampingDamageAccumulator(OrderedFloat(0.0))`.

#### A8.2 **RampingDamage fire does not overwrite existing accumulator**
- Given: Entity with `EffectStack<RampingDamageConfig>` containing `[("chip_a", increment: OrderedFloat(5.0))]` and `RampingDamageAccumulator(OrderedFloat(15.0))`
- When: `RampingDamageConfig { increment: OrderedFloat(3.0) }.fire(entity, "chip_b", world)`
- Then: Stack contains 2 entries. `RampingDamageAccumulator` is still `OrderedFloat(15.0)` -- not reset.

#### A8.3 **RampingDamage reverse removes matching entry, removes accumulator when stack is empty**
- Given: Entity with `EffectStack<RampingDamageConfig>` containing `[("chip_ramp", increment: OrderedFloat(5.0))]` and `RampingDamageAccumulator(OrderedFloat(20.0))`
- When: `RampingDamageConfig { increment: OrderedFloat(5.0) }.reverse(entity, "chip_ramp", world)`
- Then: Stack is empty. `RampingDamageAccumulator` is removed from entity.

#### A8.4 **RampingDamage reverse with remaining sources does not remove accumulator**
- Given: Entity with `EffectStack<RampingDamageConfig>` containing `[("chip_a", increment: OrderedFloat(5.0)), ("chip_b", increment: OrderedFloat(3.0))]` and `RampingDamageAccumulator(OrderedFloat(10.0))`
- When: `RampingDamageConfig { increment: OrderedFloat(5.0) }.reverse(entity, "chip_a", world)`
- Then: Stack contains 1 entry. `RampingDamageAccumulator` still present with value `OrderedFloat(10.0)`.

#### A8.5 **RampingDamage aggregate is additive**
- Given: `EffectStack<RampingDamageConfig>` with entries `[("a", increment: OrderedFloat(5.0)), ("b", increment: OrderedFloat(3.0))]`
- When: `stack.aggregate()` called
- Then: Returns `8.0` (sum of increments: 5.0 + 3.0)
- Edge case: Empty stack returns `0.0` (additive identity)

---

## Section B: Toggle Effects (fire/reverse)

### B1: FlashStep

#### B1.1 **FlashStep fire inserts FlashStepActive marker**
- Given: Entity (breaker) with no `FlashStepActive`
- When: `FlashStepConfig {}.fire(entity, "chip_flash", world)`
- Then: Entity has `FlashStepActive` component

#### B1.2 **FlashStep fire is idempotent when marker already present**
- Given: Entity with `FlashStepActive` already present
- When: `FlashStepConfig {}.fire(entity, "chip_flash", world)`
- Then: Entity still has `FlashStepActive`. No panic.

#### B1.3 **FlashStep reverse removes FlashStepActive marker**
- Given: Entity with `FlashStepActive`
- When: `FlashStepConfig {}.reverse(entity, "chip_flash", world)`
- Then: Entity no longer has `FlashStepActive`

#### B1.4 **FlashStep reverse is safe when marker is absent**
- Given: Entity without `FlashStepActive`
- When: `FlashStepConfig {}.reverse(entity, "chip_flash", world)`
- Then: No panic. Entity still has no `FlashStepActive`.

---

## Section C: Protector Effects (fire/reverse)

### C1: Shield

#### C1.1 **Shield fire spawns ShieldWall entity**
- Given: World with a bolt entity at position `(0.0, 50.0)` as the target entity
- When: `ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }.fire(entity, "chip_shield", world)`
- Then: A new entity exists with `ShieldWall`, `ShieldOwner(entity)`, `ShieldDuration(10.0)`, `ShieldReflectionCost(2.0)`, `CleanupOnExit<NodeState>`

#### C1.2 **Shield reverse despawns all ShieldWall entities owned by target**
- Given: World with 2 `ShieldWall` entities: one with `ShieldOwner(entity_a)`, one with `ShieldOwner(entity_b)`
- When: `ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }.reverse(entity_a, "chip_shield", world)`
- Then: Only the ShieldWall with `ShieldOwner(entity_a)` is despawned. The one with `ShieldOwner(entity_b)` remains.

#### C1.3 **Shield reverse is safe when no matching ShieldWall exists**
- Given: World with no ShieldWall entities
- When: `ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }.reverse(entity, "chip_shield", world)`
- Then: No panic. No entities despawned.

### C2: SecondWind

#### C2.1 **SecondWind fire spawns SecondWindWall entity**
- Given: World with a bolt entity as target
- When: `SecondWindConfig {}.fire(entity, "chip_sw", world)`
- Then: A new entity exists with `SecondWindWall`, `SecondWindOwner(entity)`, `CleanupOnExit<NodeState>`

#### C2.2 **SecondWind reverse despawns all SecondWindWall entities owned by target**
- Given: World with 1 `SecondWindWall` entity with `SecondWindOwner(entity_a)`
- When: `SecondWindConfig {}.reverse(entity_a, "chip_sw", world)`
- Then: The SecondWindWall entity is despawned

#### C2.3 **SecondWind reverse is safe when no matching wall exists**
- Given: World with no SecondWindWall entities
- When: `SecondWindConfig {}.reverse(entity, "chip_sw", world)`
- Then: No panic.

### C3: Pulse

#### C3.1 **Pulse fire inserts PulseEmitter component**
- Given: Entity with no `PulseEmitter`
- When: `PulseConfig { base_range: OrderedFloat(100.0), range_per_level: OrderedFloat(20.0), stacks: 1, speed: OrderedFloat(200.0), interval: OrderedFloat(3.0) }.fire(entity, "chip_pulse", world)`
- Then: Entity has `PulseEmitter { base_range: 100.0, range_per_level: 20.0, stacks: 1, speed: 200.0, interval: 3.0, timer: 3.0 }`

#### C3.2 **Pulse reverse removes PulseEmitter**
- Given: Entity with `PulseEmitter { base_range: 100.0, range_per_level: 20.0, stacks: 1, speed: 200.0, interval: 3.0, timer: 1.5 }`
- When: `PulseConfig { base_range: OrderedFloat(100.0), range_per_level: OrderedFloat(20.0), stacks: 1, speed: OrderedFloat(200.0), interval: OrderedFloat(3.0) }.reverse(entity, "chip_pulse", world)`
- Then: Entity no longer has `PulseEmitter`. Active shockwaves already spawned continue (not tested here -- shockwave lifecycle tested separately).

#### C3.3 **Pulse reverse is safe when PulseEmitter is absent**
- Given: Entity without `PulseEmitter`
- When: `PulseConfig { ... }.reverse(entity, "chip_pulse", world)`
- Then: No panic.

---

## Section D: Stateful Effects (fire/reverse)

### D1: Anchor

#### D1.1 **Anchor fire inserts AnchorActive component with source**
- Given: Entity (breaker) with no `AnchorActive`
- When: `AnchorConfig { bump_force_multiplier: OrderedFloat(2.0), perfect_window_multiplier: OrderedFloat(1.5), plant_delay: OrderedFloat(1.0) }.fire(entity, "chip_anchor", world)`
- Then: Entity has `AnchorActive { bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, plant_delay: 1.0, source: "chip_anchor".to_string() }`

#### D1.2 **Anchor reverse removes AnchorActive, AnchorTimer, and AnchorPlanted**
- Given: Entity with `AnchorActive { bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, plant_delay: 1.0, source: "chip_anchor".to_string() }`, `AnchorTimer(0.5)`, and `AnchorPlanted`
- When: `AnchorConfig { ... }.reverse(entity, "chip_anchor", world)`
- Then: Entity has none of: `AnchorActive`, `AnchorTimer`, `AnchorPlanted`

#### D1.3 **Anchor reverse removes bump force boost if was planted**
- Given: Entity with `AnchorActive { bump_force_multiplier: 2.0, ..., source: "chip_anchor".to_string() }`, `AnchorPlanted`, and `EffectStack<BumpForceConfig>` containing `[("chip_anchor", BumpForceConfig { multiplier: OrderedFloat(2.0) })]`
- When: `AnchorConfig { ... }.reverse(entity, "chip_anchor", world)`
- Then: `EffectStack<BumpForceConfig>` no longer contains the entry sourced from `"chip_anchor"`

#### D1.4 **Anchor reverse is safe when components are partially absent**
- Given: Entity with `AnchorActive { ..., source: "chip_anchor".to_string() }` only (no AnchorTimer, no AnchorPlanted)
- When: `AnchorConfig { ... }.reverse(entity, "chip_anchor", world)`
- Then: `AnchorActive` removed. No panic from missing AnchorTimer or AnchorPlanted.

### D2: CircuitBreaker

#### D2.1 **CircuitBreaker fire on first activation inserts counter**
- Given: Entity with no `CircuitBreakerCounter`
- When: `CircuitBreakerConfig { bumps_required: 3, spawn_count: 2, inherit: true, shockwave_range: OrderedFloat(150.0), shockwave_speed: OrderedFloat(300.0) }.fire(entity, "chip_cb", world)`
- Then: Entity has `CircuitBreakerCounter { remaining: 2, bumps_required: 3, spawn_count: 2, inherit: true, shockwave_range: 150.0, shockwave_speed: 300.0 }` (first bump counts, so remaining = 3 - 1 = 2)

#### D2.2 **CircuitBreaker fire decrements existing counter**
- Given: Entity with `CircuitBreakerCounter { remaining: 2, bumps_required: 3, spawn_count: 2, ... }`
- When: `CircuitBreakerConfig { bumps_required: 3, ... }.fire(entity, "chip_cb", world)`
- Then: `CircuitBreakerCounter.remaining` is now `1`

#### D2.3 **CircuitBreaker fire triggers reward when remaining reaches 0**
- Given: Entity with `CircuitBreakerCounter { remaining: 1, bumps_required: 3, spawn_count: 2, inherit: true, shockwave_range: 150.0, shockwave_speed: 300.0 }` at position `(100.0, 200.0)`
- When: `CircuitBreakerConfig { bumps_required: 3, spawn_count: 2, inherit: true, shockwave_range: OrderedFloat(150.0), shockwave_speed: OrderedFloat(300.0) }.fire(entity, "chip_cb", world)`
- Then: `CircuitBreakerCounter.remaining` is reset to `3`. Reward entities were spawned (2 bolts at entity position + 1 shockwave at entity position).
- Edge case: `bumps_required: 1` -- first fire immediately triggers reward and resets counter

#### D2.4 **CircuitBreaker reverse removes counter**
- Given: Entity with `CircuitBreakerCounter { remaining: 2, bumps_required: 3, ... }`
- When: `CircuitBreakerConfig { ... }.reverse(entity, "chip_cb", world)`
- Then: Entity no longer has `CircuitBreakerCounter`

#### D2.5 **CircuitBreaker reverse is safe when counter is absent**
- Given: Entity with no `CircuitBreakerCounter`
- When: `CircuitBreakerConfig { ... }.reverse(entity, "chip_cb", world)`
- Then: No panic.

### D3: EntropyEngine

EntropyEngine resets its counter internally when fired -- no separate reset system is needed. Each fire call increments the count, fires `count` random effects from the pool, then resets count to 0.

#### D3.1 **EntropyEngine fire inserts counter on first activation**
- Given: Entity with no `EntropyCounter`
- When: `EntropyConfig { max_effects: 5, pool: vec![(OrderedFloat(1.0), Box::new(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))] }.fire(entity, "chip_entropy", world)`
- Then: Entity has `EntropyCounter { count: 0, max_effects: 5, pool: [...] }` (count reset to 0 after firing 1 effect). `EffectStack<SpeedBoostConfig>` has 1 entry (the random effect was fired once for count=1).

#### D3.2 **EntropyEngine fire increments count, fires that many effects, resets to zero**
- Given: Entity with `EntropyCounter { count: 0, max_effects: 5, pool: [(OrderedFloat(1.0), SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))] }`
- When: `EntropyConfig { max_effects: 5, pool: [...] }.fire(entity, "chip_entropy", world)` called 3 times sequentially
- Then: After each fire: count increments from 0 to 1, fires 1 effect, resets to 0. After 3 fires: `EntropyCounter.count` is `0` (reset after each fire). 3 total effects have been fired (one per activation).
- Edge case: The count increments from whatever it was before the reset. On first fire after node start, count goes 0->1, fires 1 effect, resets to 0.

#### D3.3 **EntropyEngine fire caps count at max_effects**
- Given: Entity with `EntropyCounter { count: 4, max_effects: 5, pool: [(OrderedFloat(1.0), SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))] }`
- When: `EntropyConfig { max_effects: 5, pool: [...] }.fire(entity, "chip_entropy", world)`
- Then: Count increments to 5 (capped at max_effects), 5 effects fired, count resets to 0. `EntropyCounter.count` is `0`.

#### D3.4 **EntropyEngine reverse removes counter**
- Given: Entity with `EntropyCounter { count: 0, max_effects: 5, pool: [...] }`
- When: `EntropyConfig { ... }.reverse(entity, "chip_entropy", world)`
- Then: Entity no longer has `EntropyCounter`. Does NOT reverse any previously fired effects.

#### D3.5 **EntropyEngine reverse is safe when counter absent**
- Given: Entity with no `EntropyCounter`
- When: `EntropyConfig { ... }.reverse(entity, "chip_entropy", world)`
- Then: No panic.

### D4: Attraction

#### D4.1 **Attraction fire pushes entry onto ActiveAttractions**
- Given: Entity with no `ActiveAttractions`
- When: `AttractionConfig { attraction_type: AttractionType::Cell, force: OrderedFloat(50.0), max_force: Some(OrderedFloat(100.0)) }.fire(entity, "chip_attract", world)`
- Then: Entity has `ActiveAttractions` with 1 entry: `AttractionEntry { source: "chip_attract", attraction_type: AttractionType::Cell, force: 50.0, max_force: Some(100.0) }`

#### D4.2 **Attraction fire adds to existing ActiveAttractions**
- Given: Entity with `ActiveAttractions` containing 1 entry from `"chip_a"`
- When: `AttractionConfig { attraction_type: AttractionType::Breaker, force: OrderedFloat(30.0), max_force: None }.fire(entity, "chip_b", world)`
- Then: `ActiveAttractions` contains 2 entries

#### D4.3 **Attraction reverse removes matching entry**
- Given: Entity with `ActiveAttractions` containing `[AttractionEntry { source: "chip_a", attraction_type: Cell, force: 50.0, max_force: Some(100.0) }, AttractionEntry { source: "chip_b", attraction_type: Breaker, force: 30.0, max_force: None }]`
- When: `AttractionConfig { attraction_type: AttractionType::Cell, force: OrderedFloat(50.0), max_force: Some(OrderedFloat(100.0)) }.reverse(entity, "chip_a", world)`
- Then: `ActiveAttractions` contains 1 entry: the "chip_b" entry

#### D4.4 **Attraction reverse removes component when Vec becomes empty**
- Given: Entity with `ActiveAttractions` containing 1 entry from `"chip_a"`
- When: `AttractionConfig { ... }.reverse(entity, "chip_a", world)`
- Then: `ActiveAttractions` component is removed from entity entirely

#### D4.5 **Attraction reverse is safe when component is absent**
- Given: Entity with no `ActiveAttractions`
- When: `AttractionConfig { ... }.reverse(entity, "chip_a", world)`
- Then: No panic.

---

## Section E: Spawner/Area Effects (fire only, not reversible)

### E1: Shockwave

#### E1.1 **Shockwave fire spawns shockwave entity with correct components**
- Given: Entity at position `(100.0, 200.0)` with `BoltBaseDamage(10.0)` and `EffectStack<DamageBoostConfig>` aggregating to `2.0`
- When: `ShockwaveConfig { base_range: OrderedFloat(50.0), range_per_level: OrderedFloat(10.0), stacks: 3, speed: OrderedFloat(300.0) }.fire(entity, "chip_shock", world)`
- Then: A new entity exists with:
  - `ShockwaveSource`
  - `ShockwaveRadius(0.0)`
  - `ShockwaveMaxRadius(70.0)` (50.0 + 10.0 * (3 - 1))
  - `ShockwaveSpeed(300.0)`
  - `ShockwaveBaseDamage(10.0)`
  - `ShockwaveDamageMultiplier(2.0)`
  - `ShockwaveDamaged` with empty HashSet
  - `EffectSourceChip(Some("chip_shock"))`
  - `CleanupOnExit<NodeState>`
- Edge case: Entity has no `BoltBaseDamage` -- fire should handle gracefully (insert default 0 or use 0.0)

#### E1.2 **Shockwave fire range calculation with stacks=1**
- Given: Entity at position `(0.0, 0.0)` with `BoltBaseDamage(5.0)`, empty `EffectStack<DamageBoostConfig>`
- When: `ShockwaveConfig { base_range: OrderedFloat(80.0), range_per_level: OrderedFloat(15.0), stacks: 1, speed: OrderedFloat(200.0) }.fire(entity, "chip_shock", world)`
- Then: `ShockwaveMaxRadius` is `80.0` (80.0 + 15.0 * (1 - 1) = 80.0)

### E2: Explode

#### E2.1 **Explode fire sends DamageDealt for cells in range**
- Given: Entity at position `(100.0, 200.0)`. Cell entity at `(120.0, 210.0)` (within range `OrderedFloat(50.0)`). Cell entity at `(300.0, 400.0)` (outside range `OrderedFloat(50.0)`).
- When: `ExplodeConfig { range: OrderedFloat(50.0), damage: OrderedFloat(25.0) }.fire(entity, "chip_explode", world)`
- Then: `DamageDealt<Cell>` message sent for the nearby cell with `damage = 25.0`. No message sent for the far cell.
- Edge case: No cells in range -- no messages sent, no panic

#### E2.2 **Explode damage is flat, ignores damage boosts**
- Given: Entity at position `(0.0, 0.0)` with `EffectStack<DamageBoostConfig>` aggregating to `3.0`. Cell at `(10.0, 0.0)`.
- When: `ExplodeConfig { range: OrderedFloat(50.0), damage: OrderedFloat(25.0) }.fire(entity, "chip_explode", world)`
- Then: `DamageDealt<Cell>` for the cell has `damage = 25.0` (not `75.0`)

### E3: ChainLightning

#### E3.1 **ChainLightning fire deals immediate damage to first target and spawns chain entity**
- Given: Entity at position `(100.0, 200.0)` with `BoltBaseDamage(10.0)` and `EffectStack<DamageBoostConfig>` aggregating to `1.5`. Cell at `(120.0, 210.0)`.
- When: `ChainLightningConfig { arcs: 3, range: OrderedFloat(100.0), damage_mult: OrderedFloat(2.0), arc_speed: OrderedFloat(500.0) }.fire(entity, "chip_chain", world)`
- Then: `DamageDealt<Cell>` sent for first target with damage `2.0 * 10.0 * 1.5 = 30.0`. A `ChainLightningChain` entity spawned with `remaining_jumps: 2`, `damage: 30.0`, `hit_set` containing the first target, `state: ChainState::Idle`, `range: 100.0`, `arc_speed: 500.0`, `source_pos` at first target's position, and `CleanupOnExit<NodeState>`.

#### E3.2 **ChainLightning fire with arcs=1 deals damage but does not spawn chain entity**
- Given: Entity at position `(0.0, 0.0)` with `BoltBaseDamage(10.0)`, empty damage boost stack. Cell at `(10.0, 0.0)`.
- When: `ChainLightningConfig { arcs: 1, range: OrderedFloat(100.0), damage_mult: OrderedFloat(1.0), arc_speed: OrderedFloat(500.0) }.fire(entity, "chip_chain", world)`
- Then: `DamageDealt<Cell>` sent for the cell. No `ChainLightningChain` entity spawned (remaining_jumps would be 0).

#### E3.3 **ChainLightning fire with no cells in range does nothing**
- Given: Entity at position `(0.0, 0.0)`. No cells within range `OrderedFloat(50.0)`.
- When: `ChainLightningConfig { arcs: 3, range: OrderedFloat(50.0), damage_mult: OrderedFloat(1.0), arc_speed: OrderedFloat(500.0) }.fire(entity, "chip_chain", world)`
- Then: No `DamageDealt<Cell>` sent. No `ChainLightningChain` entity spawned.

### E4: PiercingBeam

#### E4.1 **PiercingBeam fire deals damage to cells along velocity direction**
- Given: Entity at position `(100.0, 200.0)` with velocity direction `(0.0, 1.0)`, `BoltBaseDamage(10.0)`, `EffectStack<DamageBoostConfig>` aggregating to `2.0`. Cell at `(100.0, 300.0)` (in beam path). Cell at `(500.0, 300.0)` (outside beam width).
- When: `PiercingBeamConfig { damage_mult: OrderedFloat(1.5), width: OrderedFloat(20.0) }.fire(entity, "chip_beam", world)`
- Then: `DamageDealt<Cell>` sent for in-beam cell with damage `1.5 * 10.0 * 2.0 = 30.0`. No message for out-of-beam cell.

#### E4.2 **PiercingBeam fire with no cells in path does nothing**
- Given: Entity at `(0.0, 0.0)` with velocity direction `(0.0, 1.0)`. No cells in beam path.
- When: `PiercingBeamConfig { damage_mult: OrderedFloat(1.0), width: OrderedFloat(10.0) }.fire(entity, "chip_beam", world)`
- Then: No `DamageDealt<Cell>` messages sent.

### E5: SpawnBolts

#### E5.1 **SpawnBolts fire spawns correct number of bolts**
- Given: Entity at position `(100.0, 200.0)`
- When: `SpawnBoltsConfig { count: 3, lifespan: None, inherit: false }.fire(entity, "chip_spawn", world)`
- Then: 3 new bolt entities exist at position `(100.0, 200.0)`, each marked `ExtraBolt`, each with random velocity directions

#### E5.2 **SpawnBolts fire with lifespan attaches timer**
- Given: Entity at position `(100.0, 200.0)`
- When: `SpawnBoltsConfig { count: 2, lifespan: Some(OrderedFloat(5.0)), inherit: false }.fire(entity, "chip_spawn", world)`
- Then: 2 new bolt entities, each with a lifespan timer set to `5.0` seconds

#### E5.3 **SpawnBolts fire with inherit copies BoundEffects**
- Given: Entity at position `(100.0, 200.0)` with `BoundEffects` containing some entries. A primary bolt exists.
- When: `SpawnBoltsConfig { count: 1, lifespan: None, inherit: true }.fire(entity, "chip_spawn", world)`
- Then: The spawned bolt has a clone of the primary bolt's `BoundEffects`. Does NOT inherit `StagedEffects`.

#### E5.4 **SpawnBolts fire with count=0 spawns nothing**
- Given: Entity at position `(0.0, 0.0)`
- When: `SpawnBoltsConfig { count: 0, lifespan: None, inherit: false }.fire(entity, "chip_spawn", world)`
- Then: No new bolt entities spawned. No panic.

### E6: SpawnPhantom

#### E6.1 **SpawnPhantom fire spawns phantom bolt entity**
- Given: Entity at position `(100.0, 200.0)`, no existing `PhantomBolt` entities with `PhantomOwner(entity)`
- When: `SpawnPhantomConfig { duration: OrderedFloat(8.0), max_active: 3 }.fire(entity, "chip_phantom", world)`
- Then: A new entity exists with `PhantomBolt`, `PhantomLifetime(8.0)`, `PhantomOwner(entity)`, `CleanupOnExit<NodeState>`

#### E6.2 **SpawnPhantom fire despawns oldest when at max_active**
- Given: Entity with 3 existing `PhantomBolt` entities owned by it: phantom_a `PhantomLifetime(2.0)`, phantom_b `PhantomLifetime(5.0)`, phantom_c `PhantomLifetime(7.0)`. `max_active = 3`.
- When: `SpawnPhantomConfig { duration: OrderedFloat(8.0), max_active: 3 }.fire(entity, "chip_phantom", world)`
- Then: phantom_a (lowest lifetime = oldest) is despawned. A new phantom exists. Total count is still 3.

#### E6.3 **SpawnPhantom fire with max_active=1 replaces existing**
- Given: Entity with 1 existing `PhantomBolt` entity owned by it with `PhantomLifetime(3.0)`
- When: `SpawnPhantomConfig { duration: OrderedFloat(5.0), max_active: 1 }.fire(entity, "chip_phantom", world)`
- Then: Old phantom despawned. New phantom with `PhantomLifetime(5.0)` exists. Total count is 1.

### E7: ChainBolt

#### E7.1 **ChainBolt fire spawns tethered bolt**
- Given: Bolt entity at position `(100.0, 200.0)`
- When: `ChainBoltConfig { tether_distance: OrderedFloat(80.0) }.fire(entity, "chip_chain_bolt", world)`
- Then: A new bolt entity at position `(100.0, 200.0)` marked `ExtraBolt`. A `DistanceConstraint` exists between the original bolt and the new bolt with `max_distance = 80.0`.

### E8: MirrorProtocol

#### E8.1 **MirrorProtocol fire spawns mirrored bolt**
- Given: Bolt entity at position `(100.0, 200.0)` with velocity `(300.0, 400.0)`
- When: `MirrorConfig { inherit: false }.fire(entity, "chip_mirror", world)`
- Then: A new bolt entity at position `(100.0, 200.0)` with velocity `(-300.0, 400.0)` (x-component negated), marked `ExtraBolt`

#### E8.2 **MirrorProtocol fire with inherit clones BoundEffects**
- Given: Bolt entity at position `(50.0, 50.0)` with velocity `(100.0, -200.0)` and `BoundEffects` containing entries with source strings `["chip_a", "chip_b"]`
- When: `MirrorConfig { inherit: true }.fire(entity, "chip_mirror", world)`
- Then: Spawned bolt has cloned `BoundEffects` with same source strings `["chip_a", "chip_b"]`. Velocity is `(-100.0, -200.0)`.

#### E8.3 **MirrorProtocol fire with zero x-velocity**
- Given: Bolt entity at position `(50.0, 50.0)` with velocity `(0.0, 400.0)`
- When: `MirrorConfig { inherit: false }.fire(entity, "chip_mirror", world)`
- Then: Spawned bolt has velocity `(0.0, 400.0)` (negating 0.0 is still 0.0 -- mirror produces a co-located bolt moving in same direction)
- Edge case: This is technically degenerate but should not panic

### E9: TetherBeam

#### E9.1 **TetherBeam fire with chain=false spawns new bolt and beam**
- Given: Bolt entity at position `(100.0, 200.0)` with `BoltBaseDamage(10.0)`
- When: `TetherBeamConfig { damage_mult: OrderedFloat(1.5), chain: false }.fire(entity, "chip_tether", world)`
- Then: A new bolt entity at `(100.0, 200.0)` marked `ExtraBolt`. A tether beam entity with `TetherBeamSource { bolt_a: entity, bolt_b: new_bolt }`, `TetherBeamDamage(15.0)` (1.5 * 10.0), and `CleanupOnExit<NodeState>`.

#### E9.2 **TetherBeam fire with chain=true connects to nearest existing bolt**
- Given: Bolt entity_a at `(100.0, 200.0)` with `BoltBaseDamage(10.0)`. Another bolt entity_b at `(120.0, 210.0)`.
- When: `TetherBeamConfig { damage_mult: OrderedFloat(2.0), chain: true }.fire(entity_a, "chip_tether", world)`
- Then: A tether beam entity with `TetherBeamSource { bolt_a: entity_a, bolt_b: entity_b }`, `TetherBeamDamage(20.0)` (2.0 * 10.0), `CleanupOnExit<NodeState>`. No new bolt spawned.

#### E9.3 **TetherBeam fire with chain=true and no other bolts does nothing**
- Given: Only bolt entity in world at `(100.0, 200.0)` with `BoltBaseDamage(10.0)`
- When: `TetherBeamConfig { damage_mult: OrderedFloat(1.0), chain: true }.fire(entity, "chip_tether", world)`
- Then: No tether beam entity created. No new bolt spawned. No panic.

### E10: GravityWell

#### E10.1 **GravityWell fire spawns gravity well entity**
- Given: Entity at position `(100.0, 200.0)`, no existing `GravityWellSource` entities with `GravityWellOwner(entity)`
- When: `GravityWellConfig { strength: OrderedFloat(50.0), duration: OrderedFloat(5.0), radius: OrderedFloat(120.0), max: 2 }.fire(entity, "chip_grav", world)`
- Then: A new entity exists with `GravityWellSource`, `GravityWellStrength(50.0)`, `GravityWellRadius(120.0)`, `GravityWellLifetime(5.0)`, `GravityWellOwner(entity)`, `CleanupOnExit<NodeState>`

#### E10.2 **GravityWell fire despawns oldest when at max**
- Given: Entity with 2 existing `GravityWellSource` entities: well_a `GravityWellLifetime(1.0)`, well_b `GravityWellLifetime(3.0)`. `max = 2`.
- When: `GravityWellConfig { strength: OrderedFloat(50.0), duration: OrderedFloat(5.0), radius: OrderedFloat(120.0), max: 2 }.fire(entity, "chip_grav", world)`
- Then: well_a (lowest lifetime = oldest) despawned. New well exists. Total count is 2.

#### E10.3 **GravityWell fire when under max does not despawn anything**
- Given: Entity with 1 existing gravity well. `max = 3`.
- When: `GravityWellConfig { strength: OrderedFloat(50.0), duration: OrderedFloat(5.0), radius: OrderedFloat(120.0), max: 3 }.fire(entity, "chip_grav", world)`
- Then: Now 2 gravity wells. None despawned.

---

## Section F: Message/Meta Effects (fire only)

### F1: LoseLife

#### F1.1 **LoseLife fire sends DamageDealt<Breaker> message**
- Given: Entity (breaker)
- When: `LoseLifeConfig {}.fire(entity, "chip_lose", world)`
- Then: `DamageDealt<Breaker>` message sent with `dealer: None`, `target: entity`, `amount: 1.0`, `source_chip: Some("chip_lose")`

#### F1.2 **LoseLife fire does not directly modify Hp**
- Given: Entity (breaker) with `Hp(3)`
- When: `LoseLifeConfig {}.fire(entity, "chip_lose", world)`
- Then: `Hp` component is still `Hp(3)` (the message handler modifies it, not fire)

### F2: TimePenalty

#### F2.1 **TimePenalty fire sends ApplyTimePenalty message**
- Given: World with NodeTimer
- When: `TimePenaltyConfig { seconds: OrderedFloat(5.0) }.fire(entity, "chip_time", world)`
- Then: `ApplyTimePenalty { seconds: 5.0 }` message sent

#### F2.2 **TimePenalty fire does not modify NodeTimer directly**
- Given: World with `NodeTimer` at `30.0` remaining
- When: `TimePenaltyConfig { seconds: OrderedFloat(5.0) }.fire(entity, "chip_time", world)`
- Then: NodeTimer remains at `30.0` (the run domain's handler subtracts seconds)

### F3: Die

#### F3.1 **Die fire sends KillYourself for bolt entity**
- Given: Entity with `Bolt` marker component
- When: `DieConfig {}.fire(entity, "chip_die", world)`
- Then: `KillYourself<Bolt>` message sent with `victim: entity`

#### F3.2 **Die fire sends KillYourself for cell entity**
- Given: Entity with `Cell` marker component
- When: `DieConfig {}.fire(entity, "chip_die", world)`
- Then: `KillYourself<Cell>` message sent with `victim: entity`

#### F3.3 **Die fire sends KillYourself for wall entity**
- Given: Entity with `Wall` marker component
- When: `DieConfig {}.fire(entity, "chip_die", world)`
- Then: `KillYourself<Wall>` message sent with `victim: entity`

#### F3.4 **Die fire sends KillYourself for breaker entity**
- Given: Entity with `Breaker` marker component
- When: `DieConfig {}.fire(entity, "chip_die", world)`
- Then: `KillYourself<Breaker>` message sent with `victim: entity`

#### F3.5 **Die fire does nothing for entity with no recognized marker**
- Given: Entity with no Bolt, Cell, Wall, or Breaker component
- When: `DieConfig {}.fire(entity, "chip_die", world)`
- Then: No message sent. No panic.

### F4: RandomEffect

#### F4.1 **RandomEffect fire delegates to selected effect**
- Given: Entity with no `EffectStack<SpeedBoostConfig>`. Pool = `[(OrderedFloat(1.0), SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))]` (single element, always selected).
- When: `RandomEffectConfig { pool: [...] }.fire(entity, "chip_random", world)`
- Then: Entity has `EffectStack<SpeedBoostConfig>` with 1 entry `("chip_random", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })` -- the delegated SpeedBoost fire ran.

#### F4.2 **RandomEffect fire with empty pool does nothing**
- Given: Entity. Pool = `[]`
- When: `RandomEffectConfig { pool: vec![] }.fire(entity, "chip_random", world)`
- Then: No effect applied. No panic.

#### F4.3 **RandomEffect fire selects weighted**
- Given: Entity. Pool = `[(OrderedFloat(0.0), SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })), (OrderedFloat(1.0), DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))]` (first has weight 0, second has weight 1).
- When: `RandomEffectConfig { pool: [...] }.fire(entity, "chip_random", world)` (run multiple times or with deterministic seed)
- Then: Only `DamageBoost` is ever selected because `SpeedBoost` has weight 0.0.

---

## Section G: Tick Systems

### G1: tick_shockwave

#### G1.1 **tick_shockwave expands radius by speed * dt**
- Given: Shockwave entity with `ShockwaveSource`, `ShockwaveRadius(0.0)`, `ShockwaveSpeed(300.0)`. Fixed timestep dt = `1.0/60.0`.
- When: `tick_shockwave` runs one frame
- Then: `ShockwaveRadius` is `300.0 / 60.0 = 5.0`
- Edge case: `ShockwaveSpeed(0.0)` -- radius stays at `0.0`

#### G1.2 **tick_shockwave does not affect non-shockwave entities**
- Given: Entity without `ShockwaveSource` but with `ShockwaveRadius(0.0)` and `ShockwaveSpeed(100.0)`
- When: `tick_shockwave` runs
- Then: `ShockwaveRadius` unchanged at `0.0`

### G2: sync_shockwave_visual

#### G2.1 **sync_shockwave_visual sets Scale2D to diameter of shockwave**
- Given: Shockwave entity with `ShockwaveSource`, `ShockwaveRadius(50.0)`
- When: `sync_shockwave_visual` runs
- Then: Entity `Scale2D` is `(100.0, 100.0)` (ShockwaveRadius * 2.0 for diameter, uniform scale on both axes)
- Edge case: `ShockwaveRadius(0.0)` -- `Scale2D` is `(0.0, 0.0)`

### G3: apply_shockwave_damage

#### G3.1 **apply_shockwave_damage sends DamageDealt for cells in radius**
- Given: Shockwave entity with `ShockwaveSource`, `ShockwaveRadius(50.0)`, `ShockwaveBaseDamage(10.0)`, `ShockwaveDamageMultiplier(2.0)`, `ShockwaveDamaged` = empty. Cell_a at distance `30.0` from shockwave. Cell_b at distance `60.0`.
- When: `apply_shockwave_damage` runs
- Then: `DamageDealt<Cell>` sent for cell_a with damage `10.0 * 2.0 = 20.0`. Cell_a added to `ShockwaveDamaged`. No message for cell_b.

#### G3.2 **apply_shockwave_damage does not damage already-hit cells**
- Given: Shockwave entity with `ShockwaveRadius(100.0)`, `ShockwaveDamaged` containing `cell_a`. Cell_a at distance `30.0`.
- When: `apply_shockwave_damage` runs
- Then: No `DamageDealt<Cell>` sent for cell_a (already in damaged set).
- Edge case: All cells in range already damaged -- no messages sent

#### G3.3 **apply_shockwave_damage with no cells in range does nothing**
- Given: Shockwave entity with `ShockwaveRadius(10.0)`. No cells within 10.0 distance.
- When: `apply_shockwave_damage` runs
- Then: No messages sent. No panic.

### G4: despawn_finished_shockwave

#### G4.1 **despawn_finished_shockwave despawns at max radius**
- Given: Shockwave entity with `ShockwaveSource`, `ShockwaveRadius(100.0)`, `ShockwaveMaxRadius(100.0)`
- When: `despawn_finished_shockwave` runs
- Then: Entity is despawned

#### G4.2 **despawn_finished_shockwave despawns when radius exceeds max**
- Given: Shockwave entity with `ShockwaveRadius(105.0)`, `ShockwaveMaxRadius(100.0)`
- When: `despawn_finished_shockwave` runs
- Then: Entity is despawned

#### G4.3 **despawn_finished_shockwave does not despawn under max**
- Given: Shockwave entity with `ShockwaveRadius(95.0)`, `ShockwaveMaxRadius(100.0)`
- When: `despawn_finished_shockwave` runs
- Then: Entity still exists
- Edge case: `ShockwaveRadius(0.0)` with `ShockwaveMaxRadius(0.0)` -- should despawn

### G5: tick_chain_lightning

#### G5.1 **tick_chain_lightning in Idle state picks target and transitions to ArcTraveling**
- Given: `ChainLightningChain` entity with `state: Idle`, `remaining_jumps: 2`, `range: 100.0`, `source_pos: (50.0, 50.0)`, `hit_set: {cell_a}`. Cell_b at `(80.0, 60.0)` (within range, not in hit_set). Cell_a at `(60.0, 55.0)` (in hit_set).
- When: `tick_chain_lightning` runs one frame
- Then: State transitions to `ArcTraveling` with `target: cell_b`, `target_pos: (80.0, 60.0)`. An arc VFX entity is spawned at `source_pos`.
- Edge case: Multiple valid targets -- one is selected randomly

#### G5.2 **tick_chain_lightning in Idle state with no valid targets despawns chain**
- Given: `ChainLightningChain` entity with `state: Idle`, `remaining_jumps: 2`, `hit_set` containing all cells in range
- When: `tick_chain_lightning` runs
- Then: Chain entity is despawned

#### G5.3 **tick_chain_lightning in Idle state with remaining_jumps=0 despawns chain**
- Given: `ChainLightningChain` entity with `state: Idle`, `remaining_jumps: 0`
- When: `tick_chain_lightning` runs
- Then: Chain entity is despawned

#### G5.4 **tick_chain_lightning in ArcTraveling advances arc position toward target**
- Given: `ChainLightningChain` entity with `state: ArcTraveling { target: cell_b, target_pos: (100.0, 100.0), arc_entity: arc_e, arc_pos: (50.0, 50.0) }`, `arc_speed: 500.0`. dt = `1.0/60.0`.
- When: `tick_chain_lightning` runs one frame
- Then: `arc_pos` has moved toward `(100.0, 100.0)` by distance `500.0 / 60.0 ~= 8.33` units

#### G5.5 **tick_chain_lightning in ArcTraveling completes arc on arrival**
- Given: `ChainLightningChain` entity with `state: ArcTraveling { target: cell_b, target_pos: (52.0, 50.0), arc_entity: arc_e, arc_pos: (50.0, 50.0) }`, `arc_speed: 500.0`, `damage: 20.0`, `remaining_jumps: 2`, `source_pos: (30.0, 30.0)`. dt large enough that arc overshoots target.
- When: `tick_chain_lightning` runs
- Then: `DamageDealt<Cell>` sent for `cell_b` with damage `20.0`. `cell_b` added to `hit_set`. `remaining_jumps` decremented to `1`. `source_pos` updated to `(52.0, 50.0)`. Arc VFX entity `arc_e` despawned. State transitions to `Idle`.

### G6: tick_anchor

#### G6.1 **tick_anchor starts timer when breaker stops moving**
- Given: Entity with `AnchorActive { plant_delay: 1.0, bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, source: "chip_anchor".to_string() }`, velocity `(0.0, 0.0)`, no `AnchorTimer`, no `AnchorPlanted`
- When: `tick_anchor` runs
- Then: Entity has `AnchorTimer(1.0)`

#### G6.2 **tick_anchor decrements timer while stationary**
- Given: Entity with `AnchorActive { plant_delay: 1.0, ..., source: "chip_anchor".to_string() }`, `AnchorTimer(0.5)`, velocity `(0.0, 0.0)`. dt = `1.0/60.0`.
- When: `tick_anchor` runs one frame
- Then: `AnchorTimer` value decreased by `1.0/60.0`

#### G6.3 **tick_anchor plants breaker when timer reaches zero**
- Given: Entity with `AnchorActive { plant_delay: 1.0, bump_force_multiplier: 2.0, ..., source: "chip_anchor".to_string() }`, `AnchorTimer(0.01)`, velocity `(0.0, 0.0)`. dt = `1.0/60.0`.
- When: `tick_anchor` runs (timer will reach 0)
- Then: `AnchorTimer` removed. `AnchorPlanted` inserted. `EffectStack<BumpForceConfig>` has entry `("chip_anchor", BumpForceConfig { multiplier: OrderedFloat(2.0) })` pushed via the `source` field from `AnchorActive`.

#### G6.4 **tick_anchor unplants when breaker starts moving**
- Given: Entity with `AnchorActive { bump_force_multiplier: 2.0, ..., source: "chip_anchor".to_string() }`, `AnchorPlanted`, velocity `(100.0, 0.0)`, `EffectStack<BumpForceConfig>` containing `("chip_anchor", BumpForceConfig { multiplier: OrderedFloat(2.0) })`
- When: `tick_anchor` runs
- Then: `AnchorPlanted` removed. `AnchorTimer` removed (if present). Bump force entry `("chip_anchor", OrderedFloat(2.0))` removed from `EffectStack<BumpForceConfig>` using the `source` field from `AnchorActive`.

#### G6.5 **tick_anchor resets timer when breaker starts moving before planting**
- Given: Entity with `AnchorActive { ..., source: "chip_anchor".to_string() }`, `AnchorTimer(0.3)`, velocity `(50.0, 0.0)`, no `AnchorPlanted`
- When: `tick_anchor` runs
- Then: `AnchorTimer` removed. No `AnchorPlanted` inserted.

#### G6.6 **tick_anchor no-op when already planted and stationary**
- Given: Entity with `AnchorActive { ..., source: "chip_anchor".to_string() }`, `AnchorPlanted`, velocity `(0.0, 0.0)`, `EffectStack<BumpForceConfig>` with entry
- When: `tick_anchor` runs
- Then: No changes. `AnchorPlanted` remains. Stack unchanged.

### G7: apply_attraction

#### G7.1 **apply_attraction steers bolt toward nearest matching entity**
- Given: Bolt entity at `(100.0, 100.0)` with velocity `(200.0, 0.0)` and `ActiveAttractions` containing `[AttractionEntry { source: "chip_a", attraction_type: AttractionType::Cell, force: 50.0, max_force: None }]`. Cell at `(100.0, 200.0)`.
- When: `apply_attraction` runs one frame
- Then: Bolt velocity has a component toward `(100.0, 200.0)`. The y-component of velocity increased (steering toward cell above).

#### G7.2 **apply_attraction respects max_force cap**
- Given: Bolt entity at `(0.0, 0.0)` with velocity `(200.0, 0.0)` and `ActiveAttractions` containing `[AttractionEntry { source: "chip_a", attraction_type: AttractionType::Cell, force: 1000.0, max_force: Some(5.0) }]`. Cell at `(0.0, 100.0)`.
- When: `apply_attraction` runs one frame
- Then: The per-tick steering delta magnitude does not exceed `5.0`.

#### G7.3 **apply_attraction with no matching targets does nothing**
- Given: Bolt entity with `ActiveAttractions` containing `[AttractionEntry { attraction_type: AttractionType::Cell, ... }]`. No cells in world.
- When: `apply_attraction` runs
- Then: Bolt velocity unchanged.

#### G7.4 **apply_attraction handles multiple entries with different attraction types**
- Given: Bolt entity at `(100.0, 100.0)` with `ActiveAttractions` containing 2 entries: one `AttractionType::Cell` and one `AttractionType::Breaker`. Cell at `(100.0, 200.0)`, Breaker at `(200.0, 100.0)`.
- When: `apply_attraction` runs
- Then: Bolt velocity has steering components toward both targets (combined).

### G8: tick_pulse

#### G8.1 **tick_pulse decrements timer**
- Given: Entity with `PulseEmitter { base_range: 100.0, range_per_level: 20.0, stacks: 2, speed: 200.0, interval: 3.0, timer: 2.0 }` at position `(50.0, 50.0)`. dt = `1.0/60.0`.
- When: `tick_pulse` runs one frame
- Then: `PulseEmitter.timer` is `2.0 - 1.0/60.0`

#### G8.2 **tick_pulse spawns shockwave when timer reaches zero**
- Given: Entity with `PulseEmitter { base_range: 100.0, range_per_level: 20.0, stacks: 2, speed: 200.0, interval: 3.0, timer: 0.01 }` at position `(50.0, 50.0)`, `BoltBaseDamage(10.0)`, `EffectStack<DamageBoostConfig>` aggregating to `1.5`. dt = `1.0/60.0`.
- When: `tick_pulse` runs (timer crosses zero)
- Then: A shockwave entity spawned at `(50.0, 50.0)` with `ShockwaveMaxRadius(120.0)` (100.0 + 20.0 * (2 - 1)), `ShockwaveSpeed(200.0)`. `PulseEmitter.timer` reset to `3.0`.

#### G8.3 **tick_pulse does not spawn shockwave when timer still positive**
- Given: Entity with `PulseEmitter { timer: 1.5, interval: 3.0, ... }`
- When: `tick_pulse` runs one frame
- Then: No shockwave entity spawned. Timer decremented.

### G9: tick_shield_duration

#### G9.1 **tick_shield_duration decrements by dt**
- Given: ShieldWall entity with `ShieldDuration(10.0)`. dt = `1.0/60.0`.
- When: `tick_shield_duration` runs one frame
- Then: `ShieldDuration` is `10.0 - 1.0/60.0`

#### G9.2 **tick_shield_duration subtracts reflection cost on bolt bounce**
- Given: ShieldWall entity with `ShieldDuration(10.0)`, `ShieldReflectionCost(2.0)`. A `BoltImpactWall` message with this shield wall entity.
- When: `tick_shield_duration` runs
- Then: `ShieldDuration` is `10.0 - dt - 2.0`

#### G9.3 **tick_shield_duration despawns shield when duration reaches zero**
- Given: ShieldWall entity with `ShieldDuration(0.01)`. dt large enough to bring below zero.
- When: `tick_shield_duration` runs
- Then: ShieldWall entity is despawned

#### G9.4 **tick_shield_duration despawns shield when reflection cost brings below zero**
- Given: ShieldWall entity with `ShieldDuration(1.5)`, `ShieldReflectionCost(2.0)`. A bolt bounce message for this shield.
- When: `tick_shield_duration` runs
- Then: `ShieldDuration` goes below zero. Entity despawned.

#### G9.5 **tick_shield_duration does nothing when no bounces and duration positive**
- Given: ShieldWall entity with `ShieldDuration(5.0)`. No bolt bounce messages. dt = `1.0/60.0`.
- When: `tick_shield_duration` runs
- Then: `ShieldDuration` is `5.0 - 1.0/60.0`. Entity remains.

### G10: tick_phantom_lifetime

#### G10.1 **tick_phantom_lifetime decrements by dt**
- Given: PhantomBolt entity with `PhantomLifetime(5.0)`. dt = `1.0/60.0`.
- When: `tick_phantom_lifetime` runs one frame
- Then: `PhantomLifetime` is `5.0 - 1.0/60.0`

#### G10.2 **tick_phantom_lifetime despawns when lifetime reaches zero**
- Given: PhantomBolt entity with `PhantomLifetime(0.01)`. dt = `1.0/60.0`.
- When: `tick_phantom_lifetime` runs
- Then: Entity is despawned

#### G10.3 **tick_phantom_lifetime does not affect non-phantom entities**
- Given: Entity with `PhantomLifetime(5.0)` but no `PhantomBolt` marker
- When: `tick_phantom_lifetime` runs
- Then: `PhantomLifetime` unchanged

### G11: tick_tether_beam_damage

#### G11.1 **tick_tether_beam_damage sends damage for cells intersecting beam**
- Given: TetherBeam entity with `TetherBeamSource { bolt_a: bolt1, bolt_b: bolt2 }`, `TetherBeamDamage(15.0)`. bolt1 at `(0.0, 0.0)`, bolt2 at `(200.0, 0.0)`. Cell at `(100.0, 0.0)` (on the line).
- When: `tick_tether_beam_damage` runs
- Then: `DamageDealt<Cell>` sent for the cell with damage `15.0`

#### G11.2 **tick_tether_beam_damage does not damage cells not on beam line**
- Given: TetherBeam entity with `TetherBeamSource { bolt_a: bolt1, bolt_b: bolt2 }`, `TetherBeamDamage(15.0)`. bolt1 at `(0.0, 0.0)`, bolt2 at `(200.0, 0.0)`. Cell at `(100.0, 100.0)` (far from line).
- When: `tick_tether_beam_damage` runs
- Then: No `DamageDealt<Cell>` sent

### G12: cleanup_tether_beams

#### G12.1 **cleanup_tether_beams despawns beam when bolt_a is gone**
- Given: TetherBeam entity with `TetherBeamSource { bolt_a: despawned_entity, bolt_b: living_bolt }`
- When: `cleanup_tether_beams` runs
- Then: TetherBeam entity is despawned

#### G12.2 **cleanup_tether_beams despawns beam when bolt_b is gone**
- Given: TetherBeam entity with `TetherBeamSource { bolt_a: living_bolt, bolt_b: despawned_entity }`
- When: `cleanup_tether_beams` runs
- Then: TetherBeam entity is despawned

#### G12.3 **cleanup_tether_beams keeps beam when both endpoints alive**
- Given: TetherBeam entity with `TetherBeamSource { bolt_a: living_bolt_a, bolt_b: living_bolt_b }`
- When: `cleanup_tether_beams` runs
- Then: TetherBeam entity still exists

### G13: tick_gravity_wells

#### G13.1 **tick_gravity_wells applies force toward well center**
- Given: GravityWell entity at `(100.0, 100.0)` with `GravityWellStrength(50.0)`, `GravityWellRadius(200.0)`, `GravityWellLifetime(5.0)`. Bolt at `(100.0, 150.0)` (distance 50.0, within radius) with velocity `(200.0, 0.0)`.
- When: `tick_gravity_wells` runs one frame
- Then: Bolt velocity has a component toward `(100.0, 100.0)` -- y-component decreased (pulled down toward well). `GravityWellLifetime` decremented by dt.

#### G13.2 **tick_gravity_wells does not affect bolts outside radius**
- Given: GravityWell at `(100.0, 100.0)` with `GravityWellRadius(50.0)`. Bolt at `(300.0, 300.0)` (distance ~283, outside radius).
- When: `tick_gravity_wells` runs
- Then: Bolt velocity unchanged.

#### G13.3 **tick_gravity_wells decrements lifetime**
- Given: GravityWell with `GravityWellLifetime(3.0)`. dt = `1.0/60.0`.
- When: `tick_gravity_wells` runs one frame
- Then: `GravityWellLifetime` is `3.0 - 1.0/60.0`

### G14: despawn_expired_wells

#### G14.1 **despawn_expired_wells despawns when lifetime reaches zero**
- Given: GravityWell entity with `GravityWellLifetime(0.0)`
- When: `despawn_expired_wells` runs
- Then: Entity is despawned

#### G14.2 **despawn_expired_wells despawns when lifetime is negative**
- Given: GravityWell entity with `GravityWellLifetime(-0.5)`
- When: `despawn_expired_wells` runs
- Then: Entity is despawned

#### G14.3 **despawn_expired_wells keeps well with positive lifetime**
- Given: GravityWell entity with `GravityWellLifetime(2.0)`
- When: `despawn_expired_wells` runs
- Then: Entity still exists

### G15: reset_ramping_damage

#### G15.1 **reset_ramping_damage resets accumulator to zero**
- Given: Entity with `RampingDamageAccumulator(OrderedFloat(25.0))`
- When: `reset_ramping_damage` runs (on NodeState::Playing enter)
- Then: `RampingDamageAccumulator` is `OrderedFloat(0.0)`

#### G15.2 **reset_ramping_damage does not remove the component**
- Given: Entity with `RampingDamageAccumulator(OrderedFloat(0.0))`
- When: `reset_ramping_damage` runs
- Then: `RampingDamageAccumulator` still present with value `OrderedFloat(0.0)`

#### G15.3 **reset_ramping_damage handles multiple entities**
- Given: Entity_a with `RampingDamageAccumulator(OrderedFloat(10.0))`, Entity_b with `RampingDamageAccumulator(OrderedFloat(30.0))`
- When: `reset_ramping_damage` runs
- Then: Both accumulators reset to `OrderedFloat(0.0)`

---

## Section H: Condition Evaluation

### H1: is_node_active

#### H1.1 **is_node_active returns true when NodeState::Playing**
- Given: World with `State<NodeState>` set to `NodeState::Playing`
- When: `is_node_active(world)` called
- Then: Returns `true`

#### H1.2 **is_node_active returns false when NodeState is not Playing**
- Given: World with `State<NodeState>` set to `NodeState::ChipSelect` (or any non-Playing state)
- When: `is_node_active(world)` called
- Then: Returns `false`
- Edge case: Test with multiple non-Playing states to confirm all return false

### H2: is_shield_active

#### H2.1 **is_shield_active returns true when at least one ShieldWall exists**
- Given: World with 1 entity having `ShieldWall` component
- When: `is_shield_active(world)` called
- Then: Returns `true`

#### H2.2 **is_shield_active returns true when multiple ShieldWalls exist**
- Given: World with 3 entities having `ShieldWall` component
- When: `is_shield_active(world)` called
- Then: Returns `true`

#### H2.3 **is_shield_active returns false when no ShieldWall exists**
- Given: World with no entities having `ShieldWall` component
- When: `is_shield_active(world)` called
- Then: Returns `false`

### H3: is_combo_active

#### H3.1 **is_combo_active returns true when streak meets threshold**
- Given: World with `ComboStreak { count: 5 }` resource
- When: `is_combo_active(world, 5)` called
- Then: Returns `true`

#### H3.2 **is_combo_active returns true when streak exceeds threshold**
- Given: World with `ComboStreak { count: 7 }` resource
- When: `is_combo_active(world, 5)` called
- Then: Returns `true`

#### H3.3 **is_combo_active returns false when streak below threshold**
- Given: World with `ComboStreak { count: 3 }` resource
- When: `is_combo_active(world, 5)` called
- Then: Returns `false`

#### H3.4 **is_combo_active returns false when streak is zero**
- Given: World with `ComboStreak { count: 0 }` resource
- When: `is_combo_active(world, 1)` called
- Then: Returns `false`
- Edge case: threshold = 0 -- returns true even with count 0

### H4: evaluate_conditions system

#### H4.1 **evaluate_conditions fires effects on false-to-true transition**
- Given: Entity with `BoundEffects` containing a During entry for `Condition::NodeActive` with `condition_active: Some(false)`. Scoped effect = `SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. World with `State<NodeState>` = `Playing`.
- When: `evaluate_conditions` runs
- Then: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }` is fired on the entity. `condition_active` updated to `Some(true)`.

#### H4.2 **evaluate_conditions reverses effects on true-to-false transition**
- Given: Entity with `BoundEffects` containing a During entry for `Condition::NodeActive` with `condition_active: Some(true)`. Scoped effect = `SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. World with `State<NodeState>` = `ChipSelect` (not Playing).
- When: `evaluate_conditions` runs
- Then: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }` is reversed on the entity. `condition_active` updated to `Some(false)`.

#### H4.3 **evaluate_conditions does nothing when condition unchanged (true -> true)**
- Given: Entity with `BoundEffects` containing a During entry for `Condition::ShieldActive` with `condition_active: Some(true)`. World with 1 `ShieldWall` entity (condition still true).
- When: `evaluate_conditions` runs
- Then: No fire or reverse calls. `condition_active` remains `Some(true)`.

#### H4.4 **evaluate_conditions does nothing when condition unchanged (false -> false)**
- Given: Entity with `BoundEffects` containing a During entry for `Condition::ShieldActive` with `condition_active: Some(false)`. World with no `ShieldWall` entities (condition still false).
- When: `evaluate_conditions` runs
- Then: No fire or reverse calls. `condition_active` remains `Some(false)`.

#### H4.5 **evaluate_conditions skips non-During entries**
- Given: Entity with `BoundEffects` containing a When entry (not During) with `condition_active: None`.
- When: `evaluate_conditions` runs
- Then: Entry is ignored entirely. No fire or reverse.

#### H4.6 **evaluate_conditions handles ComboActive with threshold**
- Given: Entity with `BoundEffects` containing a During entry for `Condition::ComboActive(3)` with `condition_active: Some(false)`. World with `ComboStreak { count: 5 }`.
- When: `evaluate_conditions` runs
- Then: Scoped effects fired. `condition_active` updated to `Some(true)`.

#### H4.7 **evaluate_conditions handles multiple During entries on same entity**
- Given: Entity with `BoundEffects` containing:
  - During entry for `NodeActive` with `condition_active: Some(false)` and scoped effect SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })
  - During entry for `ShieldActive` with `condition_active: Some(false)` and scoped effect DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) })
  World with `NodeState::Playing` and 1 `ShieldWall` entity.
- When: `evaluate_conditions` runs
- Then: Both transitions fire: SpeedBoost(OrderedFloat(1.5)) fired AND DamageBoost(OrderedFloat(2.0)) fired. Both `condition_active` updated to `Some(true)`.

#### H4.8 **evaluate_conditions processes multiple entities independently**
- Given: Entity_a with During `NodeActive` with `condition_active: Some(false)` and scoped SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }). Entity_b with During `NodeActive` with `condition_active: Some(true)` and scoped SizeBoost(SizeBoostConfig { multiplier: OrderedFloat(1.3) }). World with `NodeState::Playing`.
- When: `evaluate_conditions` runs
- Then: Entity_a gets SpeedBoost fired (false->true). Entity_b gets no change (true->true, no action).

---

## Section I: System Schedule Run Conditions

### I1: **Tick systems only run during NodeState::Playing**
- Given: Shockwave entity with `ShockwaveRadius(0.0)`, `ShockwaveSpeed(300.0)`. World NOT in `NodeState::Playing`.
- When: one frame runs
- Then: `ShockwaveRadius` is still `0.0` -- `tick_shockwave` did not run.
- Note: This applies to ALL tick systems in `EffectV3Systems::Tick`: tick_shockwave, sync_shockwave_visual, apply_shockwave_damage, despawn_finished_shockwave, tick_chain_lightning, tick_pulse, tick_phantom_lifetime, tick_tether_beam_damage, cleanup_tether_beams, tick_gravity_wells, despawn_expired_wells.

### I2: **evaluate_conditions runs after tick systems**
- Given: Shield fired this frame (ShieldWall spawned by tick_pulse or fire). During entry for ShieldActive with `condition_active: Some(false)`.
- When: FixedUpdate runs full schedule
- Then: `evaluate_conditions` sees the ShieldWall spawned by the tick system. `condition_active` transitions to `Some(true)`.
- Note: Ordering guarantee: `EffectV3Systems::Conditions` runs after `EffectV3Systems::Tick`.

---

## Section J: Dispatch Functions

### J1: fire_dispatch routes to correct effect fire

Each test verifies that `fire_dispatch` matches on the `EffectType` variant and delegates to the correct config's `fire` method. One representative test per effect category.

#### J1.1 **fire_dispatch routes passive effect (SpeedBoost)**
- Given: Entity with no `EffectStack<SpeedBoostConfig>`. `EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`.
- When: `fire_dispatch(EffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), entity, "chip_a", world)`
- Then: Entity has `EffectStack<SpeedBoostConfig>` with 1 entry `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`

#### J1.2 **fire_dispatch routes spawner effect (Shockwave)**
- Given: Entity at position `(50.0, 50.0)` with `BoltBaseDamage(10.0)`, empty `EffectStack<DamageBoostConfig>`.
- When: `fire_dispatch(EffectType::Shockwave(ShockwaveConfig { base_range: OrderedFloat(80.0), range_per_level: OrderedFloat(10.0), stacks: 1, speed: OrderedFloat(200.0) }), entity, "chip_s", world)`
- Then: A shockwave entity exists with `ShockwaveSource`, `ShockwaveMaxRadius(80.0)`, `ShockwaveSpeed(200.0)`

#### J1.3 **fire_dispatch routes toggle effect (FlashStep)**
- Given: Entity with no `FlashStepActive`.
- When: `fire_dispatch(EffectType::FlashStep(FlashStepConfig {}), entity, "chip_f", world)`
- Then: Entity has `FlashStepActive` component

#### J1.4 **fire_dispatch routes protector effect (Shield)**
- Given: Entity (bolt).
- When: `fire_dispatch(EffectType::Shield(ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }), entity, "chip_sh", world)`
- Then: A `ShieldWall` entity exists with `ShieldOwner(entity)`, `ShieldDuration(10.0)`

#### J1.5 **fire_dispatch routes stateful effect (CircuitBreaker)**
- Given: Entity with no `CircuitBreakerCounter`.
- When: `fire_dispatch(EffectType::CircuitBreaker(CircuitBreakerConfig { bumps_required: 3, spawn_count: 2, inherit: true, shockwave_range: OrderedFloat(150.0), shockwave_speed: OrderedFloat(300.0) }), entity, "chip_cb", world)`
- Then: Entity has `CircuitBreakerCounter { remaining: 2, bumps_required: 3, ... }`

#### J1.6 **fire_dispatch routes message effect (LoseLife)**
- Given: Entity (breaker).
- When: `fire_dispatch(EffectType::LoseLife(LoseLifeConfig {}), entity, "chip_ll", world)`
- Then: `DamageDealt<Breaker>` message sent

#### J1.7 **fire_dispatch routes meta effect (RandomEffect)**
- Given: Entity. Pool = `[(OrderedFloat(1.0), SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))]`.
- When: `fire_dispatch(EffectType::RandomEffect(RandomEffectConfig { pool: [...] }), entity, "chip_r", world)`
- Then: Entity has `EffectStack<SpeedBoostConfig>` with 1 entry (the delegated fire ran)

### J2: reverse_dispatch routes to correct effect reverse

Each test verifies that `reverse_dispatch` matches on the `ReversibleEffectType` variant and delegates to the correct config's `reverse` method. Representative tests only.

#### J2.1 **reverse_dispatch routes passive effect (SpeedBoost)**
- Given: Entity with `EffectStack<SpeedBoostConfig>` containing `[("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`.
- When: `reverse_dispatch(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), entity, "chip_a", world)`
- Then: `EffectStack<SpeedBoostConfig>` is empty

#### J2.2 **reverse_dispatch routes toggle effect (FlashStep)**
- Given: Entity with `FlashStepActive`.
- When: `reverse_dispatch(ReversibleEffectType::FlashStep(FlashStepConfig {}), entity, "chip_f", world)`
- Then: Entity no longer has `FlashStepActive`

#### J2.3 **reverse_dispatch routes protector effect (Shield)**
- Given: World with `ShieldWall` entity having `ShieldOwner(entity)`.
- When: `reverse_dispatch(ReversibleEffectType::Shield(ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }), entity, "chip_sh", world)`
- Then: ShieldWall entity despawned

#### J2.4 **reverse_dispatch routes stateful effect (Anchor)**
- Given: Entity with `AnchorActive { bump_force_multiplier: 2.0, perfect_window_multiplier: 1.5, plant_delay: 1.0, source: "chip_anchor".to_string() }`.
- When: `reverse_dispatch(ReversibleEffectType::Anchor(AnchorConfig { bump_force_multiplier: OrderedFloat(2.0), perfect_window_multiplier: OrderedFloat(1.5), plant_delay: OrderedFloat(1.0) }), entity, "chip_anchor", world)`
- Then: Entity no longer has `AnchorActive`

---

## Section K: Despawned-Entity Guard Tests

These tests verify that fire/reverse on a despawned entity does not panic. The entity ID exists in the world but has been despawned before the fire/reverse call runs.

### K1: Passive effect fire on despawned entity

#### K1.1 **SpeedBoost fire on despawned entity does not panic**
- Given: Entity spawned, then despawned (entity ID is stale)
- When: `SpeedBoostConfig { multiplier: OrderedFloat(1.5) }.fire(entity, "chip_a", world)`
- Then: No panic. No component inserted (entity does not exist).
- Edge case: Same applies to `reverse` on a stale entity -- no panic

### K2: Toggle effect fire on despawned entity

#### K2.1 **FlashStep fire on despawned entity does not panic**
- Given: Entity spawned, then despawned (entity ID is stale)
- When: `FlashStepConfig {}.fire(entity, "chip_flash", world)`
- Then: No panic. No `FlashStepActive` inserted.

#### K2.2 **FlashStep reverse on despawned entity does not panic**
- Given: Entity spawned, then despawned (entity ID is stale)
- When: `FlashStepConfig {}.reverse(entity, "chip_flash", world)`
- Then: No panic.

### K3: Spawner effect fire on despawned entity

#### K3.1 **Shockwave fire on despawned entity does not panic**
- Given: Entity spawned at `(100.0, 200.0)` with `BoltBaseDamage(10.0)`, then despawned (entity ID is stale)
- When: `ShockwaveConfig { base_range: OrderedFloat(50.0), range_per_level: OrderedFloat(10.0), stacks: 1, speed: OrderedFloat(300.0) }.fire(entity, "chip_shock", world)`
- Then: No panic. No shockwave entity spawned (source entity does not exist, cannot read position).

#### K3.2 **SpawnBolts fire on despawned entity does not panic**
- Given: Entity spawned, then despawned (entity ID is stale)
- When: `SpawnBoltsConfig { count: 3, lifespan: None, inherit: false }.fire(entity, "chip_spawn", world)`
- Then: No panic. No bolt entities spawned.

### K4: Protector effect reverse on despawned entity

#### K4.1 **Shield reverse on despawned entity does not panic**
- Given: Entity spawned, then despawned. A `ShieldWall` entity exists with `ShieldOwner(entity)` (references the stale entity).
- When: `ShieldConfig { duration: OrderedFloat(10.0), reflection_cost: OrderedFloat(2.0) }.reverse(entity, "chip_shield", world)`
- Then: No panic. The ShieldWall matching the stale owner is still despawned correctly (reverse queries by ShieldOwner value, not by whether owner is alive).

---

## Types (referenced, from design docs -- all defined in earlier waves)

### Config Structs (wave 2 scaffold)
- `SpeedBoostConfig { multiplier: OrderedFloat<f32> }` -- derives Clone, PartialEq, Eq, Serialize, Deserialize
- `SizeBoostConfig { multiplier: OrderedFloat<f32> }`
- `DamageBoostConfig { multiplier: OrderedFloat<f32> }`
- `BumpForceConfig { multiplier: OrderedFloat<f32> }`
- `QuickStopConfig { multiplier: OrderedFloat<f32> }`
- `VulnerableConfig { multiplier: OrderedFloat<f32> }`
- `PiercingConfig { charges: u32 }`
- `RampingDamageConfig { increment: OrderedFloat<f32> }`
- `FlashStepConfig {}`
- `ShieldConfig { duration: OrderedFloat<f32>, reflection_cost: OrderedFloat<f32> }`
- `SecondWindConfig {}`
- `PulseConfig { base_range: OrderedFloat<f32>, range_per_level: OrderedFloat<f32>, stacks: u32, speed: OrderedFloat<f32>, interval: OrderedFloat<f32> }`
- `AnchorConfig { bump_force_multiplier: OrderedFloat<f32>, perfect_window_multiplier: OrderedFloat<f32>, plant_delay: OrderedFloat<f32> }`
- `CircuitBreakerConfig { bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: OrderedFloat<f32>, shockwave_speed: OrderedFloat<f32> }`
- `EntropyConfig { max_effects: u32, pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`
- `AttractionConfig { attraction_type: AttractionType, force: OrderedFloat<f32>, max_force: Option<OrderedFloat<f32>> }`
- `ShockwaveConfig { base_range: OrderedFloat<f32>, range_per_level: OrderedFloat<f32>, stacks: u32, speed: OrderedFloat<f32> }`
- `ExplodeConfig { range: OrderedFloat<f32>, damage: OrderedFloat<f32> }`
- `ChainLightningConfig { arcs: u32, range: OrderedFloat<f32>, damage_mult: OrderedFloat<f32>, arc_speed: OrderedFloat<f32> }`
- `PiercingBeamConfig { damage_mult: OrderedFloat<f32>, width: OrderedFloat<f32> }`
- `SpawnBoltsConfig { count: u32, lifespan: Option<OrderedFloat<f32>>, inherit: bool }`
- `SpawnPhantomConfig { duration: OrderedFloat<f32>, max_active: u32 }`
- `ChainBoltConfig { tether_distance: OrderedFloat<f32> }`
- `MirrorConfig { inherit: bool }`
- `TetherBeamConfig { damage_mult: OrderedFloat<f32>, chain: bool }`
- `GravityWellConfig { strength: OrderedFloat<f32>, duration: OrderedFloat<f32>, radius: OrderedFloat<f32>, max: u32 }`
- `LoseLifeConfig {}`
- `TimePenaltyConfig { seconds: OrderedFloat<f32> }`
- `DieConfig {}`
- `RandomEffectConfig { pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }`

### Component Types (wave 2 scaffold)
- `EffectStack<T: PassiveEffect>` -- Component, Default. Fields: `entries: Vec<(String, T)>`
- `PiercingRemaining(u32)` -- Component
- `RampingDamageAccumulator(OrderedFloat<f32>)` -- Component
- `FlashStepActive` -- Component (marker)
- `ShieldWall` -- Component (marker)
- `ShieldOwner(Entity)` -- Component
- `ShieldDuration(f32)` -- Component
- `ShieldReflectionCost(f32)` -- Component
- `SecondWindWall` -- Component (marker)
- `SecondWindOwner(Entity)` -- Component
- `PulseEmitter { base_range: f32, range_per_level: f32, stacks: u32, speed: f32, interval: f32, timer: f32 }` -- Component
- `AnchorActive { bump_force_multiplier: f32, perfect_window_multiplier: f32, plant_delay: f32, source: String }` -- Component (NOTE: `source` field stores the chip source string for push/remove of BumpForceConfig entries by tick_anchor)
- `AnchorTimer(f32)` -- Component
- `AnchorPlanted` -- Component (marker)
- `CircuitBreakerCounter { remaining: u32, bumps_required: u32, spawn_count: u32, inherit: bool, shockwave_range: f32, shockwave_speed: f32 }` -- Component
- `EntropyCounter { count: u32, max_effects: u32, pool: Vec<(OrderedFloat<f32>, Box<EffectType>)> }` -- Component
- `ActiveAttractions(Vec<AttractionEntry>)` -- Component
- `AttractionEntry { source: String, attraction_type: AttractionType, force: f32, max_force: Option<f32> }`
- `ShockwaveSource` -- Component (marker)
- `ShockwaveRadius(f32)` -- Component
- `ShockwaveMaxRadius(f32)` -- Component
- `ShockwaveSpeed(f32)` -- Component
- `ShockwaveDamaged(HashSet<Entity>)` -- Component
- `ShockwaveBaseDamage(f32)` -- Component
- `ShockwaveDamageMultiplier(f32)` -- Component
- `EffectSourceChip(Option<String>)` -- Component
- `ChainLightningChain { remaining_jumps: u32, damage: f32, hit_set: HashSet<Entity>, state: ChainState, range: f32, arc_speed: f32, source_pos: Vec2 }` -- Component
- `ChainState` -- enum: `Idle`, `ArcTraveling { target: Entity, target_pos: Vec2, arc_entity: Entity, arc_pos: Vec2 }`
- `PhantomBolt` -- Component (marker)
- `PhantomLifetime(f32)` -- Component
- `PhantomOwner(Entity)` -- Component
- `TetherBeamSource { bolt_a: Entity, bolt_b: Entity }` -- Component
- `TetherBeamDamage(f32)` -- Component
- `GravityWellSource` -- Component (marker)
- `GravityWellStrength(f32)` -- Component
- `GravityWellRadius(f32)` -- Component
- `GravityWellLifetime(f32)` -- Component
- `GravityWellOwner(Entity)` -- Component
- `BoltBaseDamage(f32)` -- Component
- `ComboStreak { count: u32 }` -- Resource

### Message Types
- `ApplyTimePenalty { seconds: f32 }` -- Message
- `DamageDealt<T: GameEntity>` -- Message (from death pipeline, wave 7)
- `KillYourself<T: GameEntity>` -- Message (from death pipeline, wave 7)
- `BoltImpactWall { bolt: Entity, wall: Entity }` -- Message (existing)

### Enum Types
- `EffectType` -- 30-variant enum for fire_dispatch (all fire-capable effects)
- `ReversibleEffectType` -- 16-variant enum for reverse_dispatch (only reversible effects)
- `Condition` -- enum: `NodeActive`, `ShieldActive`, `ComboActive(u32)`
- `AttractionType` -- enum: `Breaker`, `Bolt`, `Cell`, `Wall`
- `EntityKind` -- enum: `Cell`, `Bolt`, `Wall`, `Breaker`, `Any`
- `NodeState` -- enum (existing): includes `Playing`, `ChipSelect`, etc.

---

## Messages (new to this wave)
- No new message types. All messages referenced above are defined in earlier waves (wave 2 scaffold or wave 7 death pipeline). `ApplyTimePenalty` is defined in wave 2. `DamageDealt<T>` and `KillYourself<T>` are from the death pipeline (wave 7 -- these may need to be stubbed during wave 6 RED phase).

---

## Reference Files
- `docs/todos/detail/effect-refactor/migration/new-effect-implementations/` -- per-effect behavioral specs (ONLY source of truth for fire/reverse behavior)
- `docs/todos/detail/effect-refactor/creating-effects/effect-api/fireable.md` -- Fireable trait contract
- `docs/todos/detail/effect-refactor/creating-effects/effect-api/reversible.md` -- Reversible trait contract
- `docs/todos/detail/effect-refactor/creating-effects/effect-api/passive-effect.md` -- PassiveEffect trait contract
- `docs/todos/detail/effect-refactor/evaluating-conditions/` -- condition evaluation specs
- `docs/todos/detail/effect-refactor/rust-types/components/` -- component definitions
- `docs/todos/detail/effect-refactor/rust-types/effect-stacking/` -- EffectStack and aggregation
- `docs/todos/detail/effect-refactor/migration/plugin-wiring/system-sets.md` -- system set definitions and ordering

---

## Scenario Coverage

- New invariants: none -- existing invariants (`ShieldWallAtMostOne`, `SecondWindWallAtMostOne`, `PulseRingAccumulation`, `ChainArcCountReasonable`, `GravityWellCountReasonable`) cover these effects
- New scenarios: none at this wave -- scenario coverage is evaluated at wave 14 (Full Verification Tier)
- Self-test scenarios: none needed -- existing self-tests cover the invariants referenced above
- Layout updates: none

---

## Constraints

### Test File Locations
Tests are organized per-effect in directory module layout (`effects/<name>/tests.rs`):

**Passive effects**: `src/effect_v3/effects/<name>/tests.rs`
- `speed_boost/tests.rs`, `size_boost/tests.rs`, `damage_boost/tests.rs`, `bump_force/tests.rs`, `quick_stop/tests.rs`, `vulnerable/tests.rs`, `piercing/tests.rs`, `ramping_damage/tests.rs`

**Toggle effects**: `src/effect_v3/effects/flash_step/tests.rs`

**Protector effects**: `src/effect_v3/effects/shield/tests.rs`, `second_wind/tests.rs`, `pulse/tests.rs`

**Stateful effects**: `src/effect_v3/effects/anchor/tests.rs`, `circuit_breaker/tests.rs`, `entropy_engine/tests.rs`, `attraction/tests.rs`

**Spawner effects**: `src/effect_v3/effects/shockwave/tests.rs`, `explode/tests.rs`, `chain_lightning/tests.rs`, `piercing_beam/tests.rs`, `spawn_bolts/tests.rs`, `spawn_phantom/tests.rs`, `chain_bolt/tests.rs`, `mirror_protocol/tests.rs`, `tether_beam/tests.rs`, `gravity_well/tests.rs`

**Message/meta effects**: `src/effect_v3/effects/lose_life/tests.rs`, `time_penalty/tests.rs`, `die/tests.rs`, `random_effect/tests.rs`

**Tick systems**: Tests co-located in the same effect directory's `tests.rs`:
- `shockwave/tests.rs` -- tick_shockwave, sync_shockwave_visual, apply_shockwave_damage, despawn_finished_shockwave
- `chain_lightning/tests.rs` -- tick_chain_lightning
- `anchor/tests.rs` -- tick_anchor
- `attraction/tests.rs` -- apply_attraction
- `pulse/tests.rs` -- tick_pulse
- `shield/tests.rs` -- tick_shield_duration
- `spawn_phantom/tests.rs` -- tick_phantom_lifetime (note: phantom_bolt directory, tests.rs)
- `tether_beam/tests.rs` -- tick_tether_beam_damage, cleanup_tether_beams
- `gravity_well/tests.rs` -- tick_gravity_wells, despawn_expired_wells

**Reset systems**: `src/effect_v3/effects/ramping_damage/tests.rs`

**Condition evaluators**: `src/effect_v3/conditions/tests.rs` or per-file:
- `src/effect_v3/conditions/node_active.rs` (tests inline or adjacent)
- `src/effect_v3/conditions/shield_active.rs`
- `src/effect_v3/conditions/combo_active.rs`
- `src/effect_v3/conditions/evaluate_conditions/tests.rs`

**Dispatch functions**:
- `src/effect_v3/dispatch/fire_dispatch/tests.rs`
- `src/effect_v3/dispatch/reverse_dispatch/tests.rs`

**Despawned-entity guard tests**: Co-located in each effect's `tests.rs` (e.g., K1 in `speed_boost/tests.rs`, K2 in `flash_step/tests.rs`, K3 in `shockwave/tests.rs` and `spawn_bolts/tests.rs`, K4 in `shield/tests.rs`)

### Do NOT Test
- Trigger bridge systems (wave 5 scope)
- tick_effect_timers (wave 5 scope)
- Death pipeline systems: apply_damage, detect_deaths, process_despawn (wave 7 scope)
- EffectStack generic methods: push, remove, aggregate (wave 4 scope -- tested as non-system functions)
- Tree walking: walk_effects (wave 4 scope)
- Plugin wiring / system registration (wave 2 scaffold)
- Visual/rendering output (manual playtesting only)
- Downstream consumers: bolt velocity system reading SpeedBoost stack, collision system reading DamageBoost stack, breaker movement reading QuickStop/FlashStep (these are tested in their respective domain waves)
- RON deserialization of config structs (wave 3 scope)
