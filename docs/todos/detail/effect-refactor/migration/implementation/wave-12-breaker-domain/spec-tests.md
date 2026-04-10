## Test Spec: Breaker Domain — Unified Death Pipeline Migration

### Domains
- `src/shared/` (apply_damage generic system, shared types)
- `src/breaker/` (detect_breaker_deaths, breaker kill handler)

---

### Section A: apply_damage::\<Breaker\> (shared generic system, monomorphized for Breaker)

Tests go in: `src/shared/systems/apply_damage.rs` (alongside existing `apply_damage::<Cell>` tests, if any; otherwise new file with tests for the Breaker monomorphization)

#### Behavior

1. **DamageDealt\<Breaker\> decrements Hp**
   - Given: Breaker entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead` component
   - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: Breaker's `Hp.current` is 2.0. `KilledBy.dealer` remains `None` (not a killing blow).
   - Edge case: `amount: 0.0` — Hp remains 3.0, KilledBy unchanged

2. **Multiple DamageDealt\<Breaker\> messages in one frame are all applied**
   - Given: Breaker entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
   - When: Two `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` messages are sent, then `apply_damage::<Breaker>` runs
   - Then: Breaker's `Hp.current` is 1.0

3. **Killing blow sets KilledBy.dealer**
   - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`. A separate bolt entity exists.
   - When: `DamageDealt<Breaker> { dealer: Some(<bolt_entity>), target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: Breaker's `Hp.current` is 0.0. `KilledBy.dealer` is `Some(<bolt_entity>)`.
   - Edge case: Hp goes negative (amount: 5.0 when current: 1.0) — Hp.current is -4.0, KilledBy.dealer is still set (crossed from positive to non-positive)

4. **First kill wins — KilledBy not overwritten**
   - Given: Breaker entity with `Hp { current: 2.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`. Two separate entities: bolt_a and bolt_b.
   - When: Two messages sent in the same frame: `DamageDealt<Breaker> { dealer: Some(<bolt_a>), target: <breaker_entity>, amount: 1.0, ... }` and `DamageDealt<Breaker> { dealer: Some(<bolt_b>), target: <breaker_entity>, amount: 1.0, ... }`, then `apply_damage::<Breaker>` runs
   - Then: Hp.current is 0.0. `KilledBy.dealer` is `Some(<bolt_a>)` (first kill wins — the message that crossed from positive to zero). `bolt_b`'s damage is applied to Hp (now -1.0) but does not overwrite KilledBy.
   - Note: "first kill wins" means the first message processed that causes the crossing. The second message still decrements Hp but does not overwrite.

5. **Dead breaker is skipped by apply_damage**
   - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, AND `Dead` component
   - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: Breaker's `Hp.current` remains 0.0. `KilledBy.dealer` remains `None`. The `Dead` marker caused the entity to be filtered out via `Without<Dead>`.

6. **DamageDealt\<Breaker\> for nonexistent entity is silently ignored**
   - Given: No breaker entity in the world. A dummy Entity value.
   - When: `DamageDealt<Breaker> { dealer: None, target: <dummy_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: No panic, no error. System completes normally.

7. **Breaker without Hp is not affected by DamageDealt\<Breaker\>**
   - Given: Breaker entity with `Breaker` marker component but NO `Hp` and NO `KilledBy` (infinite lives breaker)
   - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: No panic. Entity is not queryable (no Hp/KilledBy components), so nothing happens. Breaker remains unchanged.

8. **Environmental death (dealer: None) sets KilledBy.dealer to None on killing blow**
   - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
   - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` runs
   - Then: Hp.current is 0.0. `KilledBy.dealer` remains `None` (killing blow was environmental — no dealer to set). The "killing blow" is detected (crossed from positive to zero), but since the message's dealer is None, KilledBy.dealer stays None.

---

### Section B: detect_breaker_deaths

Tests go in: `src/breaker/systems/detect_breaker_deaths.rs`

#### Behavior

9. **Breaker with Hp <= 0 triggers KillYourself\<Breaker\>**
   - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
   - When: `detect_breaker_deaths` runs
   - Then: A `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` message is sent
   - Edge case: Hp.current is -5.0 (overkill) — still sends exactly one `KillYourself<Breaker>`

10. **Breaker with positive Hp does not trigger death**
    - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `detect_breaker_deaths` runs
    - Then: No `KillYourself<Breaker>` messages are sent

11. **Dead breaker is not re-detected**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, AND `Dead` component
    - When: `detect_breaker_deaths` runs
    - Then: No `KillYourself<Breaker>` messages are sent. The `Without<Dead>` filter excludes this entity.

12. **KilledBy.dealer is propagated to KillYourself.killer**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: Some(<bolt_entity>) }`, no `Dead`. A separate bolt entity exists.
    - When: `detect_breaker_deaths` runs
    - Then: `KillYourself<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>) }` is sent

13. **Breaker without Hp is not detected (infinite lives)**
    - Given: Breaker entity with `Breaker` marker component but NO `Hp`, NO `KilledBy` (infinite lives breaker)
    - When: `detect_breaker_deaths` runs
    - Then: No `KillYourself<Breaker>` messages are sent. Entity is not queryable.

---

### Section C: Breaker Kill Handler (handle_kill_yourself_breaker)

Tests go in: `src/breaker/systems/handle_kill_yourself_breaker.rs`

The breaker kill handler is unique among domain kill handlers: it does NOT despawn the breaker entity. Instead, it transitions game state to indicate the run is lost. The breaker entity persists for death animation/fx.

#### Behavior

14. **KillYourself\<Breaker\> inserts Dead marker**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: Breaker entity has `Dead` component inserted

15. **KillYourself\<Breaker\> sends Destroyed\<Breaker\>**
    - Given: Breaker entity at position `Vec2::new(0.0, -250.0)` with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: `Destroyed<Breaker> { victim: <breaker_entity>, killer: None, victim_pos: Vec2::new(0.0, -250.0), killer_pos: None }` message is sent

16. **KillYourself\<Breaker\> sends DespawnEntity**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: `DespawnEntity { entity: <breaker_entity> }` message is sent
    - Note: The breaker entity is NOT immediately despawned. It persists through trigger evaluation and fx. The `DespawnEntity` message is processed later by `process_despawn_requests` in PostFixedUpdate.

17. **KillYourself\<Breaker\> sends RunLost message**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: A `RunLost` message is sent (triggering the run-lost state transition in the run domain)
    - Edge case: If the run domain is not wired, the message simply goes unconsumed. The kill handler's responsibility is to send it.

18. **KillYourself\<Breaker\> with a killer entity includes killer_pos in Destroyed**
    - Given: Breaker entity at position `Vec2::new(0.0, -250.0)`. A bolt entity at position `Vec2::new(100.0, 50.0)`. Breaker has `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: Some(<bolt_entity>) }`, no `Dead`.
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>) }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: `Destroyed<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>), victim_pos: Vec2::new(0.0, -250.0), killer_pos: Some(Vec2::new(100.0, 50.0)) }` message is sent

19. **Dead breaker ignores duplicate KillYourself\<Breaker\>**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, AND `Dead` component already present
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: No `Destroyed<Breaker>` sent. No `DespawnEntity` sent. No `RunLost` sent. The `Dead` marker prevents double-processing. The handler should use `Without<Dead>` or equivalent guard.

20. **Breaker entity is NOT despawned by the kill handler**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_kill_yourself_breaker` runs
    - Then: Breaker entity still exists in the world. It has `Dead` component but is not despawned. Entity can still be queried with `With<Breaker>`.

---

### Section D: Breaker Spawning with Hp

Tests go in: wherever the breaker builder or spawn system tests live. Since this is a clean-room spec, the test file should be `src/breaker/systems/spawn_breaker.rs` or equivalent breaker builder test location.

#### Behavior

21. **Breaker with life_pool: Some(3) gets Hp(3) and KilledBy**
    - Given: Breaker definition with `life_pool: Some(3)`
    - When: Breaker entity is spawned from this definition
    - Then: Entity has `Hp { current: 3.0, starting: 3.0, max: None }` and `KilledBy { dealer: None }`
    - Edge case: `life_pool: Some(1)` — gets `Hp { current: 1.0, starting: 1.0, max: None }`

22. **Breaker with life_pool: None does NOT get Hp or KilledBy**
    - Given: Breaker definition with `life_pool: None` (infinite lives — Chrono, Prism)
    - When: Breaker entity is spawned from this definition
    - Then: Entity does NOT have `Hp` component. Entity does NOT have `KilledBy` component.
    - Edge case: Confirm entity has `Breaker` marker but query `Option<&Hp>` returns `None`

23. **Breaker with life_pool: Some(0) gets Hp(0) — immediately killable**
    - Given: Breaker definition with `life_pool: Some(0)` (degenerate case)
    - When: Breaker entity is spawned from this definition
    - Then: Entity has `Hp { current: 0.0, starting: 0.0, max: None }` and `KilledBy { dealer: None }`
    - Note: This entity would be detected as dead on the next detect_breaker_deaths pass. This is intentional — the data is correct even if the gameplay scenario is nonsensical.

---

### Section E: End-to-End Breaker Death Flow

Tests go in: `src/breaker/systems/detect_breaker_deaths.rs` (or a dedicated integration test file)

These tests verify the full chain works when multiple systems are wired together. They use a headless Bevy app with MinimalPlugins.

#### Behavior

24. **Full death chain: DamageDealt\<Breaker\> to death to RunLost**
    - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then the following systems run in order: `apply_damage::<Breaker>` then `detect_breaker_deaths` then `handle_kill_yourself_breaker`
    - Then: Hp.current is 0.0. `Dead` is inserted. `KillYourself<Breaker>` was sent (consumed by handler). `Destroyed<Breaker>` was sent. `DespawnEntity` was sent. `RunLost` was sent. Breaker entity still exists (not despawned — that happens in PostFixedUpdate).

25. **Repeated damage over multiple frames depletes lives**
    - Given: Breaker entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: Frame 1: `DamageDealt<Breaker> { amount: 1.0, ... }` → full pipeline runs → Hp.current is 2.0, no death. Frame 2: same → Hp.current is 1.0, no death. Frame 3: same → Hp.current is 0.0, death chain fires.
    - Then: After frame 3: `Dead` is inserted, `KillYourself<Breaker>` sent, `Destroyed<Breaker>` sent, `RunLost` sent.
    - Edge case: Frame 4: another `DamageDealt<Breaker>` arrives — `Dead` prevents apply_damage from processing it, detect_breaker_deaths skips it, kill handler skips it. No second `RunLost`.

26. **Infinite lives breaker survives entire damage chain**
    - Given: Breaker entity with `Breaker` marker, NO `Hp`, NO `KilledBy` (infinite lives)
    - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` then `detect_breaker_deaths` then `handle_kill_yourself_breaker` all run
    - Then: No changes to the breaker entity. No `KillYourself<Breaker>` sent. No `Destroyed<Breaker>` sent. No `RunLost` sent. Breaker still exists, no `Dead` component.

---

### Types

All types below are defined in the unified death pipeline docs. They should already exist (or be created as shared prerequisites) by the time these tests are written.

- `Hp { current: f32, starting: f32, max: Option<f32> }` — `#[derive(Component, Debug, Clone)]` — unified health component. Location: `src/shared/components/`
- `KilledBy { dealer: Option<Entity> }` — `#[derive(Component, Default, Debug)]` — kill attribution. Location: `src/shared/components/`
- `Dead` — `#[derive(Component)]` — marker for confirmed-dead entities. Location: `src/shared/components/`
- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — generic damage message. Location: `src/shared/messages/`
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — death request. Location: `src/shared/messages/`
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — death confirmed. Location: `src/shared/messages/`
- `DespawnEntity { entity: Entity }` — `#[derive(Message, Clone, Debug)]` — deferred despawn. Location: `src/shared/messages/`
- `GameEntity` trait — `trait GameEntity: Component {}` with `impl GameEntity for Breaker {}`. Location: `src/shared/`
- `RunLost` — existing message. Location: already exists in messages.
- `DamageTargetData` — `#[derive(QueryData)] #[query_data(mutable)]` with `hp: &'static mut Hp, killed_by: &'static mut KilledBy`. Location: `src/shared/queries.rs`
- `DeathDetectionData` — `#[derive(QueryData)]` with `entity: Entity, killed_by: &'static KilledBy, hp: &'static Hp`. Location: `src/shared/queries.rs`

### Messages

- `DamageDealt<Breaker>` — sent by LoseLife effect, consumed by `apply_damage::<Breaker>`
- `KillYourself<Breaker>` — sent by `detect_breaker_deaths`, consumed by `handle_kill_yourself_breaker`
- `Destroyed<Breaker>` — sent by `handle_kill_yourself_breaker`, consumed by death bridges / fx / triggers
- `DespawnEntity` — sent by `handle_kill_yourself_breaker`, consumed by `process_despawn_requests`
- `RunLost` — sent by `handle_kill_yourself_breaker`, consumed by run domain

### Reference Files
- `docs/todos/detail/unified-death-pipeline/rust-types/` — all type definitions
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/apply-damage-breaker.md` — apply_damage system spec
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/detect-breaker-deaths.md` — detect_breaker_deaths system spec
- `docs/todos/detail/unified-death-pipeline/migration/query-data-to-create/damage-target-data.md` — DamageTargetData
- `docs/todos/detail/unified-death-pipeline/migration/query-data-to-create/death-detection-data.md` — DeathDetectionData
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-sets.md` — DeathPipelineSystems
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` — ordering

### Scenario Coverage
- New invariants: none — `BreakerCountReasonable` already exists. The breaker is not despawned by the kill handler, so entity count invariants remain valid. A future invariant for "breaker Hp never negative without Dead" could be added but is not required for this migration.
- New scenarios: none — existing Aegis scenarios exercise the life loss path. After migration, the same scenarios validate the new pipeline.
- Self-test scenarios: none — no new InvariantKind introduced
- Layout updates: none

### Constraints
- Tests for apply_damage::\<Breaker\> go in: `src/shared/systems/apply_damage.rs` (tests module)
- Tests for detect_breaker_deaths go in: `src/breaker/systems/detect_breaker_deaths.rs` (new file, system + tests)
- Tests for handle_kill_yourself_breaker go in: `src/breaker/systems/handle_kill_yourself_breaker.rs` (new file, system + tests)
- Tests for breaker spawning with Hp go in: wherever breaker spawn/builder tests live
- Do NOT test: The LoseLife effect itself (that is the effect domain, out of scope). Do NOT test `process_despawn_requests` (shared infrastructure, separate wave). Do NOT test death bridges / trigger dispatch (effect domain). Do NOT test the run domain's handling of `RunLost` (run domain scope). Do NOT test other entity types (Cell, Bolt, Wall — separate waves).
- Do NOT reference existing `src/` code. This is a clean-room implementation.
