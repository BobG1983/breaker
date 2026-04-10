## Test Spec: Bolt Domain — Unified Death Pipeline Migration

### Domain
src/bolt/

### Overview
Migrate the bolt domain from the ad-hoc `RequestBoltDestroyed` / `cleanup_destroyed_bolts` death path to the unified death pipeline. After migration, bolt death flows through: `KillYourself<Bolt>` -> bolt kill handler (inserts `Dead`, removes from spatial index, sends `Destroyed<Bolt>` + `DespawnEntity`) -> `process_despawn_requests` (despawns entity). Two producers send `KillYourself<Bolt>`: `tick_bolt_lifespan` (timer expiry) and `bolt_lost` (off-screen, ExtraBolt only — baseline bolts get respawned, not killed). A third producer, `detect_bolt_deaths`, handles Hp-based death for future damage-to-bolt mechanics but is wave 7 scope, not wave 10.

**Scope note**: `apply_damage::<Bolt>` and `detect_bolt_deaths` are tested in wave 7 (death pipeline systems). This spec does NOT cover them. This spec covers: `tick_bolt_lifespan` migration, `bolt_lost` migration (including ExtraBolt distinction), `BoltLost` message field migration, the bolt domain kill handler, bolt builder Hp/KilledBy, and end-to-end flows.

---

### Prerequisites

The following shared types and systems must exist from previous waves (wave 2 scaffold + wave 7 death pipeline) before wave 10 tests can compile:

- `GameEntity` trait with `impl GameEntity for Bolt {}`
- `Hp { current: f32, starting: f32, max: Option<f32> }` component
- `KilledBy { dealer: Option<Entity> }` component
- `Dead` marker component
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` message with `::new()` constructor
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` message with `::new()` constructor
- `DespawnEntity { entity: Entity }` message
- `DeathPipelineSystems` system set enum with `ApplyDamage` and `DetectDeaths` variants
- `process_despawn_requests` system (shared)
- `apply_damage::<Bolt>` and `detect_bolt_deaths` systems (wave 7)
- `GlobalPosition2D` from `rantzsoft_spatial2d` (used for position extraction in kill handler)

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

3. **tick_bolt_lifespan does NOT tick timer for bolts with Birthing component (Without\<Birthing\> filter)**
   - Given: A bolt entity with `BoltLifespan` timer at 0.49 seconds remaining (total duration 0.5), delta time 0.02 seconds, AND a `Birthing` component
   - When: `tick_bolt_lifespan` runs once
   - Then: No `KillYourself::<Bolt>` message is sent. The timer remains at 0.49 seconds remaining (NOT 0.51) because the `Without<Birthing>` query filter means the system does not process this entity at all -- the timer does not tick.
   - Edge case: Bolt gains `Birthing` mid-lifespan (timer at 0.30/0.50) -- timer freezes at 0.30 while `Birthing` is present, resumes ticking when `Birthing` is removed

4. **tick_bolt_lifespan skips bolts with Dead component (Without\<Dead\> filter)**
   - Given: A bolt entity with `BoltLifespan` timer at 0.49 seconds remaining (total duration 0.5), delta time 0.02 seconds, AND a `Dead` component (already killed by another path)
   - When: `tick_bolt_lifespan` runs once
   - Then: No `KillYourself::<Bolt>` message is sent. Timer does not tick (entity excluded by `Without<Dead>` filter).
   - Edge case: Bolt marked Dead with timer at 0.50/0.50 (`just_finished()` would be true) -- still skipped

5. **tick_bolt_lifespan does NOT insert Dead component**
   - Given: A bolt entity with `BoltLifespan` timer that will expire this frame
   - When: `tick_bolt_lifespan` runs and sends `KillYourself::<Bolt>`
   - Then: The bolt entity does NOT have a `Dead` component inserted. Only the kill handler inserts `Dead`.
   - Edge case: N/A -- this is a negative assertion confirming separation of concerns

6. **tick_bolt_lifespan does NOT despawn the bolt**
   - Given: A bolt entity with `BoltLifespan` timer that will expire this frame
   - When: `tick_bolt_lifespan` runs and sends `KillYourself::<Bolt>`
   - Then: The bolt entity still exists in the world. `process_despawn_requests` handles despawn.
   - Edge case: N/A -- negative assertion

---

### Section B: bolt_lost Migration

**Critical distinction**: `bolt_lost` handles two kinds of bolts differently:
- **ExtraBolt** entities (spawned by Prism breaker on perfect bump): `bolt_lost` sends `KillYourself<Bolt>` to kill them via the death pipeline. They are NOT respawned.
- **Baseline bolts** (no `ExtraBolt` component): `bolt_lost` does NOT send `KillYourself<Bolt>`. Instead, the baseline bolt is respawned (existing respawn logic, unchanged). Baseline bolts are never killed by going off-screen.

All behaviors in this section specify `ExtraBolt` in the Given state unless testing baseline bolt behavior explicitly.

#### Behavior

7. **bolt_lost sends KillYourself\<Bolt\> for ExtraBolt when bolt leaves play area**
   - Given: A bolt entity with `ExtraBolt` component at position (100.0, -400.0) which is below the breaker/playfield bottom, a breaker entity at position (100.0, -300.0), no `Dead` component
   - When: `bolt_lost` detects the bolt is out of bounds
   - Then: Exactly one `KillYourself::<Bolt>` message is sent with `victim` equal to the bolt entity and `killer` equal to `None`
   - Edge case: Bolt at exactly the boundary position (e.g., y == playfield_bottom) -- depends on the boundary check being strictly less-than or less-than-or-equal; test the boundary

8. **bolt_lost sends BoltLost message with bolt and breaker fields populated**
   - Given: A bolt entity with `ExtraBolt` component (entity A) at position (100.0, -400.0), a breaker entity (entity B) at position (100.0, -300.0)
   - When: `bolt_lost` detects the bolt is out of bounds
   - Then: A `BoltLost { bolt: entity_A, breaker: entity_B }` message is sent with both fields correctly populated
   - Edge case: Multiple ExtraBolts lost in the same frame -- each generates its own `BoltLost` message with the correct bolt entity

9. **bolt_lost sends both KillYourself and BoltLost for same ExtraBolt**
   - Given: A bolt entity with `ExtraBolt` component at position (100.0, -400.0) that is below the playfield
   - When: `bolt_lost` runs
   - Then: Both a `KillYourself::<Bolt>` AND a `BoltLost` message are sent for the same bolt entity. The `KillYourself` feeds the death pipeline; the `BoltLost` feeds the effect system's `on_bolt_lost_occurred` bridge.
   - Edge case: N/A -- both messages always fire together for ExtraBolt off-screen events

10. **bolt_lost does NOT send KillYourself for baseline bolt (no ExtraBolt component)**
    - Given: A bolt entity WITHOUT `ExtraBolt` component at position (100.0, -400.0) below the playfield, a breaker entity at position (100.0, -300.0)
    - When: `bolt_lost` detects the bolt is out of bounds
    - Then: NO `KillYourself::<Bolt>` message is sent. The baseline bolt is respawned instead (respawn logic is unchanged and out of scope for this spec).
    - Edge case: Single bolt in play (baseline) goes off-screen -- must NOT be killed, must be respawned

11. **bolt_lost sends BoltLost for baseline bolt too (effect triggers still fire)**
    - Given: A bolt entity WITHOUT `ExtraBolt` component (entity A) at position (100.0, -400.0), a breaker entity (entity B) at position (100.0, -300.0)
    - When: `bolt_lost` detects the bolt is out of bounds
    - Then: A `BoltLost { bolt: entity_A, breaker: entity_B }` message IS sent. The effect system needs to know a bolt was lost regardless of whether it is an ExtraBolt or baseline (LoseLife trigger, etc.).
    - Edge case: N/A -- BoltLost fires for ALL bolt types

12. **bolt_lost does NOT insert Dead component**
    - Given: A bolt entity with `ExtraBolt` component that will be detected as lost
    - When: `bolt_lost` runs
    - Then: The bolt entity does NOT have a `Dead` component. The kill handler inserts `Dead`.
    - Edge case: N/A -- negative assertion

13. **bolt_lost does NOT despawn the bolt**
    - Given: A bolt entity with `ExtraBolt` component that will be detected as lost
    - When: `bolt_lost` runs
    - Then: The bolt entity still exists in the world
    - Edge case: N/A -- negative assertion

14. **bolt_lost skips bolts with Dead component (Without\<Dead\> filter)**
    - Given: A bolt entity with `ExtraBolt` component at position (100.0, -400.0) AND a `Dead` component (already killed by another path this frame)
    - When: `bolt_lost` runs
    - Then: No `KillYourself::<Bolt>` message is sent. No `BoltLost` message is sent. The already-dead bolt is not re-processed.
    - Edge case: ExtraBolt marked Dead at boundary position -- still skipped

---

### Section C: BoltLost Message Field Migration

#### Behavior

15. **BoltLost is a struct with bolt and breaker fields, not a unit struct**
    - Given: N/A -- type definition test
    - When: `BoltLost { bolt: Entity::PLACEHOLDER, breaker: Entity::PLACEHOLDER }` is constructed
    - Then: Compiles. The struct has two named `Entity` fields: `bolt` and `breaker`.
    - Edge case: Attempting to construct `BoltLost` as a unit struct (no fields) must NOT compile -- enforced by the type system, not a runtime test

---

### Section D: Bolt Kill Handler

#### Behavior

16. **Bolt kill handler inserts Dead on victim when processing KillYourself\<Bolt\>**
    - Given: A bolt entity (entity A) with `Bolt` component, `GlobalPosition2D` at (50.0, 200.0) inserted directly on the entity, no `Dead` component
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed by the bolt kill handler
    - Then: Entity A has a `Dead` component inserted
    - Edge case: `KillYourself` with a victim entity that does not have the `Bolt` component -- the kill handler should skip it (query uses `With<Bolt>` filter)

17. **Bolt kill handler removes bolt from spatial index**
    - Given: A bolt entity (entity A) inserted into the spatial index at position (50.0, 200.0)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: Entity A is no longer present in the spatial index query results
    - Edge case: Bolt not in the spatial index (already removed by another system) -- handler should not panic

18. **Bolt kill handler sends Destroyed\<Bolt\> with correct fields**
    - Given: A bolt entity (entity A) with `GlobalPosition2D` at (75.0, 150.0) inserted directly on the entity, killer is `None` (environmental death)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: A `Destroyed::<Bolt>` message is sent with `victim: entity_A`, `killer: None`, `victim_pos: Vec2::new(75.0, 150.0)`, `killer_pos: None`
    - Edge case: Bolt at position (0.0, 0.0) -- origin position still produces a valid Destroyed message

19. **Bolt kill handler sends Destroyed\<Bolt\> with killer position when killer exists**
    - Given: A bolt entity (entity A) with `GlobalPosition2D` at (75.0, 150.0) inserted directly on the entity, a killer entity (entity B) with `GlobalPosition2D` at (200.0, 300.0) inserted directly on it
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: Some(entity_B) }` message is processed
    - Then: A `Destroyed::<Bolt>` message is sent with `victim: entity_A`, `killer: Some(entity_B)`, `victim_pos: Vec2::new(75.0, 150.0)`, `killer_pos: Some(Vec2::new(200.0, 300.0))`
    - Edge case: Killer entity no longer exists in world at time of handler execution -- `killer_pos` should be `None` (entity despawned between KillYourself send and handler execution), but `killer` field still carries `Some(entity_B)` from the message

20. **Bolt kill handler sends DespawnEntity for the victim**
    - Given: A bolt entity (entity A)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: A `DespawnEntity { entity: entity_A }` message is sent
    - Edge case: N/A

21. **Bolt kill handler skips entities already marked Dead (same-frame double-kill prevention)**
    - Given: A bolt entity (entity A) with `Bolt` component AND `Dead` component already inserted (e.g., already killed by another path this frame)
    - When: A `KillYourself::<Bolt> { victim: entity_A, killer: None }` message is processed
    - Then: No `Destroyed::<Bolt>` message is sent. No `DespawnEntity` message is sent. No additional `Dead` component is inserted.
    - Edge case: Two `KillYourself::<Bolt>` messages for the same entity in the same frame -- only the first is processed. Because commands are deferred (Dead insertion is not visible until commands are applied), the kill handler must use a local `HashSet<Entity>` to track entities already processed in the current batch. The second message sees the entity in the HashSet and skips it, even though `Dead` is not yet visible on the entity.

22. **Bolt kill handler does NOT despawn the entity directly**
    - Given: A bolt entity (entity A) that receives `KillYourself::<Bolt>`
    - When: The kill handler runs
    - Then: Entity A still exists in the world (has `Dead` but is not despawned). Despawn is deferred to `process_despawn_requests` in PostFixedUpdate.
    - Edge case: N/A -- negative assertion

---

### Section E: Bolt Builder Hp and KilledBy

#### Behavior

23. **Bolt builder includes Hp component on spawned bolts**
    - Given: A bolt is spawned through the bolt builder (standard spawn path)
    - When: The bolt entity is inspected after spawn
    - Then: The entity has an `Hp` component with `current: 1.0`, `starting: 1.0`, `max: None` (bolts have 1 HP by default -- they die from any damage event, though most bolt deaths are environmental and bypass Hp)
    - Edge case: Bolt spawned with `Birthing` component -- still has `Hp` and `KilledBy` from spawn

24. **Bolt builder includes KilledBy component defaulting to None**
    - Given: A bolt is spawned through the bolt builder
    - When: The bolt entity is inspected after spawn
    - Then: The entity has a `KilledBy { dealer: None }` component
    - Edge case: N/A -- always defaults to None on fresh spawn

---

### Section F: End-to-End Integration

#### Behavior

25. **Lifespan expiry e2e: tick_bolt_lifespan through full death pipeline to despawn**
    - Given: A bolt entity with `BoltLifespan` timer at 0.49 seconds remaining (total duration 0.5), `Hp { current: 1.0, starting: 1.0, max: None }`, `KilledBy { dealer: None }`, registered in spatial index, `GlobalPosition2D` at (100.0, 200.0), delta time 0.02 seconds
    - When: The full FixedUpdate + PostFixedUpdate pipeline runs (tick_bolt_lifespan -> bolt kill handler -> process_despawn_requests)
    - Then:
      1. `tick_bolt_lifespan` sends `KillYourself::<Bolt> { victim: bolt_entity, killer: None }`
      2. Bolt kill handler: inserts `Dead` on bolt entity, removes from spatial index, sends `Destroyed::<Bolt> { victim: bolt_entity, killer: None, victim_pos: Vec2::new(100.0, 200.0), killer_pos: None }`, sends `DespawnEntity { entity: bolt_entity }`
      3. `process_despawn_requests`: despawns the bolt entity
      4. After full pipeline: bolt entity no longer exists in the world
    - Edge case: Bolt also has `Birthing` -- `tick_bolt_lifespan` should NOT fire (entity excluded from query), so the bolt survives

26. **ExtraBolt lost e2e: bolt_lost through full death pipeline to despawn**
    - Given: A bolt entity with `ExtraBolt` component at position (100.0, -400.0) below the playfield, a breaker entity at position (100.0, -300.0), bolt registered in spatial index, `Hp { current: 1.0, ... }`, `KilledBy { dealer: None }`
    - When: The full FixedUpdate + PostFixedUpdate pipeline runs (bolt_lost -> bolt kill handler -> process_despawn_requests)
    - Then:
      1. `bolt_lost` sends `KillYourself::<Bolt> { victim: bolt_entity, killer: None }` AND `BoltLost { bolt: bolt_entity, breaker: breaker_entity }`
      2. Bolt kill handler: inserts `Dead`, removes from spatial index, sends `Destroyed::<Bolt> { victim: bolt_entity, killer: None, victim_pos: Vec2::new(100.0, -400.0), killer_pos: None }`, sends `DespawnEntity { entity: bolt_entity }`
      3. `process_despawn_requests`: despawns the bolt entity
      4. After full pipeline: bolt entity no longer exists
    - Edge case: Multiple ExtraBolts lost in the same frame -- each independently goes through the full pipeline

27. **Baseline bolt lost e2e: bolt_lost respawns, does NOT kill**
    - Given: A bolt entity WITHOUT `ExtraBolt` at position (100.0, -400.0) below the playfield, a breaker entity at position (100.0, -300.0)
    - When: The full FixedUpdate pipeline runs
    - Then:
      1. `bolt_lost` sends `BoltLost { bolt: bolt_entity, breaker: breaker_entity }` (for effect triggers)
      2. `bolt_lost` does NOT send `KillYourself::<Bolt>` -- baseline bolt is respawned instead
      3. The bolt entity still exists (not killed, not despawned)
    - Edge case: N/A -- baseline bolt behavior is distinct from ExtraBolt

28. **Dead prevents double-processing across pipeline stages**
    - Given: A bolt entity with `ExtraBolt` that receives `KillYourself::<Bolt>` from BOTH `tick_bolt_lifespan` and `bolt_lost` in the same frame (unlikely but theoretically possible if lifespan expires the same frame the bolt goes out of bounds)
    - When: The bolt kill handler processes both messages
    - Then: Only the FIRST `KillYourself` is processed (inserts `Dead` via commands + tracked in local HashSet, sends `Destroyed`, sends `DespawnEntity`). The SECOND is skipped because the handler's local HashSet catches the duplicate within the same message batch. Only one `Destroyed::<Bolt>` and one `DespawnEntity` message are sent.
    - Edge case: Three KillYourself messages for same entity in one frame (two environmental + one Hp-based) -- only first processed

---

### Section G: RequestBoltDestroyed Removal

#### Behavior

29. **cleanup_destroyed_bolts system no longer exists**
    - Given: N/A -- structural assertion
    - When: The bolt plugin is built
    - Then: No system named `cleanup_destroyed_bolts` is registered. The domain kill handler replaces its functionality.
    - Note: This is verified by the kill handler e2e tests working without `cleanup_destroyed_bolts`. The system's file should be deleted.

---

### Out of Scope (Wave 7)

The following sections are tested in wave 7 (death pipeline systems spec), NOT in wave 10:

- `apply_damage::<Bolt>` -- generic shared system, monomorphized for Bolt. See wave 7 spec behaviors 8-9.
- `detect_bolt_deaths` -- Hp-based death detection system for bolts. See wave 7 spec behaviors 24-27.
- `process_despawn_requests` -- shared despawn system. See wave 7 spec behaviors 36-40.

Wave 10 tests MAY use these systems in e2e integration tests (Section F) but does not independently test their behavior.

---

### Types

#### New Types in This Wave
- `BoltLost { bolt: Entity, breaker: Entity }` -- message, replaces the old unit struct `BoltLost`. Derives: `Message, Clone, Debug`. Sent by `bolt_lost`, consumed by `on_bolt_lost_occurred` bridge and `spawn_bolt_lost_text`.

#### Shared Types (prerequisites, must exist before this wave)
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` -- message. Derives: `Message, Clone, Debug`.
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` -- message. Derives: `Message, Clone, Debug`.
- `DespawnEntity { entity: Entity }` -- message. Derives: `Message, Clone, Debug`.
- `Hp { current: f32, starting: f32, max: Option<f32> }` -- component. Derives: `Component, Debug, Clone`.
- `KilledBy { dealer: Option<Entity> }` -- component. Derives: `Component, Default, Debug`.
- `Dead` -- marker component. Derives: `Component`.
- `GameEntity` -- trait bound: `Component`. Impl'd on `Bolt`, `Cell`, `Wall`, `Breaker`.
- `DeathPipelineSystems` -- system set enum with `ApplyDamage` and `DetectDeaths` variants.
- `GlobalPosition2D` -- from `rantzsoft_spatial2d`, used for position extraction

#### Existing Types Used
- `Bolt` -- entity marker component
- `ExtraBolt` -- marker component for extra bolts spawned by Prism breaker
- `BoltLifespan` -- timer component for bolt lifespan
- `Birthing` -- marker component suppressing bolt processing during spawn animation
- `Breaker` -- entity marker component (for BoltLost breaker field)

---

### Messages

- `KillYourself::<Bolt> { victim: Entity, killer: Option<Entity> }` -- sent by `tick_bolt_lifespan`, `bolt_lost` (ExtraBolt only); consumed by bolt kill handler
- `Destroyed::<Bolt> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }` -- sent by bolt kill handler; consumed by `on_destroyed::<Bolt>` bridge (effect system)
- `DespawnEntity { entity: Entity }` -- sent by bolt kill handler; consumed by `process_despawn_requests`
- `BoltLost { bolt: Entity, breaker: Entity }` -- sent by `bolt_lost` (ALL bolt types); consumed by `on_bolt_lost_occurred` bridge, `spawn_bolt_lost_text`

---

### Reference Files

These are the design docs that define the behavior being tested. Writer-tests should read these for precise behavioral contracts:
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-modify/tick-bolt-lifespan.md` -- tick_bolt_lifespan migration spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-modify/bolt-lost.md` -- bolt_lost migration spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-remove.md` -- cleanup_destroyed_bolts removal
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
- Tests for bolt builder Hp/KilledBy go in: alongside existing bolt builder tests
- Tests for e2e integration go in: `src/bolt/systems/handle_bolt_kill/` (co-located with the kill handler which orchestrates the pipeline)

- Do NOT test: `apply_damage::<Bolt>` (wave 7 scope), `detect_bolt_deaths` (wave 7 scope), `on_destroyed::<Bolt>` bridge (effect domain, separate wave), `on_bolt_lost_occurred` bridge (effect domain, separate wave), `process_despawn_requests` (shared system, wave 7 scope), visual/fx responses to bolt death, bolt respawn logic (unchanged, tested separately), breaker domain kill handler, cell domain kill handler
- Do NOT reference existing source files in `src/` -- this is a clean-room implementation
- Do NOT import or reference `RequestBoltDestroyed` -- this type is removed
- Do NOT import or reference `cleanup_destroyed_bolts` -- this system is removed

- Compile-time guarantees (NOT runtime tests): The removal of `RequestBoltDestroyed` means any code referencing it will fail to compile. This is enforced by the type system, not by assertions. Similarly, the BoltLost struct field change is enforced by the compiler on all consumers.

- **GlobalPosition2D in tests**: Tests for the kill handler (Section D behaviors 16, 18, 19) must insert `GlobalPosition2D` directly on test entities rather than relying on transform propagation (which requires `rantzsoft_spatial2d` plugin systems to run). Use `entity.insert(GlobalPosition2D::new(Vec2::new(x, y)))` or equivalent direct construction.
