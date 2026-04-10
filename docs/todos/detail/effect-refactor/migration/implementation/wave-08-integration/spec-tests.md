# Test Spec: Effect + Death Pipeline — Integration Tests (Wave 8)

## Domain
`src/effect/` (primary), `src/shared/` (death pipeline), cross-domain bridges

## Overview

Integration tests that exercise end-to-end flows across system boundaries: trigger dispatch into tree walking into effect firing, multi-stage arming across frames, timer lifecycles, condition lifecycles, Once consumption, On redirection, the full death pipeline, die bypass, cascade delay, global triggers with On resolution, passive stacking with partial removal, and spawn watchers. Each test sets up a minimal Bevy App with the required systems and verifies observable state changes after running one or more FixedUpdate ticks.

---

## Behavior

### 1. **Bump triggers walk BoundEffects and fire effects**

- Given: A bolt entity with `BoundEffects` containing one entry: `BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }`. The bolt has `EffectStack<SpeedBoostConfig>` initialized empty (default). A breaker entity exists.
- When: `BumpPerformed { grade: BumpGrade::Perfect, bolt: bolt_entity }` message is sent. The `on_bumped` bridge system runs (dispatches `Trigger::Bumped` with `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }` on the bolt). Then `walk_effects` evaluates the bolt's BoundEffects. The `fire_effect` command is applied.
- Then: `EffectStack<SpeedBoostConfig>` on the bolt contains exactly one entry: `("chip_a", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. `aggregate()` returns `1.5`. The BoundEntry remains in BoundEffects (When re-arms, not consumed).
- Edge case: If the bolt has no BoundEffects entries matching `Bumped` (e.g., only `When(Impacted(Cell), ...)`), the stack remains empty after the bump.

### 2. **Multi-stage arming: outer When arms inner When into StagedEffects**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_b", tree: When(Bumped, When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(2.0) })))), condition_active: None }`. The bolt has empty `StagedEffects` and empty `EffectStack<SpeedBoostConfig>`.
- When: `Trigger::Bumped` is dispatched on the bolt (via `BumpPerformed`). The outer When matches. The inner tree is `When(Impacted(Cell), Fire(...))` which is a trigger gate. The walker calls `stage_effect(bolt, "chip_b", When(Impacted(Cell), Fire(SpeedBoost(...))))`.
- Then: `StagedEffects` on the bolt now contains one entry: `("chip_b", When(Impacted(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(2.0) }))))`. `EffectStack<SpeedBoostConfig>` is still empty (no Fire happened yet). BoundEffects still has the original When entry (not consumed).
- Edge case: After arming, dispatch `Trigger::Impacted(EntityKind::Cell)` on the bolt. The staged entry matches, the inner Fire evaluates, `EffectStack<SpeedBoostConfig>` now has one entry with multiplier 2.0, and the staged entry is consumed (removed from StagedEffects).

### 3. **Bumped twice: same trigger in outer and inner When requires two separate bumps**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_c", tree: When(Bumped, When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(3.0) })))), condition_active: None }`. Empty `StagedEffects`, empty `EffectStack<SpeedBoostConfig>`.
- When: First `Trigger::Bumped` is dispatched on the bolt. The outer When matches. Inner tree `When(Bumped, Fire(...))` is a trigger gate, so it is armed into StagedEffects via `stage_effect`. Walking proceeds to StagedEffects but the walker processes Staged FIRST then Bound, and the staged entry was just added via deferred command so it does NOT exist yet in this frame's walk.
- Then: After the first bump: StagedEffects has one entry `("chip_c", When(Bumped, Fire(...)))`. EffectStack is empty. No fire happened.
- When: Second `Trigger::Bumped` is dispatched on the bolt in a subsequent frame. The walker iterates StagedEffects first. The staged `When(Bumped, Fire(...))` matches. Fire evaluates. The staged entry is consumed.
- Then: After the second bump: `EffectStack<SpeedBoostConfig>` has one entry with multiplier 3.0. StagedEffects is empty (consumed). BoundEffects still has the original outer When (it re-arms, so a third and fourth bump would repeat the cycle).
- Edge case: After the second bump, the BoundEffects outer When re-arms. A third bump should arm a fresh inner When into StagedEffects again. A fourth bump should fire again. Verify this cyclical behavior.

### 4. **Until timer lifecycle: fire immediately, reverse after time expires**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_d", tree: Until(TimeExpires(OrderedFloat(0.5)), Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(2.0) })))), condition_active: None }`. Empty `EffectStack<SpeedBoostConfig>`. No `EffectTimers` component.
- When: The Until is first encountered during installation (stamp_effect or initial dispatch). The walker fires the scoped effect immediately and registers a timer.
- Then: `EffectStack<SpeedBoostConfig>` on bolt has one entry `("chip_d", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })`. `EffectTimers` component is inserted on the bolt with one entry `(OrderedFloat(0.5), OrderedFloat(0.5))`.
- When: `tick_effect_timers` runs for 0.5 seconds of game time (e.g., 30 frames at dt=1/60, or a single tick with dt=0.5). The timer reaches 0. `EffectTimerExpired { entity: bolt }` is sent. `on_time_expires` bridge dispatches `Trigger::TimeExpires(OrderedFloat(0.5))` on the bolt (Self scope).
- Then: The Until's trigger matches. `reverse_effect` is called. `EffectStack<SpeedBoostConfig>` on the bolt is now empty. The Until entry is removed from BoundEffects via `remove_effect`. EffectTimers entry is removed (and if empty, the component is removed).
- Edge case: Two Until entries with the same `TimeExpires(0.5)` duration on the same bolt. Both fire immediately, both create timer entries. When 0.5s expires, both reverse independently. Verify both stacks are empty and both entries are removed.

### 5. **During condition lifecycle: fire on true, reverse on false, re-fire on true again**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_e", tree: During(NodeActive, Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.8) })))), condition_active: Some(false) }`. Empty `EffectStack<SpeedBoostConfig>`. The world's node state is NOT active (e.g., `NodeState` resource indicates not playing).
- When: The node becomes active (NodeState transitions to Playing). `evaluate_conditions` runs and detects the transition from false to true.
- Then: `EffectStack<SpeedBoostConfig>` has one entry `("chip_e", SpeedBoostConfig { multiplier: OrderedFloat(1.8) })`. `condition_active` on the BoundEntry is now `Some(true)`.
- When: The node ends (NodeState transitions away from Playing). `evaluate_conditions` detects true-to-false transition.
- Then: `EffectStack<SpeedBoostConfig>` is empty. `condition_active` is `Some(false)`. The BoundEntry is still in BoundEffects (During entries persist).
- When: The node becomes active again.
- Then: `EffectStack<SpeedBoostConfig>` has one entry again. Verifies During cycling.
- Edge case: If the condition is already true when the During entry is first installed (condition_active starts as `Some(false)` but the world condition is true), evaluate_conditions should detect the initial false-to-true transition and fire. This is the "first frame" behavior.

### 6. **Once consumption: fires on first trigger match, removed on second**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_f", tree: Once(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }`. Empty `EffectStack<SpeedBoostConfig>`. BoundEffects has length 1.
- When: First `Trigger::Bumped` dispatched on bolt.
- Then: `EffectStack<SpeedBoostConfig>` has one entry with multiplier 1.5. The Once entry is removed from BoundEffects via `remove_effect` (deferred). After command flush, BoundEffects has length 0.
- When: Second `Trigger::Bumped` dispatched on bolt in a subsequent frame.
- Then: `EffectStack<SpeedBoostConfig>` still has exactly one entry (from the first bump). No new entry added. BoundEffects remains empty.
- Edge case: A Once and a When with the same trigger `Bumped` in BoundEffects on the same entity. First bump fires both. Second bump fires only the When. Verify the Once is consumed and the When persists.

### 7. **Sequence ordering: all terminals fire in order**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_g", tree: When(Bumped, Sequence([Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })), Fire(DamageBoost(DamageBoostConfig { multiplier: OrderedFloat(2.0) }))])), condition_active: None }`. Empty `EffectStack<SpeedBoostConfig>`, empty `EffectStack<DamageBoostConfig>`.
- When: `Trigger::Bumped` dispatched on bolt.
- Then: `EffectStack<SpeedBoostConfig>` has one entry with multiplier 1.5. `EffectStack<DamageBoostConfig>` has one entry with multiplier 2.0. Both stacks populated from a single trigger dispatch.
- Edge case: A Sequence with three identical `Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.2) }))` entries. After the trigger, `EffectStack<SpeedBoostConfig>` has three entries, all with source "chip_g" and multiplier 1.2. `aggregate()` returns `1.2 * 1.2 * 1.2 = 1.728`.

### 8. **On redirection: effect fires on participant, not owner**

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_h", tree: When(Bumped, On(Bump(Breaker), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))), condition_active: None }`. A breaker entity with empty `EffectStack<SpeedBoostConfig>`. The bolt also has empty `EffectStack<SpeedBoostConfig>`.
- When: `Trigger::Bumped` dispatched on the bolt with `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }`.
- Then: `EffectStack<SpeedBoostConfig>` on the **breaker** has one entry `("chip_h", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. `EffectStack<SpeedBoostConfig>` on the **bolt** is still empty. The On node resolved `Bump(Breaker)` to the breaker entity from the trigger context.
- Edge case: `On(Bump(Bolt))` inside the same tree structure resolves to the bolt entity itself (since it is the bolt that was bumped). The fire would target the bolt. Verify self-targeting works correctly.
- Edge case: `On(Death(Killer))` with `TriggerContext::Death { victim, killer: None }` (environmental death). The On should be skipped entirely, no fire occurs on any entity.

### 9. **Death pipeline end-to-end: DamageDealt to despawn**

- Given: A cell entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, `Cell` component, no `Dead` component. A bolt entity exists.
- When: `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 1.0, source_chip: Some("chip_i".to_string()), _marker: PhantomData }` message is sent.
- When: `apply_damage::<Cell>` runs. Hp.current goes from 1.0 to 0.0. Since Hp crossed from positive to zero, `KilledBy.dealer` is set to `Some(bolt_entity)`.
- When: `detect_cell_deaths` runs. Cell has `Hp.current <= 0` and no `Dead` marker. `KillYourself<Cell> { victim: cell_entity, killer: Some(bolt_entity), _marker: PhantomData }` is sent.
- When: Domain kill handler processes `KillYourself<Cell>`. Inserts `Dead` on cell_entity. Sends `Destroyed<Cell> { victim: cell_entity, killer: Some(bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)), _marker: PhantomData }`. Sends `DespawnEntity { entity: cell_entity }`.
- When: `on_destroyed::<Cell>` bridge dispatches `Trigger::Died` on cell_entity (Local), `Trigger::Killed(EntityKind::Cell)` on bolt_entity (Local), `Trigger::DeathOccurred(EntityKind::Cell)` globally.
- When: `process_despawn_requests` runs in PostFixedUpdate. Cell entity is despawned.
- Then: Cell entity no longer exists in the world. Bolt entity still exists. The full chain DamageDealt -> apply_damage -> detect_deaths -> KillYourself -> domain handler -> Destroyed -> triggers -> DespawnEntity -> despawn completed in one frame (except the death bridge reads Destroyed next frame per the standard Bevy message persistence pattern).
- Edge case: Cell with `Hp { current: 2.0, ... }` receiving `DamageDealt` with `amount: 1.0`. Hp goes to 1.0. `detect_cell_deaths` does NOT send KillYourself (Hp > 0). Cell survives. Send another `DamageDealt` with `amount: 1.0`. Now Hp reaches 0. Death pipeline fires.

### 10. **Die bypass: Fire(Die) sends KillYourself directly, no Hp change**

- Given: A cell entity with `Hp { current: 5.0, starting: 5.0, max: None }`, `Cell` component, no `Dead`, `KilledBy { dealer: None }`. A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_j", tree: When(Bumped, On(Impact(Impactee), Fire(Die(DieConfig {})))), condition_active: None }`. TriggerContext must carry impact info — actually, Die is fired via `When(Bumped, ...)` with Bump context. The On uses `Impact(Impactee)` which requires Impact context. **Correction**: For Die to target the cell, the tree must use a death-compatible context. The plan says "When(Bumped, Fire(Die))" — this fires Die on the **owner** (the bolt), not on a cell. Let me re-read the plan.

  **Re-reading plan**: "When(Bumped, Fire(Die)). KillYourself sent directly, no Hp change." This means the bolt itself has `When(Bumped, Fire(Die))` — when bumped, the bolt sends itself into the death pipeline.

- Given (corrected): A bolt entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `Bolt` component, `KilledBy { dealer: None }`, no `Dead`. BoundEffects containing: `BoundEntry { source: "chip_j", tree: When(Bumped, Fire(Die(DieConfig {}))), condition_active: None }`.
- When: `Trigger::Bumped` dispatched on bolt. Fire evaluates `Die(DieConfig {})`. The `DieConfig::fire` inspects the bolt entity, finds `Bolt` component, sends `KillYourself<Bolt> { victim: bolt_entity, killer: None, _marker: PhantomData }`.
- Then: `KillYourself<Bolt>` is in the message queue. `Hp` on the bolt is still `{ current: 3.0, starting: 3.0, max: None }` — Die does NOT change Hp. The bolt's Hp is irrelevant; it dies from KillYourself regardless. After the domain handler processes KillYourself, `Dead` is inserted, `Destroyed<Bolt>` is sent, and `DespawnEntity` follows.
- Edge case: An entity with no `Hp` component. `Fire(Die)` still sends `KillYourself` — Die does not check Hp. The entity enters the death pipeline via KillYourself, bypassing the Hp-based detection entirely.

### 11. **Cascade delay: death-triggered DamageDealt is processed next frame**

- Given: Cell A at position (100.0, 200.0) with `Hp { current: 1.0, ... }`, `Cell`, `KilledBy { dealer: None }`. Cell B at position (120.0, 200.0) with `Hp { current: 1.0, ... }`, `Cell`, `KilledBy { dealer: None }`. Cell A has `BoundEffects` containing: `BoundEntry { source: "chip_k", tree: When(Died, On(Death(Killer), Fire(Shockwave(ShockwaveConfig { ... })))), condition_active: None }` (or some effect that sends `DamageDealt<Cell>` targeting Cell B). For simplicity, use a tree that fires `DamageDealt<Cell>` on death.

  **Clarification**: The cascade delay test verifies that when Cell A dies and its death trigger fires an effect that sends `DamageDealt<Cell>` targeting Cell B, that damage is NOT processed in the same frame. Per the system ordering docs:
  - Frame N: Game systems send DamageDealt for Cell A. ApplyDamage decrements A's Hp. DetectDeaths sends KillYourself for A. Domain handler sends Destroyed for A.
  - Frame N+1: `on_destroyed::<Cell>` bridge reads `Destroyed<Cell>` (persisted from frame N). Dispatches `Trigger::Died` on Cell A. Cell A's tree fires an effect that sends `DamageDealt<Cell>` for Cell B. This DamageDealt is consumed by ApplyDamage in the same frame N+1 (Bridge runs before ApplyDamage). So the damage to Cell B IS processed in frame N+1.
  - The key assertion: Cell B is NOT damaged in frame N (the frame Cell A took its fatal hit). Cell B IS damaged in frame N+1.

- Given: Cell A with `Hp { current: 1.0, starting: 1.0, max: None }`, Cell B with `Hp { current: 1.0, starting: 1.0, max: None }`. Cell A has a death-triggered effect tree that, upon Died trigger, fires an effect producing `DamageDealt<Cell> { target: cell_b, amount: 1.0, ... }`. A bolt entity exists as the initial killer.
- When: Frame 1: `DamageDealt<Cell> { target: cell_a, amount: 1.0, dealer: Some(bolt) }` is sent. `apply_damage::<Cell>` processes it. Cell A Hp -> 0. `detect_cell_deaths` sends KillYourself. Domain handler sends `Destroyed<Cell>` for Cell A.
- Then: After frame 1: Cell A is dead (has `Dead` marker). Cell B Hp is still 1.0 — no damage processed on B this frame. The `Destroyed<Cell>` for Cell A is in the message queue.
- When: Frame 2: `on_destroyed::<Cell>` bridge reads `Destroyed<Cell>` for A. Dispatches `Trigger::Died` on Cell A. Cell A's tree fires and produces `DamageDealt<Cell> { target: cell_b, amount: 1.0 }`. Since Bridge runs before ApplyDamage, this DamageDealt is processed by `apply_damage::<Cell>` in the same frame 2.
- Then: After frame 2: Cell B Hp is 0. `detect_cell_deaths` sends KillYourself for Cell B. Cell B enters the death pipeline.
- Edge case: If the death-triggered effect fires `DamageDealt` in the Bridge set (same frame), and ApplyDamage also runs in the same frame (it does, after Bridge), the cascade IS processed in frame N+1 total from the initial damage. Verify the one-frame delay from initial damage to cascade damage.

### 12. **Global trigger with On: DeathOccurred(Cell) with On(Death(Killer)) fires on killer**

- Given: A cell entity (victim) at position (100.0, 200.0) with `Cell` component. A bolt entity (killer) at position (50.0, 300.0) with `Bolt` component and empty `EffectStack<SpeedBoostConfig>`. A third entity (observer) with `BoundEffects` containing: `BoundEntry { source: "chip_l", tree: When(DeathOccurred(Cell), On(Death(Killer), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) })))), condition_active: None }`. This observer entity could be any entity — perhaps the breaker, perhaps a cell, perhaps the bolt itself.
- When: `Destroyed<Cell> { victim: cell_entity, killer: Some(bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)) }` is sent. `on_destroyed::<Cell>` dispatches `Trigger::DeathOccurred(EntityKind::Cell)` globally with `TriggerContext::Death { victim: cell_entity, killer: Some(bolt_entity) }`.
- Then: The observer's BoundEffects are walked. The When matches `DeathOccurred(Cell)`. The On resolves `Death(Killer)` to `bolt_entity` from the TriggerContext. `fire_effect(bolt_entity, SpeedBoost(...), "chip_l")` is called. `EffectStack<SpeedBoostConfig>` on the **bolt** has one entry `("chip_l", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. The observer entity's own stack is unchanged.
- Edge case: DeathOccurred(Cell) fires on ALL entities with BoundEffects. If both the observer and another entity have matching When(DeathOccurred(Cell), ...) trees, both fire. Verify both observers independently resolve On targets and fire effects.
- Edge case: `On(Death(Killer))` with an environmental death (`killer: None` in context). The On is skipped. No fire_effect called. EffectStack on all entities remains empty.

### 13. **Passive stacking: two sources, remove one, aggregate reflects remainder**

- Given: A bolt entity with `EffectStack<SpeedBoostConfig>` containing two entries: `[("chip_m", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_n", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })]`. `aggregate()` returns `1.5 * 2.0 = 3.0`.
- When: `reverse_effect(bolt, ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), "chip_m")` is called.
- Then: `EffectStack<SpeedBoostConfig>` has one entry: `("chip_n", SpeedBoostConfig { multiplier: OrderedFloat(2.0) })`. `aggregate()` returns `2.0`.
- Edge case: Remove the second entry too. Stack is empty. `aggregate()` returns `1.0` (multiplicative identity for empty stack).
- Edge case: Two entries from the SAME source with the SAME config: `[("chip_o", SpeedBoostConfig { multiplier: OrderedFloat(1.5) }), ("chip_o", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })]`. Reversing once removes only the first matching entry. Stack has one entry remaining. `aggregate()` returns `1.5`.

### 14. **Spawn watcher: register for Bolt, spawn bolt, verify tree stamped**

- Given: `SpawnStampRegistry` resource contains one entry: `("chip_p", EntityKind::Bolt, When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))))`. No bolt entities exist.
- When: A bolt entity is spawned with the `Bolt` component (via `commands.spawn(Bolt)`). The spawn watcher system runs (queries `Added<Bolt>`, checks registry for matching EntityKind).
- Then: The newly spawned bolt entity has `BoundEffects` containing one entry: `BoundEntry { source: "chip_p", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }`. The bolt also has `StagedEffects` (inserted as a pair). The registry entry persists (not consumed — it watches for future spawns too).
- Edge case: Spawn a second bolt. It also receives the same tree stamped. Both bolts have independent copies of the BoundEffects entry.
- Edge case: Registry has `EntityKind::Cell` watcher. Spawning a bolt does NOT trigger it. The bolt has no BoundEffects from this watcher.
- Edge case: Registry has `EntityKind::Any` watcher. Spawning a bolt triggers it. Spawning a cell triggers it. Both receive the tree.

---

## Types

All types referenced in these tests come from the effect-refactor and unified-death-pipeline design docs. No new types are introduced for these integration tests.

### Existing types used (from design docs, implemented in waves 2-7):
- `BoundEffects`, `BoundEntry`, `StagedEffects` — effect tree storage components
- `Tree`, `ScopedTree`, `Terminal`, `ScopedTerminal`, `RootNode` — tree node enums
- `Trigger`, `TriggerContext` — trigger dispatch types
- `EffectType`, `ReversibleEffectType` — effect type enums
- `EffectStack<T>`, `PassiveEffect` — generic stacking component and trait
- `SpeedBoostConfig`, `DamageBoostConfig`, `DieConfig`, `ShockwaveConfig` — effect configs
- `Hp`, `KilledBy`, `Dead` — death pipeline components
- `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity` — death pipeline messages
- `EffectTimers`, `EffectTimerExpired` — timer lifecycle types
- `SpawnStampRegistry` — spawn watcher resource
- `Condition`, `EntityKind`, `ParticipantTarget`, `BumpTarget`, `DeathTarget`, `ImpactTarget` — enum types
- `GameEntity` — marker trait for entity types
- `Fireable`, `Reversible` — effect dispatch traits
- `OrderedFloat<f32>` — from `ordered-float` crate, for f32 equality
- `BumpPerformed`, `BumpGrade` — bump message and grade enum
- `Bolt`, `Breaker`, `Cell`, `Wall` — entity marker components
- `RouteType` — bound vs staged routing

### Test helpers needed:
- A minimal test app builder that registers the effect systems (Bridge, Tick, Conditions), death pipeline systems (ApplyDamage, DetectDeaths), and process_despawn_requests
- Helper to send messages and advance the app by one or more FixedUpdate frames
- Helper to set up BoundEffects with specific entries on entities

---

## Messages

No new messages. All messages used in integration tests are defined in waves 2-7:
- `BumpPerformed` — sent by breaker domain, consumed by bump bridges
- `DamageDealt<Cell>`, `DamageDealt<Bolt>` — death pipeline damage messages
- `KillYourself<Cell>`, `KillYourself<Bolt>` — death pipeline kill commands
- `Destroyed<Cell>`, `Destroyed<Bolt>` — death confirmation messages
- `DespawnEntity` — deferred despawn request
- `EffectTimerExpired` — timer expiry notification

---

## Reference Files

Since this is a clean-room implementation, reference files point to the design docs, not existing source code:

- `docs/todos/detail/effect-refactor/walking-effects/walking-algorithm.md` — walking algorithm (Staged first, then Bound)
- `docs/todos/detail/effect-refactor/walking-effects/when.md` — When arming rules
- `docs/todos/detail/effect-refactor/walking-effects/once.md` — Once consumption
- `docs/todos/detail/effect-refactor/walking-effects/until.md` — Until timer lifecycle
- `docs/todos/detail/effect-refactor/walking-effects/on.md` — On redirection
- `docs/todos/detail/effect-refactor/walking-effects/fire.md` — Fire leaf
- `docs/todos/detail/effect-refactor/walking-effects/during.md` — During condition lifecycle
- `docs/todos/detail/effect-refactor/walking-effects/sequence.md` — Sequence ordering
- `docs/todos/detail/effect-refactor/evaluating-conditions/evaluate-conditions.md` — condition polling
- `docs/todos/detail/effect-refactor/dispatching-triggers/dispatch-algorithm.md` — trigger context table
- `docs/todos/detail/effect-refactor/migration/plugin-wiring/system-set-ordering.md` — effect system ordering
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` — death pipeline ordering
- `docs/todos/detail/effect-refactor/storing-effects/bound-effects.md` — BoundEffects storage
- `docs/todos/detail/effect-refactor/storing-effects/staged-effects.md` — StagedEffects storage
- `docs/todos/detail/effect-refactor/storing-effects/spawn-stamp-registry.md` — SpawnStampRegistry

---

## Scenario Coverage

- New invariants: none — integration correctness is verified by unit tests in waves 4-7 and these integration tests. Existing scenario invariants (`BoltSpeedAccurate`, `BoltInBounds`, etc.) will exercise effect pipeline paths once domain migration is complete in waves 9-12.
- New scenarios: none at this stage — integration tests cover the cross-domain flows. Scenario coverage for the full effect + death pipeline will be evaluated in wave 14 (Full Verification Tier).
- Self-test scenarios: none needed — no new InvariantKind variants introduced.
- Layout updates: none — integration tests use programmatic entity setup, not scenario layouts.

---

## Constraints

### Tests go in:
`src/effect/systems/integration_tests.rs` (or `src/effect/systems/integration_tests/` directory if the test file exceeds 400 lines)

All integration tests are in a single test module within the effect domain, since the effect domain is the integration point for the pipeline.

### Test structure:
Each test function sets up a minimal Bevy App with:
1. `MinimalPlugins` for basic Bevy scheduling
2. The specific systems needed for the test (bridges, walkers, tick systems, condition evaluator, apply_damage, detect_deaths, process_despawn_requests as needed)
3. Entities with the exact components described in each behavior's Given
4. Messages sent via `MessageWriter` or by directly invoking the bridge system
5. App advanced via `app.update()` for one or more frames
6. Assertions on component state (`EffectStack`, `BoundEffects`, `StagedEffects`, `Hp`, `Dead`, entity existence)

### Do NOT test:
- Individual system unit behavior (covered in waves 4-7 unit tests)
- RON deserialization (covered in wave 3)
- Visual/audio effects (not testable in headless mode)
- Specific effect implementations beyond SpeedBoost/DamageBoost/Die (waves 4-7 cover all 30 effects individually)
- Domain-specific kill handler logic beyond confirming the pipeline completes (waves 9-12 cover domain migration)

### Test naming convention:
`test_integration_<feature>` — e.g., `test_integration_bump_fires_speed_boost`, `test_integration_multi_stage_arming`, `test_integration_cascade_delay`

### Frame advancement:
Tests that verify multi-frame behavior (cascade delay, timer lifecycle) must advance the app by specific frame counts and assert intermediate states. Use a helper like `advance_frames(app: &mut App, n: usize)` that calls `app.update()` n times.

### Command flushing:
Many assertions depend on deferred commands being applied. After `app.update()`, commands are flushed. Tests must account for this — fire_effect, stage_effect, remove_effect are all deferred commands that apply after the system that queued them completes.
