# Test Spec: Effect + Death Pipeline — Integration Tests (Wave 8)

## Domain
`src/effect_v3/` (primary), `src/shared/` (death pipeline), cross-domain bridges

## Overview

Integration tests that exercise end-to-end flows across system boundaries: trigger dispatch into tree walking into effect firing, multi-stage arming across frames, timer lifecycles, condition lifecycles, Once consumption, On redirection, the full death pipeline, die bypass, cascade delay, global triggers with On resolution, passive stacking with partial removal, and spawn watchers. Each test sets up a minimal Bevy App with the required systems and verifies observable state changes after running one or more FixedUpdate ticks.

---

## Prerequisites

The following waves must be complete before wave 8 integration tests can be written and run:

- **Wave 2** (scaffold) — all type definitions exist: Tree, ScopedTree, Terminal, ScopedTerminal, Trigger, TriggerContext, Condition, EffectType, ReversibleEffectType, EntityKind, RouteType, ParticipantTarget, BumpTarget, ImpactTarget, DeathTarget, BoltLostTarget, BoundEffects, BoundEntry, StagedEffects, EffectStack, SpawnStampRegistry, EffectTimers, Hp, KilledBy, Dead, DamageDealt, KillYourself, Destroyed, DespawnEntity, all config structs (SpeedBoostConfig, DamageBoostConfig, DieConfig, LoseLifeConfig, etc.), all marker components (Bolt, Breaker, Cell, Wall), GameEntity trait, Fireable/Reversible traits, PassiveEffect trait, NodeState, BumpPerformed, BumpGrade, OrderedFloat
- **Wave 3** (RON assets) — RON deserialization for tree syntax
- **Wave 4** (functions) — EffectStack methods (aggregate, push, remove_by_source), walk_effects, fire_effect/reverse_effect/stage_effect/remove_effect commands, evaluate_conditions helper functions (is_node_active, etc.), PassiveEffect implementations
- **Wave 5** (triggers) — all bridge systems (on_bumped, on_bump_occurred, on_destroyed, on_impacted, on_time_expires, on_node_start_occurred, etc.), tick_effect_timers, track_combo_streak
- **Wave 6** (effects) — all 30 effect fire/reverse implementations (SpeedBoost, DamageBoost, Die, LoseLife, Shockwave, etc.), spawn watcher system, condition evaluation
- **Wave 7** (death pipeline) — apply_damage, detect_cell_deaths, detect_bolt_deaths, detect_wall_deaths, detect_breaker_deaths, process_despawn_requests

---

## Behavior

### 1. **Bump triggers walk BoundEffects and fire effects**

- Given: A bolt entity with `BoundEffects` containing one entry: `BoundEntry { source: "chip_a", tree: When(Bumped, Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }`. The bolt has `EffectStack<SpeedBoostConfig>` initialized empty (default). A breaker entity exists.
- When: `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` message is sent. The `on_bumped` bridge system runs (dispatches `Trigger::Bumped` with `TriggerContext::Bump { bolt: Some(bolt_entity), breaker: breaker_entity }` on the bolt). Then `walk_effects` evaluates the bolt's BoundEffects. The `fire_effect` command is applied.
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

- Given: A bolt entity with `BoundEffects` containing: `BoundEntry { source: "chip_e", tree: During(NodeActive, Box::new(ScopedTree::Fire(ReversibleEffectType::SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.8) })))), condition_active: Some(false) }`. Empty `EffectStack<SpeedBoostConfig>`. The world has `State<NodeState>` set to `NodeState::Loading` (not active).
- When: `State<NodeState>` transitions to `NodeState::Playing`. `evaluate_conditions` runs and detects the transition from false to true (because `is_node_active` now returns true).
- Then: `EffectStack<SpeedBoostConfig>` has one entry `("chip_e", SpeedBoostConfig { multiplier: OrderedFloat(1.8) })`. `condition_active` on the BoundEntry is now `Some(true)`.
- When: `State<NodeState>` transitions from `NodeState::Playing` to `NodeState::Loading`. `evaluate_conditions` detects true-to-false transition.
- Then: `EffectStack<SpeedBoostConfig>` is empty. `condition_active` is `Some(false)`. The BoundEntry is still in BoundEffects (During entries persist).
- When: `State<NodeState>` transitions back to `NodeState::Playing`.
- Then: `EffectStack<SpeedBoostConfig>` has one entry again. Verifies During cycling.
- Edge case: If `State<NodeState>` is already `NodeState::Playing` when the During entry is first installed (condition_active starts as `Some(false)` but `is_node_active` returns true), evaluate_conditions should detect the initial false-to-true transition and fire. This is the "first frame" behavior.

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
- When: Stub kill handler processes `KillYourself<Cell>`. Inserts `Dead` on cell_entity. Sends `Destroyed<Cell> { victim: cell_entity, killer: Some(bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)), _marker: PhantomData }`. Sends `DespawnEntity { entity: cell_entity }`.
- When: `on_destroyed::<Cell>` bridge dispatches `Trigger::Died` on cell_entity (Local), `Trigger::Killed(EntityKind::Cell)` on bolt_entity (Local), `Trigger::DeathOccurred(EntityKind::Cell)` globally.
- When: `process_despawn_requests` runs in PostFixedUpdate. Cell entity is despawned.
- Then: Cell entity no longer exists in the world. Bolt entity still exists. The full chain DamageDealt -> apply_damage -> detect_deaths -> KillYourself -> stub kill handler -> Destroyed -> triggers -> DespawnEntity -> despawn completed in one frame (except the death bridge reads Destroyed next frame per the standard Bevy message persistence pattern).
- Edge case: Cell with `Hp { current: 2.0, ... }` receiving `DamageDealt` with `amount: 1.0`. Hp goes to 1.0. `detect_cell_deaths` does NOT send KillYourself (Hp > 0). Cell survives. Send another `DamageDealt` with `amount: 1.0`. Now Hp reaches 0. Death pipeline fires.

### 10. **Die bypass: Fire(Die) sends KillYourself directly, no Hp change**

- Given: A bolt entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `Bolt` component, `KilledBy { dealer: None }`, no `Dead`. A breaker entity exists. BoundEffects on bolt containing: `BoundEntry { source: "chip_j", tree: When(Bumped, Fire(Die(DieConfig {}))), condition_active: None }`.
- When: `BumpPerformed { grade: BumpGrade::Perfect, bolt: Some(bolt_entity), breaker: breaker_entity }` is sent. `on_bumped` bridge dispatches `Trigger::Bumped` on bolt. Fire evaluates `Die(DieConfig {})`. The `DieConfig::fire` inspects the bolt entity, finds `Bolt` component, sends `KillYourself<Bolt> { victim: bolt_entity, killer: None, _marker: PhantomData }`.
- Then: `KillYourself<Bolt>` is in the message queue. `Hp` on the bolt is still `{ current: 3.0, starting: 3.0, max: None }` — Die does NOT change Hp. The bolt's Hp is irrelevant; it dies from KillYourself regardless. After the stub kill handler processes KillYourself, `Dead` is inserted, `Destroyed<Bolt>` is sent, and `DespawnEntity` follows.
- Edge case: An entity with no `Hp` component. `Fire(Die)` still sends `KillYourself` — Die does not check Hp. The entity enters the death pipeline via KillYourself, bypassing the Hp-based detection entirely.

### 11. **Cascade delay: death-triggered DamageDealt is processed next frame**

  The cascade delay test verifies that when Cell A dies and its death bridge fires a tree that sends `DamageDealt<Cell>` targeting Cell B, that damage is NOT processed in the same frame as Cell A's death. The mechanism:
  - Frame 1: Game systems send DamageDealt for Cell A. ApplyDamage decrements A's Hp. DetectDeaths sends KillYourself for A. Stub kill handler inserts Dead, sends Destroyed for A.
  - Frame 2: `on_destroyed::<Cell>` bridge reads `Destroyed<Cell>` (persisted from frame 1). Dispatches `Trigger::Died` on Cell A. Cell A's tree fires `Die(DieConfig {})` targeting Cell B via `On(Death(Killer), ...)` redirection — but for this test we use a simpler mechanism: the test directly injects `DamageDealt<Cell> { target: cell_b, amount: 1.0 }` in frame 2 to simulate cascade damage. Since Bridge runs before ApplyDamage, this DamageDealt is processed by `apply_damage::<Cell>` in the same frame 2.
  - The key assertion: Cell B is NOT damaged in frame 1 (the frame Cell A took its fatal hit). Cell B IS damaged in frame 2.

- Given: Cell A at position (100.0, 200.0) with `Hp { current: 1.0, starting: 1.0, max: None }`, `Cell`, `KilledBy { dealer: None }`, no `Dead`. Cell A has no BoundEffects (cascade is simulated by the test injecting DamageDealt directly in frame 2). Cell B at position (120.0, 200.0) with `Hp { current: 1.0, starting: 1.0, max: None }`, `Cell`, `KilledBy { dealer: None }`, no `Dead`. A bolt entity exists as the initial killer.
- When: Frame 1: `DamageDealt<Cell> { target: cell_a, amount: 1.0, dealer: Some(bolt_entity), source_chip: None, _marker: PhantomData }` is sent. `apply_damage::<Cell>` processes it. Cell A Hp goes to 0.0. `detect_cell_deaths` sends `KillYourself<Cell>` for Cell A. Stub kill handler inserts `Dead` on Cell A, sends `Destroyed<Cell> { victim: cell_a, killer: Some(bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)), _marker: PhantomData }`, sends `DespawnEntity { entity: cell_a }`.
- Then: After frame 1: Cell A has `Dead` marker. Cell B Hp is still 1.0 — no damage processed on B this frame. The `Destroyed<Cell>` for Cell A is in the message queue.
- When: Frame 2: The test injects `DamageDealt<Cell> { target: cell_b, amount: 1.0, dealer: None, source_chip: None, _marker: PhantomData }` to simulate cascade damage (in a real game, a death-triggered effect would produce this). `apply_damage::<Cell>` processes it. Cell B Hp goes to 0.0.
- Then: After frame 2: Cell B Hp is 0.0. `detect_cell_deaths` sends `KillYourself<Cell>` for Cell B. Cell B enters the death pipeline.
- Edge case: Verify the one-frame delay from initial damage to cascade damage — Cell B must NOT have `Dead` or reduced Hp after frame 1, only after frame 2.

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

### 15. **Killed(EntityKind) trigger: killer entity's tree fires on kill**

- Given: A bolt entity with `Bolt` component, `BoundEffects` containing: `BoundEntry { source: "chip_q", tree: When(Killed(Cell), Fire(SpeedBoost(SpeedBoostConfig { multiplier: OrderedFloat(1.5) }))), condition_active: None }`, `StagedEffects` (default), empty `EffectStack<SpeedBoostConfig>`. A cell entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `Cell`, `KilledBy { dealer: None }`, no `Dead`. A breaker entity exists.
- When: `DamageDealt<Cell> { dealer: Some(bolt_entity), target: cell_entity, amount: 1.0, source_chip: None, _marker: PhantomData }` is sent. `apply_damage::<Cell>` decrements cell Hp to 0.0. `detect_cell_deaths` sends `KillYourself<Cell>`. Stub kill handler inserts `Dead`, sends `Destroyed<Cell> { victim: cell_entity, killer: Some(bolt_entity), victim_pos: Vec2::new(100.0, 200.0), killer_pos: Some(Vec2::new(50.0, 300.0)), _marker: PhantomData }`, sends `DespawnEntity { entity: cell_entity }`.
- When: Next frame: `on_destroyed::<Cell>` bridge reads `Destroyed<Cell>`. Dispatches `Trigger::Killed(EntityKind::Cell)` on bolt_entity (Local, on killer) with `TriggerContext::Death { victim: cell_entity, killer: Some(bolt_entity) }`.
- Then: `EffectStack<SpeedBoostConfig>` on the bolt has one entry `("chip_q", SpeedBoostConfig { multiplier: OrderedFloat(1.5) })`. The bolt's BoundEffects entry persists (When re-arms).
- Edge case: Bolt has `When(Killed(Bolt), Fire(SpeedBoost(...)))` — killing a cell does NOT trigger this tree because `Killed(Bolt)` does not match `Killed(Cell)`. `EffectStack<SpeedBoostConfig>` remains empty.
- Edge case: Cell death with no killer (`killer: None`, environmental death). `Killed(Cell)` is NOT dispatched on any entity (no killer exists). The bolt's tree does not fire.

### 16. **RON deserialization smoke test: all breaker definitions deserialize**
- Given: All `.breaker.ron` files in `breaker-game/assets/breakers/`
- When: For each file, read the contents and deserialize as `BreakerDefinition` using `ron::from_str`
- Then: Every file deserializes successfully. If any file fails, the test panics with the filename and the deserialization error.
- Edge case: An empty directory produces zero assertions (test passes vacuously — no files to check).
- Note: The test dynamically discovers files at runtime using `std::fs::read_dir`. Adding a new `.breaker.ron` file automatically includes it in the test. Do NOT hardcode filenames.

### 17. **RON deserialization smoke test: all bolt definitions deserialize**
- Given: All `.bolt.ron` files in `breaker-game/assets/bolts/`
- When: For each file, read the contents and deserialize as `BoltDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

### 18. **RON deserialization smoke test: all cell definitions deserialize**
- Given: All `.cell.ron` files in `breaker-game/assets/cells/`
- When: For each file, read the contents and deserialize as `CellDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

### 19. **RON deserialization smoke test: all chip definitions deserialize**
- Given: All `.chip.ron` files in `breaker-game/assets/chips/standard/`
- When: For each file, read the contents and deserialize as `ChipDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

### 20. **RON deserialization smoke test: all evolution definitions deserialize**
- Given: All `.evolution.ron` files in `breaker-game/assets/chips/evolutions/`
- When: For each file, read the contents and deserialize as `EvolutionDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

### 21. **RON deserialization smoke test: all node definitions deserialize**
- Given: All `.node.ron` files in `breaker-game/assets/nodes/`
- When: For each file, read the contents and deserialize as `NodeDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

### 22. **RON deserialization smoke test: all wall definitions deserialize**
- Given: All `.wall.ron` files in `breaker-game/assets/walls/`
- When: For each file, read the contents and deserialize as `WallDefinition` using `ron::from_str`
- Then: Every file deserializes successfully with filename in the error message on failure.
- Note: Dynamic file discovery via `std::fs::read_dir`.

---

## Types

All types referenced in these tests come from the effect-refactor and unified-death-pipeline design docs. No new types are introduced for these integration tests.

### Existing types used (from design docs, implemented in waves 2-7):
- `BoundEffects`, `BoundEntry`, `StagedEffects` — effect tree storage components
- `Tree`, `ScopedTree`, `Terminal`, `ScopedTerminal`, `RootNode` — tree node enums
- `Trigger`, `TriggerContext` — trigger dispatch types
- `EffectType`, `ReversibleEffectType` — effect type enums
- `EffectStack<T>`, `PassiveEffect` — generic stacking component and trait
- `SpeedBoostConfig`, `DamageBoostConfig`, `DieConfig`, `LoseLifeConfig`, `ShockwaveConfig` — effect configs
- `Hp`, `KilledBy`, `Dead` — death pipeline components
- `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `DespawnEntity` — death pipeline messages
- `EffectTimers`, `EffectTimerExpired` — timer lifecycle types
- `SpawnStampRegistry` — spawn watcher resource
- `Condition`, `EntityKind`, `ParticipantTarget`, `BumpTarget`, `DeathTarget`, `ImpactTarget` — enum types
- `GameEntity` — marker trait for entity types
- `Fireable`, `Reversible` — effect dispatch traits
- `OrderedFloat<f32>` — from `ordered-float` crate, for f32 equality
- `BumpPerformed { grade: BumpGrade, bolt: Option<Entity>, breaker: Entity }`, `BumpGrade` — bump message and grade enum
- `Bolt`, `Breaker`, `Cell`, `Wall` — entity marker components
- `RouteType` — bound vs staged routing

### Test helpers needed:
- A minimal test app builder that registers the effect systems (Bridge, Tick, Conditions), death pipeline systems (ApplyDamage, DetectDeaths), process_despawn_requests, and stub kill handlers for Cell, Bolt, Wall, Breaker
- Helper to send messages and advance the app by one or more FixedUpdate frames
- Helper to set up BoundEffects with specific entries on entities
- `stub_kill_handler::<T>` system — reads `KillYourself<T>`, inserts `Dead`, sends `Destroyed<T>` and `DespawnEntity`

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
- `docs/todos/detail/effect-refactor/dispatching-triggers/death/killed.md` — Killed(EntityKind) trigger
- `docs/todos/detail/effect-refactor/dispatching-triggers/death/died.md` — Died trigger
- `docs/todos/detail/effect-refactor/migration/new-effect-implementations/die.md` — Die effect fire behavior
- `docs/todos/detail/effect-refactor/migration/new-effect-implementations/lose-life.md` — LoseLife effect fire behavior

---

## Scenario Coverage

- New invariants: none — integration correctness is verified by unit tests in waves 4-7 and these integration tests. Existing scenario invariants (`BoltSpeedAccurate`, `BoltInBounds`, etc.) will exercise effect pipeline paths once domain migration is complete in waves 9-12.
- New scenarios: none at this stage — integration tests cover the cross-domain flows. Scenario coverage for the full effect + death pipeline will be evaluated in wave 14 (Full Verification Tier).
- Self-test scenarios: none needed — no new InvariantKind variants introduced.
- Layout updates: none — integration tests use programmatic entity setup, not scenario layouts.

---

## Constraints

### Tests go in:
`src/effect_v3/systems/integration_tests.rs` (or `src/effect_v3/systems/integration_tests/` directory if the test file exceeds 400 lines)

All integration tests are in a single test module within the effect domain, since the effect domain is the integration point for the pipeline.

### Test structure:
Each test function sets up a minimal Bevy App with:
1. `MinimalPlugins` for basic Bevy scheduling
2. The specific systems needed for the test (bridges, walkers, tick systems, condition evaluator, apply_damage, detect_deaths, process_despawn_requests as needed)
3. Entities with the exact components described in each behavior's Given
4. Messages sent via `MessageWriter` or by directly invoking the bridge system
5. App advanced via `app.update()` for one or more frames
6. Assertions on component state (`EffectStack`, `BoundEffects`, `StagedEffects`, `Hp`, `Dead`, entity existence)

### Stub kill handlers for death pipeline tests:
Behaviors 9, 10, 11, 12, and 15 exercise the full death pipeline, which requires domain kill handlers (implemented in waves 9-12). Since wave 8 runs before those waves, integration tests must register **stub kill handlers** in the test app. Each stub kill handler:
1. Reads `KillYourself<T>` messages (one per entity type: `KillYourself<Cell>`, `KillYourself<Bolt>`, etc.)
2. For each message: inserts `Dead` on `msg.victim`
3. Sends `Destroyed<T> { victim: msg.victim, killer: msg.killer, victim_pos: <query Position2D or use hardcoded Vec2>, killer_pos: <query killer Position2D or None>, _marker: PhantomData }`
4. Sends `DespawnEntity { entity: msg.victim }`

This is the minimal behavior needed to complete the pipeline. The stubs skip all domain-specific logic (invulnerability checks, shield walls, VFX, audio, stats tracking). A single generic stub function `stub_kill_handler::<T: GameEntity + Component>` can serve all entity types.

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
