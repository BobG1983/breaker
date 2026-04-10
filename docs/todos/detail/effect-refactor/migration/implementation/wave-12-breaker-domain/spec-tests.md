## Test Spec: Breaker Domain — Unified Death Pipeline Migration

### Domains
- `breaker-game/src/breaker/` (breaker kill handler, builder Hp/KilledBy insertion)

---

### Prerequisites

Wave 12 depends on these earlier waves being complete:

- **Wave 2 (Scaffold)**: Shared types must exist — `Hp`, `KilledBy`, `Dead`, `DamageDealt<T>`, `KillYourself<T>`, `Destroyed<T>`, `GameEntity`, `DeathDetectionData`, `DeathPipelineSystems` set enum. Without these, tests cannot compile.
- **Wave 7 (Death Pipeline Core)**: `apply_damage::<Breaker>` and `detect_breaker_deaths` systems must be implemented and tested. Wave 12 does NOT re-test these systems — it only tests the breaker kill handler and builder changes.

---

### Section A: apply_damage::\<Breaker\> — OUT OF SCOPE

`apply_damage::<Breaker>` is tested in **wave 7** (death pipeline core). Wave 12 does NOT duplicate those tests.

---

### Section B: detect_breaker_deaths — OUT OF SCOPE

`detect_breaker_deaths` is tested in **wave 7** (death pipeline core). Wave 12 does NOT duplicate those tests.

---

### Section C: Breaker Kill Handler (handle_breaker_kill)

Tests go in: `breaker-game/src/breaker/systems/handle_breaker_kill.rs` (new file — system + tests, or directory module if tests exceed threshold)

The breaker kill handler is unique among domain kill handlers: it does NOT despawn the breaker entity and does NOT send DespawnEntity. The breaker persists through game-over — cleanup happens via `CleanupOnExit<RunState>` when the run state exits. Instead, the handler transitions game state by sending `RunLost`.

#### Behavior

14. **KillYourself\<Breaker\> inserts Dead marker**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: Breaker entity has `Dead` component inserted

15. **KillYourself\<Breaker\> sends Destroyed\<Breaker\>**
    - Given: Breaker entity at position `Vec2::new(0.0, -250.0)` with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: `Destroyed<Breaker> { victim: <breaker_entity>, killer: None, victim_pos: Vec2::new(0.0, -250.0), killer_pos: None }` message is sent

16. **KillYourself\<Breaker\> sends RunLost message**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: A `RunLost` message is sent (triggering the run-lost state transition in the run domain)
    - Edge case: If the run domain is not wired, the message simply goes unconsumed. The kill handler's responsibility is to send it.

17. **KillYourself\<Breaker\> with a killer entity includes killer_pos in Destroyed**
    - Given: Breaker entity at position `Vec2::new(0.0, -250.0)`. A bolt entity at position `Vec2::new(100.0, 50.0)`. Breaker has `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: Some(<bolt_entity>) }`, no `Dead`.
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>) }` is sent, then `handle_breaker_kill` runs
    - Then: `Destroyed<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>), victim_pos: Vec2::new(0.0, -250.0), killer_pos: Some(Vec2::new(100.0, 50.0)) }` message is sent

18. **Dead breaker ignores duplicate KillYourself\<Breaker\>**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, AND `Dead` component already present
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: No `Destroyed<Breaker>` sent. No `RunLost` sent. The `Dead` marker prevents double-processing. The handler should use `Without<Dead>` or equivalent guard.

19. **Breaker entity is NOT despawned by the kill handler**
    - Given: Breaker entity with `Hp { current: 0.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: Breaker entity still exists in the world. It has `Dead` component but is not despawned. Entity can still be queried with `With<Breaker>`. No `DespawnEntity` message is sent. Cleanup happens later via `CleanupOnExit<RunState>`.

20. **KillYourself\<Breaker\> with killer entity gone (despawned) sets killer_pos to None**
    - Given: Breaker entity at position `Vec2::new(0.0, -250.0)` with `KilledBy { dealer: Some(<bolt_entity>) }`, no `Dead`. The bolt entity has been despawned (no longer exists in the world).
    - When: `KillYourself<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>) }` is sent, then `handle_breaker_kill` runs
    - Then: `Destroyed<Breaker> { victim: <breaker_entity>, killer: Some(<bolt_entity>), victim_pos: Vec2::new(0.0, -250.0), killer_pos: None }` message is sent. The killer entity reference is preserved in the message but position lookup returns None because the entity no longer exists.

21. **KillYourself\<Breaker\> for stale victim entity is skipped gracefully**
    - Given: No breaker entity exists in the world. A dummy Entity value is used.
    - When: `KillYourself<Breaker> { victim: <dummy_entity>, killer: None }` is sent, then `handle_breaker_kill` runs
    - Then: No panic, no error. No `Destroyed<Breaker>` sent. No `RunLost` sent. System completes normally. The entity lookup returns nothing, so the message is silently skipped.

---

### Section D: Breaker Spawning with Hp

Tests go in: `breaker-game/src/breaker/builder/tests/build_tests.rs` (alongside existing builder tests)

#### Behavior

22. **Breaker with life_pool: Some(3) gets Hp(3) and KilledBy**
    - Given: Breaker definition with `life_pool: Some(3)`
    - When: Breaker entity is spawned from this definition
    - Then: Entity has `Hp { current: 3.0, starting: 3.0, max: None }` and `KilledBy { dealer: None }`
    - Edge case: `life_pool: Some(1)` — gets `Hp { current: 1.0, starting: 1.0, max: None }`

23. **Breaker with life_pool: None does NOT get Hp or KilledBy**
    - Given: Breaker definition with `life_pool: None` (infinite lives — Chrono, Prism)
    - When: Breaker entity is spawned from this definition
    - Then: Entity does NOT have `Hp` component. Entity does NOT have `KilledBy` component.
    - Edge case: Confirm entity has `Breaker` marker but query `Option<&Hp>` returns `None`

24. **Breaker with life_pool: Some(0) gets Hp(0) — immediately killable**
    - Given: Breaker definition with `life_pool: Some(0)` (degenerate case)
    - When: Breaker entity is spawned from this definition
    - Then: Entity has `Hp { current: 0.0, starting: 0.0, max: None }` and `KilledBy { dealer: None }`
    - Note: This entity would be detected as dead on the next detect_breaker_deaths pass. This is intentional — the data is correct even if the gameplay scenario is nonsensical.

25. **Breaker entity does NOT have LivesCount after spawning with new builder**
    - Given: Breaker definition with `life_pool: Some(3)`
    - When: Breaker entity is spawned from the updated builder
    - Then: Entity does NOT have `LivesCount` component. `Hp` replaces `LivesCount` for tracking breaker life state.

---

### Section E: End-to-End Breaker Death Flow

Tests go in: `breaker-game/src/breaker/systems/handle_breaker_kill.rs` (or a dedicated integration test file alongside the kill handler)

These tests verify the full chain works when multiple systems are wired together. They use a headless Bevy app with MinimalPlugins.

#### Behavior

26. **Full death chain: DamageDealt\<Breaker\> to death to RunLost**
    - Given: Breaker entity with `Hp { current: 1.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then the following systems run in order: `apply_damage::<Breaker>` then `detect_breaker_deaths` then `handle_breaker_kill`
    - Then: Hp.current is 0.0. `Dead` is inserted. `KillYourself<Breaker>` was sent (consumed by handler). `Destroyed<Breaker>` was sent. `RunLost` was sent. Breaker entity still exists (not despawned — cleanup happens via `CleanupOnExit<RunState>`). No `DespawnEntity` was sent.

27. **Repeated damage over multiple frames depletes lives**
    - Given: Breaker entity with `Hp { current: 3.0, starting: 3.0, max: None }`, `KilledBy { dealer: None }`, no `Dead`
    - When: Frame 1: `DamageDealt<Breaker> { amount: 1.0, ... }` then full pipeline runs then Hp.current is 2.0, no death. Frame 2: same then Hp.current is 1.0, no death. Frame 3: same then Hp.current is 0.0, death chain fires.
    - Then: After frame 3: `Dead` is inserted, `KillYourself<Breaker>` sent, `Destroyed<Breaker>` sent, `RunLost` sent. No `DespawnEntity` sent.
    - Edge case: Frame 4: another `DamageDealt<Breaker>` arrives — `Dead` prevents apply_damage from processing it, detect_breaker_deaths skips it, kill handler skips it. No second `RunLost`.

28. **Infinite lives breaker survives entire damage chain**
    - Given: Breaker entity with `Breaker` marker, NO `Hp`, NO `KilledBy` (infinite lives)
    - When: `DamageDealt<Breaker> { dealer: None, target: <breaker_entity>, amount: 1.0, source_chip: None }` is sent, then `apply_damage::<Breaker>` then `detect_breaker_deaths` then `handle_breaker_kill` all run
    - Then: No changes to the breaker entity. No `KillYourself<Breaker>` sent. No `Destroyed<Breaker>` sent. No `RunLost` sent. Breaker still exists, no `Dead` component.

---

### Types

All types below are defined in the unified death pipeline docs. They should already exist (or be created as shared prerequisites) by the time these tests are written.

- `Hp { current: f32, starting: f32, max: Option<f32> }` — `#[derive(Component, Debug, Clone)]` — unified health component. Location: `breaker-game/src/shared/components/`
- `KilledBy { dealer: Option<Entity> }` — `#[derive(Component, Default, Debug)]` — kill attribution. Location: `breaker-game/src/shared/components/`
- `Dead` — `#[derive(Component)]` — marker for confirmed-dead entities. Location: `breaker-game/src/shared/components/`
- `DamageDealt<T: GameEntity> { dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — generic damage message. Location: `breaker-game/src/shared/messages/`
- `KillYourself<T: GameEntity> { victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — death request. Location: `breaker-game/src/shared/messages/`
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2>, _marker: PhantomData<T> }` — `#[derive(Message, Clone, Debug)]` — death confirmed. Location: `breaker-game/src/shared/messages/`
- `GameEntity` trait — `trait GameEntity: Component {}` with `impl GameEntity for Breaker {}`. Location: `breaker-game/src/shared/`
- `RunLost` — existing message. Location: `breaker-game/src/state/run/messages.rs`
- `DeathDetectionData` — `#[derive(QueryData)]` with `entity: Entity, killed_by: &'static KilledBy, hp: &'static Hp`. Location: `breaker-game/src/shared/queries.rs`

### Messages

- `DamageDealt<Breaker>` — sent by LoseLife effect (updated in this wave), consumed by `apply_damage::<Breaker>`
- `KillYourself<Breaker>` — sent by `detect_breaker_deaths`, consumed by `handle_breaker_kill`
- `Destroyed<Breaker>` — sent by `handle_breaker_kill`, consumed by death bridges / fx / triggers
- `RunLost` — sent by `handle_breaker_kill`, consumed by run domain

Note: The breaker kill handler does NOT send `DespawnEntity`. The breaker entity persists through game-over and is cleaned up by `CleanupOnExit<RunState>`.

### Reference Files
- `docs/todos/detail/unified-death-pipeline/rust-types/` — all type definitions
- `docs/todos/detail/unified-death-pipeline/migration/systems-to-create/detect-breaker-deaths.md` — detect_breaker_deaths system spec
- `docs/todos/detail/unified-death-pipeline/migration/query-data-to-create/death-detection-data.md` — DeathDetectionData
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-sets.md` — DeathPipelineSystems
- `docs/todos/detail/unified-death-pipeline/migration/plugin-wiring/system-set-ordering.md` — ordering

### Scenario Coverage
- New invariants: none — `BreakerCountReasonable` already exists. The breaker is not despawned by the kill handler, so entity count invariants remain valid. A future invariant for "breaker Hp never negative without Dead" could be added but is not required for this migration.
- New scenarios: none — existing Aegis scenarios exercise the life loss path. After migration, the same scenarios validate the new pipeline.
- Self-test scenarios: none — no new InvariantKind introduced
- Layout updates: none

### Constraints
- `apply_damage::<Breaker>` is tested in wave 7 — do NOT duplicate those tests here.
- `detect_breaker_deaths` is tested in wave 7 — do NOT duplicate those tests here.
- LivesCount is removed from the breaker builder — `Hp` replaces it. The `LoseLife` effect's `fire()` is updated to send `DamageDealt<Breaker>` instead of directly mutating `LivesCount`.
- Tests for handle_breaker_kill go in: `breaker-game/src/breaker/systems/handle_breaker_kill.rs` (new file, system + tests)
- Tests for breaker spawning with Hp go in: `breaker-game/src/breaker/builder/tests/build_tests.rs` (existing file)
- Do NOT test: The LoseLife effect itself (that is the effect domain, out of scope). Do NOT test death bridges / trigger dispatch (effect domain). Do NOT test the run domain's handling of `RunLost` (run domain scope). Do NOT test other entity types (Cell, Bolt, Wall — separate waves).
