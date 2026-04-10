## Test Spec: Bolt Domain — Unified Death Pipeline Migration

### Domain
src/bolt/

### Overview
Migrate the bolt domain from the ad-hoc `RequestBoltDestroyed` / `cleanup_destroyed_bolts` death path to the unified death pipeline. After migration, bolt death flows through: `KillYourself<Bolt>` -> bolt kill handler (inserts `Dead`, removes from spatial index, sends `Destroyed<Bolt>` + `DespawnEntity`) -> `process_despawn_requests` (despawns entity). Two producers send `KillYourself<Bolt>`: `tick_bolt_lifespan` (timer expiry) and `bolt_lost` (off-screen). A third producer, `detect_bolt_deaths`, handles Hp-based death for future damage-to-bolt mechanics.

---

### Section A: tick_bolt_lifespan Migration

#### Behavior

1. **tick_bolt_lifespan sends KillYourself\<Bolt\> when timer expires**
   - Given: A bolt entity with `BoltLifespan` timer at 0.48 seconds remaining (total duration 0.5), delta time 0.02 seconds per tick, no `Birthing` component, no `Dead` component
   - When: `tick_bolt_lifespan` runs once (timer becomes 0.50/0.50, `just_finished()` returns true)
   - Then: Exactly one `KillYourself::<Bolt>` message is sent with `victim` equal to the bolt entity and `killer` equal to `None`
   - Edge case: Timer at exactly 0.0 remaining before tick -- should have already fired on the previous frame's `just_finished()`; no duplicate message this frame

2. **tick_bolt_lifespan does not send KillYourself when timer has not expired**
   - Given: A bolt entity with `BoltLifespan` timer at 0.2 seconds remaining (total duration 0.5), delta time 0.02 seconds
   - When: `tick_bolt_lifespan` runs once (timer becomes 0.22/0.50)
   - Then: No `KillYourself::<Bolt>` message is sent
   - Edge case: Timer at 0.499 remaining -- one tick away from finishing, still no message

3. **tick_bolt_lifespan skips bolts with Birthing component**
   - Given: A bolt entity with `BoltLifespan` timer at 0.49 seconds remaining (total duration 0.5), delta time 0.02 seconds, AND a `Birthing` component
   - When: `tick_bolt_lifespan` runs once
   - Then: No `KillYourself::<Bolt>` message is sent. The timer should still tick (reaching 0.51/0.50), but the system does not send the kill message while `Birthing` is present.
   - Edge case: Bolt has `Birthing` AND timer `just_finished()` -- message suppressed until `Birthing` is removed

4. **tick_bolt_lifespan does NOT insert Dead component**
   - Given: A bolt entity with `BoltLifespan` timer that will expire this frame
   - When: `tick_bolt_lifespan` runs and sends `KillYourself::<Bolt>`
   - Then: The bolt entity does NOT have a `Dead` component inserted. Only the kill handler inserts `Dead`.
   - Edge case: N/A -- this is a negative assertion confirming separation of concerns

5. **tick_bolt_lifespan does NOT despawn the bolt**
   - Given: A bolt entity with `BoltLifespan` timer that will expire this frame
   - When: `tick_bolt_lifespan` runs and sends `KillYourself::<Bolt>`
   - Then: The bolt entity still exists in the world. `process_despawn_requests` handles despawn.
   - Edge case: N/A -- negative assertion

6. **tick_bolt_lifespan does NOT send RequestBoltDestroyed (removed message type)**
   - Given: A bolt entity with `BoltLifespan` timer that will expire this frame
   - When: `tick_bolt_lifespan` runs
   - Then: No `RequestBoltDestroyed` message is sent (this message type no longer exists)
   - Edge case: N/A -- compile-time guarantee if the type is removed; test exists to verify behavioral contract

---

### Section B: bolt_lost Migration

#### Behavior

7. **bolt_lost sends KillYourself\<Bolt\> when bolt leaves play area**
   - Given: A bolt entity at position (100.0, -400.0) which is below the breaker/playfield bottom, a breaker entity at position (100.0, -300.0)
   - When: `bolt_lost` detects the bolt is out of bounds
   - Then: Exactly one `KillYourself::<Bolt>` message is sent with `victim` equal to the bolt entity and `killer` equal to `None`
   - Edge case: Bolt at exactly the boundary position (e.g., y == playfield_bottom) -- depends on the boundary check being strictly less-than or less-than-or-equal; test the boundary

8. **bolt_lost sends BoltLost message with bolt and breaker fields populated**
   - Given: A bolt entity (entity A) at position (100.0, -400.0), a breaker entity (entity B) at position (100.0, -300.0)
   - When: `bolt_lost` detects the bolt is out of bounds
   - Then: A `BoltLost { bolt: entity_A, breaker: entity_B }` message is sent with both fields correctly populated
   - Edge case: Multiple bolts lost in the same frame -- each generates its own `BoltLost` message with the correct bolt entity

9. **bolt_lost sends both KillYourself and BoltLost for same bolt**
   - Given: A bolt entity at position (100.0, -400.0) that is below the playfield
   - When: `bolt_lost` runs
   - Then: Both a `KillYourself::<Bolt>` AND a `BoltLost` message are sent for the same bolt entity. The `KillYourself` feeds the death pipeline; the `BoltLost` feeds the effect system's `on_bolt_lost_occurred` bridge.
   - Edge case: N/A -- both messages always fire together for the same event

10. **bolt_lost does NOT insert Dead component**
    - Given: A bolt entity that will be detected as lost
    - When: `bolt_lost` runs
    - Then: The bolt entity does NOT have a `Dead` component. The kill handler inserts `Dead`.
    - Edge case: N/A -- negative assertion

11. **bolt_lost does NOT despawn the bolt**
    - Given: A bolt entity that will be detected as lost
    - When: `bolt_lost` runs
    - Then: The bolt entity still exists in the world
    - Edge case: N/A -- negative assertion

12. **bolt_lost does NOT send RequestBoltDestroyed (removed message type)**
    - Given: A bolt entity that will be detected as lost
    - When: `bolt_lost` runs
    - Then: No `RequestBoltDestroyed` message is sent (type removed)
    - Edge case: N/A -- compile-time guarantee

---

### Section C: BoltLost Message Field Migration

#### Behavior

13. **BoltLost is a struct with bolt and breaker fields, not a unit struct**
    - Given: N/A -- type definition test
    - When: `BoltLost { bolt: Entity::PLACEHOLDER, breaker: Entity::PLACEHOLDER }` is constructed
    - Then: Compiles. The struct has two named `Entity` fields: `bolt` and `breaker`.
    - Edge case: Attempting to construct `BoltLost` as a unit struct (no fields) must NOT compile -- enforced by the type system, not a runtime test

---

### Section D: Bolt Kill Handler

#### Behavior

14. **Bolt kill handler inserts Dead on victim when processing KillYourself\<Bolt\>**
    - Given: A bolt entity (entity A) with `Bolt` component, position at (50.0, 200.0), no `Dead` component
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed by the bolt kill handler
    - Then: Entity A has a `Dead` component inserted
    - Edge case: `KillYourself` with a victim entity that does not have the `Bolt` component -- the kill handler should skip it (query uses `With<Bolt>` filter)

15. **Bolt kill handler removes bolt from spatial index**
    - Given: A bolt entity (entity A) inserted into the spatial index at position (50.0, 200.0)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: Entity A is no longer present in the spatial index query results
    - Edge case: Bolt not in the spatial index (already removed by another system) -- handler should not panic

16. **Bolt kill handler sends Destroyed\<Bolt\> with correct fields**
    - Given: A bolt entity (entity A) at position (75.0, 150.0), killer is `None` (environmental death)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: A `Destroyed::<Bolt>` message is sent with `victim: entity_A`, `killer: None`, `victim_pos: Vec2::new(75.0, 150.0)`, `killer_pos: None`
    - Edge case: Bolt at position (0.0, 0.0) -- origin position still produces a valid Destroyed message

17. **Bolt kill handler sends Destroyed\<Bolt\> with killer position when killer exists**
    - Given: A bolt entity (entity A) at position (75.0, 150.0), a killer entity (entity B) at position (200.0, 300.0)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: Some(entity_B) }` message is processed
    - Then: A `Destroyed::<Bolt>` message is sent with `victim: entity_A`, `killer: Some(entity_B)`, `victim_pos: Vec2::new(75.0, 150.0)`, `killer_pos: Some(Vec2::new(200.0, 300.0))`
    - Edge case: Killer entity no longer exists in world at time of handler execution -- `killer_pos` should be `None` (entity despawned between KillYourself send and handler execution), but `killer` field still carries `Some(entity_B)` from the message

18. **Bolt kill handler sends DespawnEntity for the victim**
    - Given: A bolt entity (entity A)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: A `DespawnEntity { entity: entity_A }` message is sent
    - Edge case: N/A

19. **Bolt kill handler skips entities already marked Dead (double-kill prevention)**
    - Given: A bolt entity (entity A) with `Bolt` component AND `Dead` component already inserted
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: No `Destroyed::<Bolt>` message is sent. No `DespawnEntity` message is sent. No additional `Dead` component is inserted.
    - Edge case: Two `KillYourself::<Bolt>` messages for the same entity in the same frame -- only the first is processed; the second sees `Dead` and skips

20. **Bolt kill handler does NOT despawn the entity directly**
    - Given: A bolt entity (entity A) that receives `KillYourself::<Bolt>`
    - When: The kill handler runs
    - Then: Entity A still exists in the world (has `Dead` but is not despawned). Despawn is deferred to `process_despawn_requests` in PostFixedUpdate.
    - Edge case: N/A -- negative assertion

---

### Section E: detect_bolt_deaths (Hp-Based Death Path)

#### Behavior

21. **detect_bolt_deaths sends KillYourself\<Bolt\> when Hp is zero or below**
    - Given: A bolt entity (entity A) with `Bolt` component, `Hp { current: 0.0, starting: 1.0, max: None }`, `KilledBy { dealer: Some(entity_B) }`, no `Dead` component
    - When: `detect_bolt_deaths` runs
    - Then: Exactly one `KillYourself::<Bolt>` message is sent with `victim: entity_A`, `killer: Some(entity_B)`
    - Edge case: `Hp { current: -5.0, ... }` (negative HP from overkill damage) -- still sends KillYourself

22. **detect_bolt_deaths does not send KillYourself when Hp is positive**
    - Given: A bolt entity with `Bolt` component, `Hp { current: 0.5, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, no `Dead` component
    - When: `detect_bolt_deaths` runs
    - Then: No `KillYourself::<Bolt>` message is sent
    - Edge case: `Hp { current: 0.001, ... }` (barely alive) -- no message

23. **detect_bolt_deaths skips bolts with Dead component**
    - Given: A bolt entity with `Bolt` component, `Hp { current: 0.0, ... }`, `KilledBy { dealer: None }`, AND a `Dead` component
    - When: `detect_bolt_deaths` runs
    - Then: No `KillYourself::<Bolt>` message is sent (already dead, skip via `Without<Dead>`)
    - Edge case: Entity has `Dead` but Hp is positive -- still skipped (Dead takes priority)

24. **detect_bolt_deaths does NOT insert Dead**
    - Given: A bolt entity with Hp <= 0
    - When: `detect_bolt_deaths` runs and sends `KillYourself::<Bolt>`
    - Then: The bolt entity does NOT have `Dead` inserted. Only the kill handler inserts `Dead`.
    - Edge case: N/A -- negative assertion

25. **detect_bolt_deaths reads killer from KilledBy component**
    - Given: A bolt entity with `Hp { current: 0.0, ... }`, `KilledBy { dealer: Some(entity_C) }`
    - When: `detect_bolt_deaths` runs
    - Then: `KillYourself::<Bolt>` is sent with `killer: Some(entity_C)` -- propagating the kill attribution from `apply_damage`
    - Edge case: `KilledBy { dealer: None }` (environmental death via Hp path, unlikely but possible) -- `killer: None` in the message

---

### Section F: apply_damage::\<Bolt\> (Shared Generic System)

#### Behavior

26. **apply_damage\<Bolt\> decrements Hp by damage amount**
    - Given: A bolt entity with `Bolt` component, `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead` component
    - When: A `DamageDealt::<Bolt> { dealer: Some(entity_X), target: bolt_entity, amount: 1.0, source_chip: None }` message is processed by `apply_damage::<Bolt>`
    - Then: The bolt entity's `Hp.current` is `2.0`
    - Edge case: Damage amount `0.0` -- Hp unchanged (2.0 after 1.0 damage stays, no second damage applied)

27. **apply_damage\<Bolt\> sets KilledBy on killing blow**
    - Given: A bolt entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`
    - When: A `DamageDealt::<Bolt> { dealer: Some(entity_X), target: bolt_entity, amount: 1.0, source_chip: None }` is processed
    - Then: `Hp.current` is `0.0` AND `KilledBy.dealer` is `Some(entity_X)`
    - Edge case: Overkill damage (amount: 5.0 on Hp 1.0) -- `Hp.current` is `-4.0`, `KilledBy.dealer` is `Some(entity_X)`

28. **apply_damage\<Bolt\> does not overwrite existing KilledBy (first kill wins)**
    - Given: A bolt entity with `Hp { current: 2.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`
    - When: Two `DamageDealt::<Bolt>` messages arrive in the same frame -- first from entity_X (amount 2.0, kills), second from entity_Y (amount 1.0, overkill)
    - Then: `KilledBy.dealer` is `Some(entity_X)` (first kill wins, entity_Y does not overwrite)
    - Edge case: Both messages deal exactly lethal damage -- first processed wins

29. **apply_damage\<Bolt\> skips entities with Dead component**
    - Given: A bolt entity with `Bolt`, `Hp { current: 1.0, ... }`, AND `Dead` component
    - When: A `DamageDealt::<Bolt>` message targeting this entity is processed
    - Then: `Hp.current` remains `1.0` -- damage is not applied to dead entities
    - Edge case: N/A

30. **apply_damage\<Bolt\> does not set KilledBy on non-killing blow**
    - Given: A bolt entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`
    - When: A `DamageDealt::<Bolt> { dealer: Some(entity_X), target: bolt_entity, amount: 1.0, ... }` is processed
    - Then: `Hp.current` is `2.0` AND `KilledBy.dealer` is `None` (still unset -- this was not a killing blow)
    - Edge case: Damage that brings Hp to exactly 0.0 IS a killing blow (0.0 <= 0)

---

### Section G: Bolt Builder Hp and KilledBy

#### Behavior

31. **Bolt builder includes Hp component on spawned bolts**
    - Given: A bolt is spawned through the bolt builder (standard spawn path)
    - When: The bolt entity is inspected after spawn
    - Then: The entity has an `Hp` component with `current: 1.0`, `starting: 1.0`, `max: None` (bolts have 1 HP by default -- they die from any damage event, though most bolt deaths are environmental and bypass Hp)
    - Edge case: Bolt spawned with `Birthing` component -- still has `Hp` and `KilledBy` from spawn

32. **Bolt builder includes KilledBy component defaulting to None**
    - Given: A bolt is spawned through the bolt builder
    - When: The bolt entity is inspected after spawn
    - Then: The entity has a `KilledBy { dealer: None }` component
    - Edge case: N/A -- always defaults to None on fresh spawn

---

### Section H: End-to-End Integration

#### Behavior

33. **Lifespan expiry e2e: tick_bolt_lifespan through full death pipeline to despawn**
    - Given: A bolt entity with `BoltLifespan` timer at 0.49 seconds remaining (total duration 0.5), `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, registered in spatial index, at position (100.0, 200.0), delta time 0.02 seconds
    - When: The full FixedUpdate + PostFixedUpdate pipeline runs (tick_bolt_lifespan -> bolt kill handler -> process_despawn_requests)
    - Then:
      1. `tick_bolt_lifespan` sends `KillYourself::<Bolt> { victim: bolt_entity, killer: None }`
      2. Bolt kill handler: inserts `Dead` on bolt entity, removes from spatial index, sends `Destroyed::<Bolt> { victim: bolt_entity, killer: None, victim_pos: Vec2::new(100.0, 200.0), killer_pos: None }`, sends `DespawnEntity { entity: bolt_entity }`
      3. `process_despawn_requests`: despawns the bolt entity
      4. After full pipeline: bolt entity no longer exists in the world
    - Edge case: Bolt also has `Birthing` -- `tick_bolt_lifespan` should NOT fire, so the bolt survives

34. **Bolt lost e2e: bolt_lost through full death pipeline to despawn**
    - Given: A bolt entity at position (100.0, -400.0) below the playfield, a breaker entity at position (100.0, -300.0), bolt registered in spatial index, `Hp { current: 1.0, ... }`, `KilledBy { dealer: None }`
    - When: The full FixedUpdate + PostFixedUpdate pipeline runs (bolt_lost -> bolt kill handler -> process_despawn_requests)
    - Then:
      1. `bolt_lost` sends `KillYourself::<Bolt> { victim: bolt_entity, killer: None }` AND `BoltLost { bolt: bolt_entity, breaker: breaker_entity }`
      2. Bolt kill handler: inserts `Dead`, removes from spatial index, sends `Destroyed::<Bolt> { victim: bolt_entity, killer: None, victim_pos: Vec2::new(100.0, -400.0), killer_pos: None }`, sends `DespawnEntity { entity: bolt_entity }`
      3. `process_despawn_requests`: despawns the bolt entity
      4. After full pipeline: bolt entity no longer exists
    - Edge case: Multiple bolts lost in the same frame -- each independently goes through the full pipeline

35. **Dead prevents double-processing across pipeline stages**
    - Given: A bolt entity that receives `KillYourself::<Bolt>` from BOTH `tick_bolt_lifespan` and `bolt_lost` in the same frame (unlikely but theoretically possible if lifespan expires the same frame the bolt goes out of bounds)
    - When: The bolt kill handler processes both messages
    - Then: Only the FIRST `KillYourself` is processed (inserts `Dead`, sends `Destroyed`, sends `DespawnEntity`). The SECOND is skipped because the entity now has `Dead`. Only one `Destroyed::<Bolt>` and one `DespawnEntity` message are sent.
    - Edge case: Three KillYourself messages for same entity in one frame (two environmental + one Hp-based) -- only first processed

36. **Hp-based death e2e: DamageDealt through detect to kill handler to despawn**
    - Given: A bolt entity with `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, at position (50.0, 100.0), a damage-dealing entity (entity_X) at position (60.0, 110.0)
    - When: A `DamageDealt::<Bolt> { dealer: Some(entity_X), target: bolt_entity, amount: 1.0, source_chip: None }` flows through `apply_damage::<Bolt>` -> `detect_bolt_deaths` -> bolt kill handler -> `process_despawn_requests`
    - Then:
      1. `apply_damage`: Hp.current becomes 0.0, KilledBy.dealer becomes Some(entity_X)
      2. `detect_bolt_deaths`: sends `KillYourself::<Bolt> { victim: bolt_entity, killer: Some(entity_X) }`
      3. Bolt kill handler: inserts Dead, removes from spatial index, sends `Destroyed::<Bolt> { victim: bolt_entity, killer: Some(entity_X), victim_pos: Vec2::new(50.0, 100.0), killer_pos: Some(Vec2::new(60.0, 110.0)) }`, sends `DespawnEntity { entity: bolt_entity }`
      4. `process_despawn_requests`: despawns bolt entity
    - Edge case: entity_X is despawned before the kill handler reads its position -- `killer_pos` is `None`, but `killer` field is still `Some(entity_X)`

---

### Section I: RequestBoltDestroyed Removal

#### Behavior

37. **RequestBoltDestroyed message type no longer exists**
    - Given: N/A -- type system test
    - When: Code attempts to reference `RequestBoltDestroyed`
    - Then: Compilation fails -- the type has been removed entirely
    - Note: This is enforced by the removal itself. Tests in this wave should NOT reference `RequestBoltDestroyed`. The absence of the type is the test -- no system sends it, no system reads it.

38. **cleanup_destroyed_bolts system no longer exists**
    - Given: N/A -- structural assertion
    - When: The bolt plugin is built
    - Then: No system named `cleanup_destroyed_bolts` is registered. The domain kill handler replaces its functionality.
    - Note: This is verified by the kill handler e2e tests working without `cleanup_destroyed_bolts`. The system's file should be deleted.

---

### Types

#### New Types in This Wave
- `BoltLost { bolt: Entity, breaker: Entity }` -- message, replaces the old unit struct `BoltLost`. Derives: `Message, Clone, Debug`. Sent by `bolt_lost`, consumed by `on_bolt_lost_occurred` bridge and `spawn_bolt_lost_text`.

#### Shared Types (prerequisites, must exist before this wave)
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` -- message. Derives: `Message, Clone, Debug`.
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` -- message. Derives: `Message, Clone, Debug`.
- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` -- message. Derives: `Message, Clone, Debug`.
- `DespawnEntity { entity: Entity }` -- message. Derives: `Message, Clone, Debug`.
- `Hp { current: f32, starting: f32, max: Option<f32> }` -- component. Derives: `Component, Debug, Clone`.
- `KilledBy { dealer: Option<Entity> }` -- component. Derives: `Component, Default, Debug`.
- `Dead` -- marker component. Derives: `Component`.
- `GameEntity` -- trait bound: `Component`. Impl'd on `Bolt`, `Cell`, `Wall`, `Breaker`.
- `DeathPipelineSystems` -- system set enum with `ApplyDamage` and `DetectDeaths` variants.
- `DeathDetectionData` -- `#[derive(QueryData)]` with `entity: Entity`, `killed_by: &'static KilledBy`, `hp: &'static Hp`.

#### Existing Types Used
- `Bolt` -- entity marker component
- `BoltLifespan` -- timer component for bolt lifespan
- `Birthing` -- marker component suppressing bolt processing during spawn animation
- `Breaker` -- entity marker component (for BoltLost breaker field)

---

### Messages

- `KillYourself::<Bolt> { victim: Entity, killer: Option<Entity> }` -- sent by `tick_bolt_lifespan`, `bolt_lost`, `detect_bolt_deaths`; consumed by bolt kill handler
- `Destroyed::<Bolt> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }` -- sent by bolt kill handler; consumed by `on_destroyed::<Bolt>` bridge (effect system)
- `DespawnEntity { entity: Entity }` -- sent by bolt kill handler; consumed by `process_despawn_requests`
- `BoltLost { bolt: Entity, breaker: Entity }` -- sent by `bolt_lost`; consumed by `on_bolt_lost_occurred` bridge, `spawn_bolt_lost_text`
- `DamageDealt::<Bolt> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String> }` -- sent by future bolt-damaging mechanics; consumed by `apply_damage::<Bolt>`

---

### Reference Files

These are the design docs that define the behavior being tested. Writer-tests should read these for precise behavioral contracts:
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-modify/tick-bolt-lifespan.md` -- tick_bolt_lifespan migration spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-modify/bolt-lost.md` -- bolt_lost migration spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-remove.md` -- cleanup_destroyed_bolts removal
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/apply-damage-bolt.md` -- apply_damage::<Bolt> spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/detect-bolt-deaths.md` -- detect_bolt_deaths spec
- `docs/todos/detail/unified-death-pipeline/rust-types/` -- all death pipeline type definitions
- `docs/todos/detail/effect-refactor/migration/new-trigger-implementations/bolt-lost/types.md` -- BoltLost field migration
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-sets.md` -- DeathPipelineSystems ordering

---

### Scenario Coverage

- New invariants: none -- existing `BoltInBounds`, `BoltCountReasonable`, `BoltSpeedAccurate`, `NoEntityLeaks` invariants cover bolt lifecycle. The death pipeline is an implementation change, not a new observable property. `NoEntityLeaks` specifically catches bolts that fail to despawn.
- New scenarios: none -- existing chaos and stress scenarios exercise bolt lifespan expiry and bolt loss. The migration should be transparent to scenarios.
- Self-test scenarios: none -- no new InvariantKind added.
- Layout updates: none -- bolt entities are spawned by the system, not defined in layouts.

---

### Constraints

- Tests for `tick_bolt_lifespan` migration go in: `src/bolt/systems/tick_bolt_lifespan/` (existing system, tests alongside or in tests submodule)
- Tests for `bolt_lost` migration go in: `src/bolt/systems/bolt_lost/` (existing system directory)
- Tests for bolt kill handler go in: `src/bolt/systems/handle_bolt_kill/` (new system file/directory)
- Tests for `detect_bolt_deaths` go in: `src/bolt/systems/detect_bolt_deaths/` (new system file)
- Tests for `apply_damage::<Bolt>` go in: `src/shared/systems/apply_damage/` (shared generic system -- may already have tests for Cell monomorphization; add Bolt-specific tests)
- Tests for bolt builder Hp/KilledBy go in: alongside existing bolt builder tests
- Tests for e2e integration go in: `src/bolt/systems/handle_bolt_kill/` (co-located with the kill handler which orchestrates the pipeline)

- Do NOT test: `on_destroyed::<Bolt>` bridge (effect domain, separate wave), `on_bolt_lost_occurred` bridge (effect domain, separate wave), `process_despawn_requests` (shared system, tested in its own wave), visual/fx responses to bolt death, bolt respawn logic (unchanged, tested separately), breaker domain kill handler, cell domain kill handler
- Do NOT reference existing source files in `src/` -- this is a clean-room implementation
- Do NOT import or reference `RequestBoltDestroyed` -- this type is removed
- Do NOT import or reference `cleanup_destroyed_bolts` -- this system is removed
