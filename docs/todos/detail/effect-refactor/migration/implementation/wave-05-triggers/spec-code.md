## Implementation Spec: Effect — Wave 5 Trigger Bridge Systems

### Domain
`src/effect/triggers/`

### Failing Tests
- `src/effect/triggers/bump/bridges.rs` (or `tests.rs` if split) — tests for all 10 bump bridge systems
- `src/effect/triggers/impact/bridges.rs` (or `tests.rs`) — tests for 12 impact bridge systems (6 on_impacted + 6 on_impact_occurred)
- `src/effect/triggers/death/bridges.rs` (or `tests.rs`) — tests for 4 monomorphized death bridge systems
- `src/effect/triggers/bolt_lost/bridges.rs` (or `tests.rs`) — tests for on_bolt_lost_occurred
- `src/effect/triggers/node/bridges.rs` (or `tests.rs`) — tests for on_node_start_occurred, on_node_end_occurred, on_node_timer_threshold_occurred
- `src/effect/triggers/node/check_thresholds.rs` (or `tests.rs`) — tests for check_node_timer_thresholds game system
- `src/effect/triggers/node/resources.rs` — tests for reset_node_timer_thresholds
- `src/effect/triggers/time/bridges.rs` (or `tests.rs`) — tests for on_time_expires
- `src/effect/triggers/time/tick_timers.rs` (or `tests.rs`) — tests for tick_effect_timers game system
- `src/effect/conditions/track_combo_streak.rs` (or `tests.rs` if split) — tests for track_combo_streak
- `src/effect/storage/watch_spawn_registry.rs` (or `tests.rs` if split) — tests for watch_spawn_registry

Exact file paths will be determined by the test spec. The writer-code must find and satisfy all failing tests in these locations.

---

### What to Implement

#### Bump Bridge Systems (10 systems in `src/effect/triggers/bump/bridges.rs`)

1. **`on_bumped`**: Local bridge. Reads `BumpPerformed` messages. For any successful grade (Perfect, Early, Late), builds `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`. If `msg.bolt` is `Some(bolt)`, queries bolt for `(&BoundEffects, &StagedEffects)` and calls `walk_effects(bolt, &Trigger::Bumped, &context, bound, staged, &mut commands)`. Then queries breaker and calls `walk_effects` on it.

2. **`on_perfect_bumped`**: Local bridge. Reads `BumpPerformed`. Filters `msg.grade == BumpGrade::Perfect`. Same Local walk pattern as `on_bumped` but with `Trigger::PerfectBumped`.

3. **`on_early_bumped`**: Local bridge. Reads `BumpPerformed`. Filters `msg.grade == BumpGrade::Early`. Same Local walk pattern with `Trigger::EarlyBumped`.

4. **`on_late_bumped`**: Local bridge. Reads `BumpPerformed`. Filters `msg.grade == BumpGrade::Late`. Same Local walk pattern with `Trigger::LateBumped`.

5. **`on_bump_occurred`**: Global bridge. Reads `BumpPerformed`. For any successful grade, builds `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`. Iterates ALL entities with `(Entity, &BoundEffects, &StagedEffects)` and calls `walk_effects` with `Trigger::BumpOccurred`.

6. **`on_perfect_bump_occurred`**: Global bridge. Reads `BumpPerformed`. Filters `BumpGrade::Perfect`. Global walk with `Trigger::PerfectBumpOccurred`. Context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.

7. **`on_early_bump_occurred`**: Global bridge. Reads `BumpPerformed`. Filters `BumpGrade::Early`. Global walk with `Trigger::EarlyBumpOccurred`. Context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.

8. **`on_late_bump_occurred`**: Global bridge. Reads `BumpPerformed`. Filters `BumpGrade::Late`. Global walk with `Trigger::LateBumpOccurred`. Context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.

9. **`on_bump_whiff_occurred`**: Global bridge. Reads `BumpWhiffed` (unit message, no fields). Builds `TriggerContext::None`. Global walk with `Trigger::BumpWhiffOccurred`.

10. **`on_no_bump_occurred`**: Global bridge. Reads `BoltImpactBreaker` messages. Filters `msg.bump_status == BumpStatus::Inactive`. Builds `TriggerContext::Bump { bolt: Some(msg.bolt), breaker: msg.breaker }`. Global walk with `Trigger::NoBumpOccurred`.

#### Impact Bridge Systems (12 systems in `src/effect/triggers/impact/bridges.rs`)

Six `on_impacted_*` systems (Local) and six `on_impact_occurred_*` systems (Global). One of each per collision message type.

11. **`on_impacted_bolt_cell`**: Reads `BoltImpactCell { cell, bolt }`. Context: `TriggerContext::Impact { impactor: bolt, impactee: cell }`. Walks bolt with `Trigger::Impacted(EntityKind::Cell)`, walks cell with `Trigger::Impacted(EntityKind::Bolt)`. Same context for both.

12. **`on_impacted_bolt_wall`**: Reads `BoltImpactWall { bolt, wall }`. Context: `Impact { impactor: bolt, impactee: wall }`. Walks bolt with `Impacted(Wall)`, walks wall with `Impacted(Bolt)`.

13. **`on_impacted_bolt_breaker`**: Reads `BoltImpactBreaker { bolt, breaker, .. }`. Context: `Impact { impactor: bolt, impactee: breaker }`. Walks bolt with `Impacted(Breaker)`, walks breaker with `Impacted(Bolt)`.

14. **`on_impacted_breaker_cell`**: Reads `BreakerImpactCell { breaker, cell }`. Context: `Impact { impactor: breaker, impactee: cell }`. Walks breaker with `Impacted(Cell)`, walks cell with `Impacted(Breaker)`.

15. **`on_impacted_breaker_wall`**: Reads `BreakerImpactWall { breaker, wall }`. Context: `Impact { impactor: breaker, impactee: wall }`. Walks breaker with `Impacted(Wall)`, walks wall with `Impacted(Breaker)`.

16. **`on_impacted_cell_wall`**: Reads `CellImpactWall { cell, wall }`. Context: `Impact { impactor: cell, impactee: wall }`. Walks cell with `Impacted(Wall)`, walks wall with `Impacted(Cell)`.

17. **`on_impact_occurred_bolt_cell`**: Reads `BoltImpactCell`. Context: `Impact { impactor: bolt, impactee: cell }`. Global sweep 1: `ImpactOccurred(EntityKind::Cell)`. Global sweep 2: `ImpactOccurred(EntityKind::Bolt)`. Same context for both sweeps.

18. **`on_impact_occurred_bolt_wall`**: Reads `BoltImpactWall`. Context: `Impact { impactor: bolt, impactee: wall }`. Sweep 1: `ImpactOccurred(Wall)`. Sweep 2: `ImpactOccurred(Bolt)`.

19. **`on_impact_occurred_bolt_breaker`**: Reads `BoltImpactBreaker`. Context: `Impact { impactor: bolt, impactee: breaker }`. Sweep 1: `ImpactOccurred(Breaker)`. Sweep 2: `ImpactOccurred(Bolt)`.

20. **`on_impact_occurred_breaker_cell`**: Reads `BreakerImpactCell`. Context: `Impact { impactor: breaker, impactee: cell }`. Sweep 1: `ImpactOccurred(Cell)`. Sweep 2: `ImpactOccurred(Breaker)`.

21. **`on_impact_occurred_breaker_wall`**: Reads `BreakerImpactWall`. Context: `Impact { impactor: breaker, impactee: wall }`. Sweep 1: `ImpactOccurred(Wall)`. Sweep 2: `ImpactOccurred(Breaker)`.

22. **`on_impact_occurred_cell_wall`**: Reads `CellImpactWall`. Context: `Impact { impactor: cell, impactee: wall }`. Sweep 1: `ImpactOccurred(Wall)`. Sweep 2: `ImpactOccurred(Cell)`.

#### Death Bridge Systems (4 monomorphized in `src/effect/triggers/death/bridges.rs`)

A single generic system `on_destroyed<T: GameEntity>` monomorphized for Cell, Bolt, Wall, Breaker. Each reads `Destroyed<T>` messages (from previous frame via Bevy message persistence).

23. **`on_destroyed::<Cell>`**: Reads `Destroyed<Cell>`. For each message:
    - Step 1: Walk victim with `Trigger::Died`, context `TriggerContext::Death { victim: msg.victim, killer: msg.killer }`.
    - Step 2: If `msg.killer` is `Some(killer)` AND killer entity exists in world, classify killer's `EntityKind` by checking for Bolt/Breaker/Cell/Wall component markers, then walk killer with `Trigger::Killed(EntityKind::Cell)`, same context. If killer entity no longer exists, emit a debug warning and skip.
    - Step 3: Walk all entities with BoundEffects/StagedEffects with `Trigger::DeathOccurred(EntityKind::Cell)`, same context.
    - Order: Died before Killed before DeathOccurred.

24. **`on_destroyed::<Bolt>`**: Same pattern, reads `Destroyed<Bolt>`, dispatches `Killed(Bolt)` and `DeathOccurred(Bolt)`. Most bolt deaths are environmental (killer = None), so Killed is usually skipped.

25. **`on_destroyed::<Wall>`**: Same pattern, reads `Destroyed<Wall>`, dispatches `Killed(Wall)` and `DeathOccurred(Wall)`.

26. **`on_destroyed::<Breaker>`**: Same pattern, reads `Destroyed<Breaker>`, dispatches `Killed(Breaker)` and `DeathOccurred(Breaker)`. Breaker deaths are typically environmental.

**Killer classification helper**: A function (or inline match) that takes an `Entity` and a `&World` (or component queries) and returns `EntityKind` by checking which marker component (Bolt, Cell, Wall, Breaker) is present. Returns `EntityKind::Any` as fallback if none match (should not happen in practice, debug_assert against it).

#### Bolt Lost Bridge System (1 system in `src/effect/triggers/bolt_lost/bridges.rs`)

27. **`on_bolt_lost_occurred`**: Global bridge. Reads `BoltLost { bolt, breaker }` message (note: migrated from unit struct to carry both entities). Builds `TriggerContext::BoltLost { bolt: msg.bolt, breaker: msg.breaker }`. Global walk all entities with `Trigger::BoltLostOccurred`.

#### Node Bridge Systems (3 in `src/effect/triggers/node/bridges.rs`)

28. **`on_node_start_occurred`**: Global bridge. Runs on `OnEnter(NodeState::Playing)` (NOT FixedUpdate). Walks all entities with BoundEffects/StagedEffects with `Trigger::NodeStartOccurred` and `TriggerContext::None`.

29. **`on_node_end_occurred`**: Global bridge. Runs on `OnExit(NodeState::Playing)` (NOT FixedUpdate). Walks all entities with BoundEffects/StagedEffects with `Trigger::NodeEndOccurred` and `TriggerContext::None`.

30. **`on_node_timer_threshold_occurred`**: Global bridge. Reads `NodeTimerThresholdCrossed { ratio }` message. Walks all entities with `Trigger::NodeTimerThresholdOccurred(ratio)` and `TriggerContext::None`.

#### Node Game System (1 in `src/effect/triggers/node/check_thresholds.rs`)

31. **`check_node_timer_thresholds`**: Game system (not a bridge). Reads `Res<NodeTimer>` (from `crate::state::run::node::resources::NodeTimer`) to compute the current node timer ratio as `(node_timer.total - node_timer.remaining) / node_timer.total`. Reads `ResMut<NodeTimerThresholdRegistry>` resource. For each threshold in `thresholds` where `ratio >= threshold` and threshold NOT in `fired`: sends `NodeTimerThresholdCrossed { ratio: threshold.into_inner() }` message and inserts threshold into `fired`. If `node_timer.total` is `0.0`, the system is a no-op (avoids division by zero).

#### Node Reset System (1 in `src/effect/triggers/node/register.rs` or separate file)

32. **`reset_node_timer_thresholds`**: Runs on `OnEnter(NodeState::Playing)`. Clears `fired` set in `NodeTimerThresholdRegistry` by calling `registry.fired.clear()`. Does NOT clear `thresholds` — those persist across nodes since effect trees don't change mid-run.

#### Time Bridge System (1 in `src/effect/triggers/time/bridges.rs`)

33. **`on_time_expires`**: Self-scoped bridge. Reads `EffectTimerExpired { entity, original_duration }` message. Queries the referenced entity for `(&BoundEffects, &StagedEffects)`. Walks ONLY that entity with `Trigger::TimeExpires(msg.original_duration.into_inner())` and `TriggerContext::None`. The `original_duration` is read from the message, NOT from the entity's `EffectTimers` component. By the time this bridge runs (next frame, due to Bridge < Tick set ordering), `tick_effect_timers` has already removed the expired timer entry from `EffectTimers`.

**Implementation note on TimeExpires duration**: The `EffectTimerExpired` message carries the `original_duration` so the bridge can construct `Trigger::TimeExpires(original_duration)`. Per the authoritative type doc (`docs/todos/detail/effect-refactor/rust-types/messages/effect-timer-expired.md`), the message definition is `EffectTimerExpired { entity: Entity, original_duration: OrderedFloat<f32> }`. The `tick_effect_timers` system includes the `original_duration` when sending.

#### Time Game System (1 in `src/effect/triggers/time/tick_timers.rs`)

34. **`tick_effect_timers`**: Game system (not a bridge). Queries all entities with `&mut EffectTimers`. For each entity, iterates `timers` vec. Decrements `remaining_seconds` by `time.delta_secs()` (from `Time<Fixed>`). If `remaining_seconds <= 0.0`: sends `EffectTimerExpired { entity, original_duration }` and marks the entry for removal. After iteration, removes all expired entries. If `timers` vec is now empty, removes the `EffectTimers` component from the entity.

#### Combo Streak Tracker (1 system, likely in `src/effect/conditions/`)

35. **`track_combo_streak`**: Game system. Reads `BumpPerformed` and `BumpWhiffed` messages. Also reads `BoltImpactBreaker` where `bump_status == BumpStatus::Inactive` for NoBump resets.
    - On `BumpPerformed` with `BumpGrade::Perfect`: increment `ComboStreak.count`.
    - On `BumpPerformed` with `BumpGrade::Early` or `BumpGrade::Late`: reset `ComboStreak.count = 0`.
    - On `BumpWhiffed`: reset `ComboStreak.count = 0`.
    - On `BoltImpactBreaker` with `BumpStatus::Inactive`: reset `ComboStreak.count = 0`.
    - Does NOT persist across runs. Resource is `Default` initialized.
    - DOES persist across nodes within a run.

#### Spawn Stamp Registry Watcher (1 system, likely in `src/effect/storage/`)

36. **`watch_spawn_registry`**: Game system. Queries for entities with `Added<Bolt>`, `Added<Cell>`, `Added<Wall>`, `Added<Breaker>` (newly spawned this frame). Reads `SpawnStampRegistry` resource. For each newly spawned entity, determines its `EntityKind` by checking which marker component it has. For each matching entry `(source, entity_kind, tree)` in the registry where `entity_kind` matches, clones the tree and calls `commands.stamp_effect(entity, source.clone(), tree.clone())`.

---

### Patterns to Follow

- **Bridge pattern**: All bridges follow the same structure — read message, build TriggerContext, determine scope, call `walk_effects`. No game logic, no entity modification, no message sending. See `docs/todos/detail/effect-refactor/creating-triggers/trigger-api/bridge-systems.md`.
- **Walker signature**: `walk_effects(entity, &Trigger, &TriggerContext, &BoundEffects, &StagedEffects, &mut Commands)`. See `docs/todos/detail/effect-refactor/walking-effects/walking-algorithm.md`. The walker is a helper function, not a system. Bridges are regular systems that query entities and call the walker.
- **Message reading**: Use `MessageReader<T>` to read messages. Messages persist for one frame in Bevy 0.18. Death bridges intentionally read from the previous frame.
- **Global walk pattern**: Query `Query<(Entity, &BoundEffects, &StagedEffects)>` and iterate all matches.
- **Local walk pattern**: Use `Query::get(entity)` on specific participant entities. If the entity does not have BoundEffects/StagedEffects, skip it silently (entity may not have any effect trees installed).
- **Naming convention**: Bridge systems use `on_` prefix. Game systems use descriptive verb-noun names. See `docs/todos/detail/effect-refactor/creating-triggers/naming-convention.md`.
- **Death bridge generic pattern**: Implement as a generic function `on_destroyed<T: GameEntity>`, then register four monomorphized instances. The generic system reads `MessageReader<Destroyed<T>>`.

### RON Data
None. No RON data changes needed for trigger bridges. All tunable values are in the Trigger enum parameters (thresholds come from effect tree definitions, not RON config).

### Schedule

#### FixedUpdate — EffectSystems::Bridge

All of the following run in `FixedUpdate` within `EffectSystems::Bridge`, with `run_if(in_state(NodeState::Playing))`:

| System | After | Notes |
|--------|-------|-------|
| `on_bumped` | `BreakerSystems::GradeBump` | Local, walks bolt + breaker |
| `on_perfect_bumped` | `BreakerSystems::GradeBump` | Local, walks bolt + breaker |
| `on_early_bumped` | `BreakerSystems::GradeBump` | Local, walks bolt + breaker |
| `on_late_bumped` | `BreakerSystems::GradeBump` | Local, walks bolt + breaker |
| `on_bump_occurred` | `BreakerSystems::GradeBump` | Global |
| `on_perfect_bump_occurred` | `BreakerSystems::GradeBump` | Global |
| `on_early_bump_occurred` | `BreakerSystems::GradeBump` | Global |
| `on_late_bump_occurred` | `BreakerSystems::GradeBump` | Global |
| `on_bump_whiff_occurred` | `BreakerSystems::GradeBump` | Global, reads BumpWhiffed |
| `on_no_bump_occurred` | `BreakerSystems::GradeBump` AND `BoltSystems::BreakerCollision` | Global, reads BoltImpactBreaker |
| `on_impacted_bolt_cell` | `BoltSystems::CellCollision` | Local |
| `on_impacted_bolt_wall` | `BoltSystems::WallCollision` | Local |
| `on_impacted_bolt_breaker` | `BoltSystems::BreakerCollision` | Local |
| `on_impacted_breaker_cell` | (no specific after) | Local |
| `on_impacted_breaker_wall` | (no specific after) | Local |
| `on_impacted_cell_wall` | (no specific after) | Local |
| `on_impact_occurred_bolt_cell` | `BoltSystems::CellCollision` | Global, 2 sweeps |
| `on_impact_occurred_bolt_wall` | `BoltSystems::WallCollision` | Global, 2 sweeps |
| `on_impact_occurred_bolt_breaker` | `BoltSystems::BreakerCollision` | Global, 2 sweeps |
| `on_impact_occurred_breaker_cell` | (no specific after) | Global, 2 sweeps |
| `on_impact_occurred_breaker_wall` | (no specific after) | Global, 2 sweeps |
| `on_impact_occurred_cell_wall` | (no specific after) | Global, 2 sweeps |
| `on_destroyed::<Cell>` | (previous frame messages) | Mixed scope |
| `on_destroyed::<Bolt>` | (previous frame messages) | Mixed scope |
| `on_destroyed::<Wall>` | (previous frame messages) | Mixed scope |
| `on_destroyed::<Breaker>` | (previous frame messages) | Mixed scope |
| `on_bolt_lost_occurred` | `BoltSystems::BoltLost` | Global |
| `on_node_timer_threshold_occurred` | (none — one-frame delay from Tick) | Global, reads NodeTimerThresholdCrossed from previous frame |
| `on_time_expires` | (none — one-frame delay from Tick) | Self-scoped, reads EffectTimerExpired from previous frame |
| `track_combo_streak` | `BreakerSystems::GradeBump` | Reads BumpPerformed, BumpWhiffed, BoltImpactBreaker |
| `watch_spawn_registry` | (after entity spawning systems) | Reads Added<T> |

#### FixedUpdate — EffectSystems::Tick

| System | After | Notes |
|--------|-------|-------|
| `tick_effect_timers` | (none within set) | Produces EffectTimerExpired messages |
| `check_node_timer_thresholds` | (after node timer tick) | Produces NodeTimerThresholdCrossed messages |

**Important**: `tick_effect_timers` and `check_node_timer_thresholds` live in `EffectSystems::Tick` (which runs AFTER `EffectSystems::Bridge` by set ordering). Their messages are consumed by bridges `on_time_expires` and `on_node_timer_threshold_occurred` in the NEXT frame via standard Bevy message persistence. This one-frame delay is by design. Do NOT add `.after(tick_effect_timers)` or `.after(check_node_timer_thresholds)` to the bridge registrations — the set ordering already enforces Bridge < Tick, and the bridges intentionally read previous-frame messages.

#### OnEnter/OnExit — EffectSystems::Reset

| System | Schedule | Notes |
|--------|----------|-------|
| `on_node_start_occurred` | `OnEnter(NodeState::Playing)` | NOT FixedUpdate |
| `on_node_end_occurred` | `OnExit(NodeState::Playing)` | NOT FixedUpdate |
| `reset_node_timer_thresholds` | `OnEnter(NodeState::Playing)` | Clears fired set |

---

### Resources to Initialize

These resources are initialized by `EffectPlugin::build`:
- `SpawnStampRegistry` — `Default` (empty Vec)
- `ComboStreak` — `Default` (count: 0)
- `NodeTimerThresholdRegistry` — `Default` (empty thresholds Vec, empty fired HashSet)

---

### Components Used (already defined in Wave 2)

- `BoundEffects` — `src/effect/storage/bound_effects.rs`
- `StagedEffects` — `src/effect/storage/staged_effects.rs`
- `EffectTimers` — `src/effect/triggers/time/components.rs`

---

### Messages Used (already defined or to be extended)

**Existing messages consumed by bridges:**
- `BumpPerformed { grade: BumpGrade, bolt: Option<Entity>, breaker: Entity }` — from `breaker/messages.rs`
- `BumpWhiffed` — unit message from `breaker/messages.rs`
- `BoltImpactBreaker { bolt: Entity, breaker: Entity, bump_status: BumpStatus }` — from `bolt/messages.rs` (migrated to include `bump_status`)
- `BoltImpactCell { cell: Entity, bolt: Entity }` — from `bolt/messages.rs`
- `BoltImpactWall { bolt: Entity, wall: Entity }` — from `bolt/messages.rs`
- `BreakerImpactCell { breaker: Entity, cell: Entity }` — from `breaker/messages.rs`
- `BreakerImpactWall { breaker: Entity, wall: Entity }` — from `breaker/messages.rs`
- `CellImpactWall { cell: Entity, wall: Entity }` — from `cells/messages.rs`
- `Destroyed<Cell>`, `Destroyed<Bolt>`, `Destroyed<Wall>`, `Destroyed<Breaker>` — from shared death pipeline
- `BoltLost { bolt: Entity, breaker: Entity }` — from `bolt/messages.rs` (migrated from unit struct)

**New messages produced by game systems (defined in effect/triggers/):**
- `NodeTimerThresholdCrossed { ratio: f32 }` — sent by `check_node_timer_thresholds`, consumed by `on_node_timer_threshold_occurred`
- `EffectTimerExpired { entity: Entity, original_duration: OrderedFloat<f32> }` — sent by `tick_effect_timers`, consumed by `on_time_expires`. Per the authoritative type doc the field is `original_duration`, not `duration`.

---

### Register Functions

Each trigger category has a `register(app: &mut App)` function in its `register.rs` file. The writer-code must implement these. `EffectPlugin::build` calls each one.

**`triggers::bump::register(app)`** — registers all 10 bump bridges:
```
// 9 bump bridges that only need .after(BreakerSystems::GradeBump)
app.add_systems(FixedUpdate, (
    on_bumped,
    on_perfect_bumped,
    on_early_bumped,
    on_late_bumped,
    on_bump_occurred,
    on_perfect_bump_occurred,
    on_early_bump_occurred,
    on_late_bump_occurred,
    on_bump_whiff_occurred,
)
    .in_set(EffectSystems::Bridge)
    .after(BreakerSystems::GradeBump)
    .run_if(in_state(NodeState::Playing))
);

// on_no_bump_occurred needs BOTH .after constraints because it reads
// BoltImpactBreaker (produced by BoltSystems::BreakerCollision) and
// filters on BumpStatus (set by BreakerSystems::GradeBump)
app.add_systems(FixedUpdate,
    on_no_bump_occurred
        .in_set(EffectSystems::Bridge)
        .after(BreakerSystems::GradeBump)
        .after(BoltSystems::BreakerCollision)
        .run_if(in_state(NodeState::Playing))
);
```

**`triggers::impact::register(app)`** — registers all 12 impact bridges:
```
app.add_systems(FixedUpdate, (
    on_impacted_bolt_cell,
    on_impacted_bolt_wall,
    on_impacted_bolt_breaker,
    on_impacted_breaker_cell,
    on_impacted_breaker_wall,
    on_impacted_cell_wall,
    on_impact_occurred_bolt_cell,
    on_impact_occurred_bolt_wall,
    on_impact_occurred_bolt_breaker,
    on_impact_occurred_breaker_cell,
    on_impact_occurred_breaker_wall,
    on_impact_occurred_cell_wall,
)
    .in_set(EffectSystems::Bridge)
    .run_if(in_state(NodeState::Playing))
);
// Per-system after constraints for bolt collision bridges
```

**`triggers::death::register(app)`** — registers 4 monomorphized death bridges:
```
app.add_systems(FixedUpdate, (
    on_destroyed::<Cell>,
    on_destroyed::<Bolt>,
    on_destroyed::<Wall>,
    on_destroyed::<Breaker>,
)
    .in_set(EffectSystems::Bridge)
    .run_if(in_state(NodeState::Playing))
);
```

**`triggers::bolt_lost::register(app)`** — registers bolt lost bridge:
```
app.add_systems(FixedUpdate,
    on_bolt_lost_occurred
        .in_set(EffectSystems::Bridge)
        .after(BoltSystems::BoltLost)
        .run_if(in_state(NodeState::Playing))
);
```

**`triggers::node::register(app)`** — registers node bridges, game system, reset, resource, message:
```
// Bridges
app.add_systems(OnEnter(NodeState::Playing), on_node_start_occurred);
app.add_systems(OnExit(NodeState::Playing), on_node_end_occurred);
// No .after(check_node_timer_thresholds) needed. Bridge runs in EffectSystems::Bridge
// which is ordered before EffectSystems::Tick by set ordering. The one-frame delay
// (Tick sends NodeTimerThresholdCrossed this frame, Bridge reads it next frame) is intentional.
app.add_systems(FixedUpdate,
    on_node_timer_threshold_occurred
        .in_set(EffectSystems::Bridge)
        .run_if(in_state(NodeState::Playing))
);

// Game system
app.add_systems(FixedUpdate,
    check_node_timer_thresholds
        .in_set(EffectSystems::Tick)
        .run_if(in_state(NodeState::Playing))
);

// Reset
app.add_systems(OnEnter(NodeState::Playing), reset_node_timer_thresholds);

// Resource
app.init_resource::<NodeTimerThresholdRegistry>();

// Message
app.add_message::<NodeTimerThresholdCrossed>();
```

**`triggers::time::register(app)`** — registers time bridge, game system, component, message:
```
// Bridge — no .after(tick_effect_timers) needed. Bridge runs in EffectSystems::Bridge
// which is ordered before EffectSystems::Tick by set ordering. The one-frame delay
// (Tick sends EffectTimerExpired this frame, Bridge reads it next frame) is intentional.
app.add_systems(FixedUpdate,
    on_time_expires
        .in_set(EffectSystems::Bridge)
        .run_if(in_state(NodeState::Playing))
);

// Game system
app.add_systems(FixedUpdate,
    tick_effect_timers
        .in_set(EffectSystems::Tick)
        .run_if(in_state(NodeState::Playing))
);

// Message
app.add_message::<EffectTimerExpired>();
```

**Additional registrations in EffectPlugin::build (not in trigger register functions):**
```
// Combo streak tracker
app.add_systems(FixedUpdate,
    track_combo_streak
        .in_set(EffectSystems::Bridge)
        .after(BreakerSystems::GradeBump)
        .run_if(in_state(NodeState::Playing))
);

// Spawn stamp watcher
app.add_systems(FixedUpdate,
    watch_spawn_registry
        .in_set(EffectSystems::Bridge)
        .run_if(in_state(NodeState::Playing))
);

// Resources
app.init_resource::<SpawnStampRegistry>();
app.init_resource::<ComboStreak>();
```

---

### Wiring Requirements

The writer-code must update the following wiring files:

1. **`src/effect/triggers/bump/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register;` and re-exports.
2. **`src/effect/triggers/bump/register.rs`**: `pub(crate) fn register(app: &mut App)` that registers all 10 bump bridges.
3. **`src/effect/triggers/impact/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register;` and re-exports.
4. **`src/effect/triggers/impact/register.rs`**: `pub(crate) fn register(app: &mut App)` that registers all 12 impact bridges.
5. **`src/effect/triggers/death/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register;` and re-exports.
6. **`src/effect/triggers/death/register.rs`**: `pub(crate) fn register(app: &mut App)` that registers all 4 death bridges.
7. **`src/effect/triggers/bolt_lost/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register;` and re-exports.
8. **`src/effect/triggers/bolt_lost/register.rs`**: `pub(crate) fn register(app: &mut App)`.
9. **`src/effect/triggers/node/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register; pub(crate) mod check_thresholds; pub(crate) mod resources; pub(crate) mod messages;` and re-exports.
10. **`src/effect/triggers/node/register.rs`**: `pub(crate) fn register(app: &mut App)` that registers node bridges, check_thresholds, reset, resource, message.
11. **`src/effect/triggers/time/mod.rs`**: `pub(crate) mod bridges; pub(crate) mod register; pub(crate) mod tick_timers; pub(crate) mod components; pub(crate) mod messages;` and re-exports.
12. **`src/effect/triggers/time/register.rs`**: `pub(crate) fn register(app: &mut App)` that registers time bridge, tick_timers, component, message.
13. **`src/effect/triggers/mod.rs`**: `pub(crate) mod bump; pub(crate) mod impact; pub(crate) mod death; pub(crate) mod bolt_lost; pub(crate) mod node; pub(crate) mod time;` and re-exports.
14. **`src/effect/plugin.rs`**: Ensure `EffectPlugin::build` calls all trigger `register` functions, registers `track_combo_streak`, `watch_spawn_registry`, and initializes `SpawnStampRegistry`, `ComboStreak`.
15. **`src/effect/conditions/mod.rs`**: Add `pub(crate) mod track_combo_streak;` (new system file for combo tracking).
16. **`src/effect/storage/mod.rs`**: Add `pub(crate) mod watch_spawn_registry;` (new system file for spawn watching).

Note: Trigger module files should already have stubs from Wave 2. The conditions and storage modules may need new file creation if Wave 2 only stubbed empty directories.

---

### Prerequisites (Wave 2 Scaffold Must Provide)

Wave 5 depends on the following types being present as at least stubs from wave 2. If any are missing or have wrong signatures, the writer-code must flag it — NOT create them in wave 5.

**Message types with specific field requirements:**
- `BoltImpactBreaker { bolt: Entity, breaker: Entity, bump_status: BumpStatus }` — must include `bump_status: BumpStatus` field. `on_no_bump_occurred` and `track_combo_streak` filter on this.
- `BoltLost { bolt: Entity, breaker: Entity }` — must carry both `bolt` and `breaker` entity fields (migrated from unit struct). `on_bolt_lost_occurred` reads both.
- `EffectTimerExpired { entity: Entity, original_duration: OrderedFloat<f32> }` — must include `original_duration` field per authoritative type doc.

**Death pipeline types (in `src/shared/`):**
- `GameEntity` trait — implemented on `Bolt`, `Cell`, `Wall`, `Breaker`
- `Destroyed<T: GameEntity> { victim: Entity, killer: Option<Entity>, victim_pos: Vec2, killer_pos: Option<Vec2> }` message — generic death message read by death bridges
- `Hp`, `KilledBy`, `Dead`, `DamageDealt`, `KillYourself`, `DespawnEntity` — other death pipeline types (not directly consumed by wave 5 bridges but must exist for compilation)

**Effect storage types:**
- `BoundEffects` component — `src/effect/storage/bound_effects.rs`
- `StagedEffects` component — `src/effect/storage/staged_effects.rs`
- `EffectTimers` component — `src/effect/triggers/time/components.rs`

**Trigger and context types:**
- `Trigger` enum with all variants listed in this spec
- `TriggerContext` enum with `Bump`, `Impact`, `Death`, `BoltLost`, `None` variants
- `EntityKind` enum with `Cell`, `Bolt`, `Wall`, `Breaker`, `Any` variants

**Entity marker components:**
- `Bolt`, `Cell`, `Wall`, `Breaker` marker components (for `Added<T>` queries in `watch_spawn_registry` and entity kind classification in death bridges)

**Cross-domain resources:**
- `NodeTimer { remaining: f32, total: f32 }` — from `crate::state::run::node::resources::NodeTimer`. Read by `check_node_timer_thresholds` to compute timer ratio.

**Walker function:**
- `walk_effects(entity, &Trigger, &TriggerContext, &BoundEffects, &StagedEffects, &mut Commands)` — implemented in wave 4, called by all bridges

---

### Constraints

- **Do NOT modify**: Test files. The writer-code must not change any test the test-writer produced.
- **Do NOT modify**: `src/effect/types/` — all type definitions were created in Wave 2.
- **Do NOT modify**: `src/effect/walking/` — the walking algorithm was implemented in Wave 4.
- **Do NOT modify**: `src/effect/commands/` — command extensions were implemented in Wave 4.
- **Do NOT modify**: `src/effect/dispatch/` — fire/reverse dispatch was implemented in Wave 4.
- **Do NOT modify**: `src/effect/effects/` — effect implementations are Wave 6, not this wave.
- **Do NOT modify**: `src/effect/conditions/evaluate_conditions.rs` — condition evaluation is a separate system, not part of this wave (except `track_combo_streak` which updates `ComboStreak`).
- **Do NOT modify**: `src/bolt/`, `src/breaker/`, `src/cells/`, `src/shared/` — domain code is off-limits. Bridges read messages from other domains but never modify them.
- **Do NOT add**: Effect firing or reversing logic. Bridges call `walk_effects`, which calls command extensions. The walker handles all tree evaluation.
- **Do NOT add**: Game logic in bridges. No damage calculation, no entity modification, no state changes. Bridges are pure translators.
- **Do NOT add**: Tick systems for spawned effect entities (shockwave, chain_lightning, etc.). Those are in EffectSystems::Tick and belong to Wave 6.

---

### Implementation Notes

1. **Death bridge generic approach**: The recommended pattern is a single generic function `fn on_destroyed<T: GameEntity>(...)` with the generic parameter determining which `Destroyed<T>` to read and which `EntityKind` to use for `Killed` and `DeathOccurred`. However, the function needs to know the `EntityKind` for type T. Two approaches:
   - Add `fn entity_kind() -> EntityKind` to the `GameEntity` trait. This is clean but touches `src/shared/`.
   - Use a `classify_entity(entity, world)` helper that checks marker components. This is self-contained.
   - Prefer the approach that is already stubbed from Wave 2.

2. **`on_time_expires` reads duration from message**: The `EffectTimerExpired` message includes `original_duration: OrderedFloat<f32>` so the bridge can construct the correct `Trigger::TimeExpires(original_duration)`. The bridge reads the duration from the message, not from the entity's `EffectTimers` component (which has already had the expired entry removed by `tick_effect_timers`).

3. **Bolt field in BumpPerformed is Option<Entity>**: When `msg.bolt` is `None`, the Local bump bridges skip the bolt walk entirely but still walk the breaker. This is explicitly documented in the design docs.

4. **Impact bridges fire two triggers per collision**: Local bridges walk impactor with `Impacted(impactee_kind)` and impactee with `Impacted(impactor_kind)`. Global bridges do two full sweeps, one with `ImpactOccurred(impactee_kind)` and one with `ImpactOccurred(impactor_kind)`.

5. **Entity safety**: Entities are never despawned during FixedUpdate. The death pipeline defers despawn to PostFixedUpdate via `process_despawn_requests`. The walker and bridges can safely iterate all entries without checking entity validity mid-walk.

6. **All bridges within EffectSystems::Bridge can run in parallel**: They read different messages and write only through deferred commands. No shared mutable state.

7. **ComboStreak updates from multiple message sources**: `track_combo_streak` must read three separate message types: `BumpPerformed`, `BumpWhiffed`, and `BoltImpactBreaker` (for NoBump detection). This requires three `MessageReader` parameters.

8. **watch_spawn_registry uses Added<T> filter**: This is a Bevy change detection filter that matches entities that received the component this frame. The system must check all four entity types (Bolt, Cell, Wall, Breaker) each frame.
