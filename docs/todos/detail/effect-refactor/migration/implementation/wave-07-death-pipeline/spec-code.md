## Implementation Spec: Shared/Cells/Bolt/Breaker/Walls — Death Pipeline Systems

### Domains
- `src/shared/` — `apply_damage::<T>`, `process_despawn_requests`, `DamageTargetData`, `DeathDetectionData`
- `src/cells/` — `detect_cell_deaths`
- `src/bolt/` — `detect_bolt_deaths`
- `src/walls/` — `detect_wall_deaths`
- `src/breaker/` — `detect_breaker_deaths`

### Failing Tests
- `src/shared/systems/apply_damage.rs` — tests for `apply_damage::<T>` (generic, monomorphized per entity type)
- `src/shared/systems/process_despawn_requests.rs` — tests for `process_despawn_requests`
- `src/cells/systems/detect_cell_deaths.rs` — tests for `detect_cell_deaths`
- `src/bolt/systems/detect_bolt_deaths.rs` — tests for `detect_bolt_deaths`
- `src/walls/systems/detect_wall_deaths.rs` — tests for `detect_wall_deaths`
- `src/breaker/systems/detect_breaker_deaths.rs` — tests for `detect_breaker_deaths`

Test counts are determined by the test spec; the writer-code must make all tests pass without modifying any test code.

---

### Prerequisites

This wave assumes the following are already complete before writer-code runs:

| Prerequisite | Wave | What must exist |
|-------------|------|-----------------|
| Wave 2 scaffold | 2 | All death pipeline types stubbed: `GameEntity` trait, `Hp`, `KilledBy`, `Dead`, `DamageDealt<T>`, `KillYourself<T>`, `DespawnEntity`, `Destroyed<T>`. All system stubs with empty bodies. `DeathPipelinePlugin` registered in `game.rs`. `EffectV3Systems::Tick` system set defined in `src/effect_v3/sets.rs`. |
| Wave 2 scaffold | 2 | `src/shared/systems/mod.rs` exists (created in wave 2 scaffold). `pub(crate) mod systems;` declared in `src/shared/mod.rs`. |
| Wave 2 scaffold | 2 | `src/shared/queries.rs` exists with `DamageTargetData` and `DeathDetectionData` stubs. |
| Wave 2 scaffold | 2 | `src/shared/sets.rs` exists with `DeathPipelineSystems` enum. |
| Wave 2 scaffold | 2 | `src/shared/components/` directory module exists with `hp.rs`, `killed_by.rs`, `dead.rs`. |

**Critical**: `EffectV3Systems::Tick` must already be defined and configured as a system set in the effect domain (wave 2 scaffold). `DeathPipelineSystems::ApplyDamage` orders `.after(EffectV3Systems::Tick)`, so the set must exist at plugin build time or Bevy will panic.

---

### What to Implement

#### 1. `apply_damage::<T>` (generic system)

**File**: `src/shared/systems/apply_damage.rs`

A single generic system function monomorphized per `GameEntity` type. The plugin registers four instances: `apply_damage::<Cell>`, `apply_damage::<Bolt>`, `apply_damage::<Wall>`, `apply_damage::<Breaker>`.

**Signature**:
```rust
pub fn apply_damage<T: GameEntity>(
    mut targets: Query<DamageTargetData, (With<T>, Without<Dead>)>,
    mut messages: MessageReader<DamageDealt<T>>,
)
```

For `apply_damage::<Cell>` specifically, the query also includes `Without<Locked>`:
```rust
// Cell specialization has an additional filter — see "Cell Specialization" below
```

**Behavior**:
1. Iterate over all `DamageDealt<T>` messages via `messages.read()`.
2. For each message, look up `message.target` in the `targets` query. If the entity is not found (already dead, missing Hp, or filtered out by `Without<Dead>`/`Without<Locked>`), skip silently.
3. Decrement `hp.current` by `message.amount`.
4. **Killing blow detection**: If `hp.current` was positive before this message and is now `<= 0.0`, AND `killed_by.dealer` is `None` (not already set), set `killed_by.dealer = message.dealer`. First kill wins — if `KilledBy.dealer` is already `Some(_)`, do NOT overwrite.
5. Do NOT despawn. Do NOT send `KillYourself`. Do NOT update visuals.

**Cell Specialization — `Without<Locked>` Filter**:

The generic system signature uses `(With<T>, Without<Dead>)`. For the Cell monomorphization, the filter must also include `Without<Locked>`. There are two approaches:

**Approach A (Preferred)**: Write `apply_damage` as a generic system with `(With<T>, Without<Dead>)` filters. Then write a separate `apply_damage_cell` wrapper that adds the `Without<Locked>` filter. Register the wrapper for Cell and the generic for other types.

**Approach B**: Make the system fully generic with a second trait bound or a custom filter trait that yields `Without<Locked>` for Cell and an always-true filter for others.

The writer-code should use **Approach A** — a thin wrapper is simpler than a trait-based filter abstraction.

Duplicate the ~10-line body in both `apply_damage_cell` and the generic `apply_damage::<T>`. No helper function needed — the body is short enough that duplication is clearer than trying to abstract over different query filters (Bevy's `Query` type with different filter tuples cannot be passed to a shared function without complex trait gymnastics).

```rust
// Public system for Cell — includes Without<Locked>
pub fn apply_damage_cell(
    mut targets: Query<DamageTargetData, (With<Cell>, Without<Dead>, Without<Locked>)>,
    mut messages: MessageReader<DamageDealt<Cell>>,
) {
    // ~10 lines: iterate messages, lookup target, decrement hp, set killed_by on killing blow
}

// Public generic system for Bolt/Wall/Breaker
pub fn apply_damage<T: GameEntity>(
    mut targets: Query<DamageTargetData, (With<T>, Without<Dead>)>,
    mut messages: MessageReader<DamageDealt<T>>,
) {
    // Same ~10-line body as apply_damage_cell
}
```

Both functions contain identical logic. The only difference is the query filter tuple.

---

#### 2. `detect_cell_deaths`

**File**: `src/cells/systems/detect_cell_deaths.rs`

**Signature**:
```rust
pub fn detect_cell_deaths(
    query: Query<DeathDetectionData, (With<Cell>, Without<Dead>)>,
    mut kill_messages: MessageWriter<KillYourself<Cell>>,
)
```

**Behavior**:
1. Iterate all entities matching the query.
2. For each entity where `hp.current <= 0.0`, send `KillYourself::<Cell>` with `victim: entity`, `killer: killed_by.dealer`.
3. Do NOT insert `Dead`. Do NOT despawn.

**RequiredToClear — scope deferral**: The design docs specify that `detect_cell_deaths` should read `Has<RequiredToClear>` for downstream node completion tracking. However, this wave does NOT include `RequiredToClear` in the query. Wave 9 (cell domain migration) adds it when the cell kill handler and node completion tracking are implemented. For this wave, the query is simply `(With<Cell>, Without<Dead>)` with `DeathDetectionData` fields only.

---

#### 3. `detect_bolt_deaths`

**File**: `src/bolt/systems/detect_bolt_deaths.rs`

**Signature**:
```rust
pub fn detect_bolt_deaths(
    query: Query<DeathDetectionData, (With<Bolt>, Without<Dead>)>,
    mut kill_messages: MessageWriter<KillYourself<Bolt>>,
)
```

**Behavior**: Same as `detect_cell_deaths` but for Bolt entities.

---

#### 4. `detect_wall_deaths`

**File**: `src/walls/systems/detect_wall_deaths.rs`

**Signature**:
```rust
pub fn detect_wall_deaths(
    query: Query<DeathDetectionData, (With<Wall>, Without<Dead>)>,
    mut kill_messages: MessageWriter<KillYourself<Wall>>,
)
```

**Behavior**: Same as `detect_cell_deaths` but for Wall entities.

---

#### 5. `detect_breaker_deaths`

**File**: `src/breaker/systems/detect_breaker_deaths.rs`

**Signature**:
```rust
pub fn detect_breaker_deaths(
    query: Query<DeathDetectionData, (With<Breaker>, Without<Dead>)>,
    mut kill_messages: MessageWriter<KillYourself<Breaker>>,
)
```

**Behavior**: Same as `detect_cell_deaths` but for Breaker entities.

---

#### 6. `process_despawn_requests`

**File**: `src/shared/systems/process_despawn_requests.rs`

**Signature**:
```rust
pub fn process_despawn_requests(
    mut commands: Commands,
    mut messages: MessageReader<DespawnEntity>,
)
```

**Behavior**:
1. Iterate all `DespawnEntity` messages via `messages.read()`.
2. For each message, call `commands.entity(msg.entity).try_despawn()`.
3. Use `try_despawn` — NOT `despawn` — because the entity may already have been despawned by another `DespawnEntity` message in the same frame or by another system.

---

#### 7. `DamageTargetData` (QueryData)

**File**: `src/shared/queries.rs`

```rust
#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct DamageTargetData {
    pub hp: &'static mut Hp,
    pub killed_by: &'static mut KilledBy,
}
```

Mutable because `apply_damage` writes to both `Hp` (decrement) and `KilledBy` (set on killing blow). The `With<T>` and `Without<Dead>` / `Without<Locked>` filters are applied at the system level, not in the QueryData.

---

#### 8. `DeathDetectionData` (QueryData)

**File**: `src/shared/queries.rs`

```rust
#[derive(QueryData)]
pub(crate) struct DeathDetectionData {
    pub entity: Entity,
    pub killed_by: &'static KilledBy,
    pub hp: &'static Hp,
}
```

Read-only — detection systems only read `Hp` and `KilledBy`. No `#[query_data(mutable)]` needed.

---

### Prerequisite Types (Must Already Exist Before This Wave)

The following types are prerequisites that must be created in an earlier wave or as shared prerequisites before this wave launches. The writer-code for this wave does NOT create these; it uses them.

| Type | Kind | Location | Description |
|------|------|----------|-------------|
| `GameEntity` | Trait | `src/shared/traits.rs` | Marker trait: `trait GameEntity: Component {}` with impls for `Bolt`, `Cell`, `Wall`, `Breaker` |
| `Hp` | Component | `src/shared/components/hp.rs` | `{ current: f32, starting: f32, max: Option<f32> }` with `#[derive(Component, Debug, Clone)]` |
| `KilledBy` | Component | `src/shared/components/killed_by.rs` | `{ dealer: Option<Entity> }` with `#[derive(Component, Default, Debug)]` |
| `Dead` | Component | `src/shared/components/dead.rs` | Marker: `#[derive(Component)] struct Dead;` |
| `Locked` | Component | Existing in cells domain | Already exists — lock cells use this |
| `DamageDealt<T>` | Message | `src/shared/messages.rs` | `{ dealer: Option<Entity>, target: Entity, amount: f32, source_chip: Option<String>, _marker: PhantomData<T> }` |
| `KillYourself<T>` | Message | `src/shared/messages.rs` | `{ victim: Entity, killer: Option<Entity>, _marker: PhantomData<T> }` |
| `DespawnEntity` | Message | `src/shared/messages.rs` | `{ entity: Entity }` |
| `Bolt` | Component | Existing in bolt domain | Entity marker |
| `Cell` | Component | Existing in cells domain | Entity marker |
| `Wall` | Component | Existing in walls domain | Entity marker |
| `Breaker` | Component | Existing in breaker domain | Entity marker |

If any of these do not exist when the writer-code runs, the build will fail. The orchestrator must ensure they are created first.

---

### Patterns to Follow

- **Generic system pattern**: The `apply_damage::<T>` system is a single Rust function with a generic type parameter `T: GameEntity`. It is registered four times in the plugin, once per entity type. Each registration creates a separate Bevy system with its own query and message reader.
- **Message reading**: Use `messages.read()` to iterate all messages. This is the Bevy 0.18 `MessageReader` API.
- **Message writing**: Use `kill_messages.send(KillYourself { .. })` to send messages. This is the Bevy 0.18 `MessageWriter` API.
- **QueryData usage**: Import `DamageTargetData` and `DeathDetectionData` from `crate::shared::queries` and use them as the query type parameter. The mutable variant is used directly (not the `ReadOnly` variant) in `apply_damage`; the read-only `DeathDetectionData` is used in detect systems.
- **`try_despawn`**: Bevy 0.18 provides `commands.entity(e).try_despawn()` which is a no-op if the entity does not exist. Use this instead of `despawn()` to avoid panics on double-despawn.
- **PhantomData in messages**: When constructing `KillYourself<T>`, use `_marker: PhantomData` (Rust infers the type from the surrounding context).

---

### RON Data

None. No RON data changes are needed for this wave.

---

### Schedule

#### FixedUpdate Systems

| System | SystemSet | Ordering |
|--------|-----------|----------|
| `apply_damage::<Cell>` (or `apply_damage_cell`) | `DeathPipelineSystems::ApplyDamage` | After `EffectV3Systems::Tick` (configured on the set) |
| `apply_damage::<Bolt>` | `DeathPipelineSystems::ApplyDamage` | After `EffectV3Systems::Tick` (configured on the set) |
| `apply_damage::<Wall>` | `DeathPipelineSystems::ApplyDamage` | After `EffectV3Systems::Tick` (configured on the set) |
| `apply_damage::<Breaker>` | `DeathPipelineSystems::ApplyDamage` | After `EffectV3Systems::Tick` (configured on the set) |
| `detect_cell_deaths` | `DeathPipelineSystems::DetectDeaths` | After `DeathPipelineSystems::ApplyDamage` (configured on the set) |
| `detect_bolt_deaths` | `DeathPipelineSystems::DetectDeaths` | After `DeathPipelineSystems::ApplyDamage` (configured on the set) |
| `detect_wall_deaths` | `DeathPipelineSystems::DetectDeaths` | After `DeathPipelineSystems::ApplyDamage` (configured on the set) |
| `detect_breaker_deaths` | `DeathPipelineSystems::DetectDeaths` | After `DeathPipelineSystems::ApplyDamage` (configured on the set) |

#### PostFixedUpdate Systems

| System | Ordering |
|--------|----------|
| `process_despawn_requests` | No ordering constraints within PostFixedUpdate — it runs after all FixedUpdate systems |

#### SystemSet Configuration

```rust
app.configure_sets(FixedUpdate, (
    DeathPipelineSystems::ApplyDamage
        .after(EffectV3Systems::Tick),
    DeathPipelineSystems::DetectDeaths
        .after(DeathPipelineSystems::ApplyDamage),
));
```

#### Within-Set Parallelism

All four `apply_damage::<T>` systems run in parallel within the `ApplyDamage` set — they read different typed messages and query disjoint entity populations.

All four `detect_*_deaths` systems run in parallel within the `DetectDeaths` set — they query disjoint entity populations and write different typed messages.

---

### System Sets (New)

**File**: `src/shared/sets.rs` (or wherever `DeathPipelineSystems` is defined — check if a shared sets module exists)

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum DeathPipelineSystems {
    ApplyDamage,
    DetectDeaths,
}
```

This enum must be `pub` so domain plugins (cells, bolt, walls, breaker) and the death pipeline plugin can reference it for ordering.

---

### Plugin Wiring

**File**: `src/shared/plugin.rs` (or a new `src/shared/death_pipeline_plugin.rs` — see below)

Create `DeathPipelinePlugin`:

```rust
pub struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // System set ordering
        app.configure_sets(FixedUpdate, (
            DeathPipelineSystems::ApplyDamage
                .after(EffectV3Systems::Tick),
            DeathPipelineSystems::DetectDeaths
                .after(DeathPipelineSystems::ApplyDamage),
        ));

        // Apply damage systems (FixedUpdate, in ApplyDamage set)
        app.add_systems(FixedUpdate, (
            apply_damage_cell,  // or apply_damage::<Cell> if using the generic directly
            apply_damage::<Bolt>,
            apply_damage::<Wall>,
            apply_damage::<Breaker>,
        ).in_set(DeathPipelineSystems::ApplyDamage));

        // Detect deaths systems (FixedUpdate, in DetectDeaths set)
        app.add_systems(FixedUpdate, (
            detect_cell_deaths,
            detect_bolt_deaths,
            detect_wall_deaths,
            detect_breaker_deaths,
        ).in_set(DeathPipelineSystems::DetectDeaths));

        // Deferred despawn (PostFixedUpdate)
        app.add_systems(PostFixedUpdate, process_despawn_requests);
    }
}
```

**Registration**: `DeathPipelinePlugin` must be wired in `src/game.rs` via `.add(DeathPipelinePlugin)` in the `PluginGroupBuilder` chain (see Module Wiring below). It should appear after domain plugins that define entity types (bolt, cells, walls, breaker) and after the effect plugin (which defines `EffectV3Systems::Tick`). This should already be stubbed from wave 2 scaffold.

Note: The detect_*_deaths systems are registered by the `DeathPipelinePlugin`, NOT by their respective domain plugins. Even though `detect_cell_deaths` lives in `src/cells/systems/`, it is the death pipeline plugin that registers it. This keeps all death pipeline ordering in one place.

---

### Module Wiring

Each new system file and the shared queries file need `mod` declarations and re-exports.

**If `src/shared/systems/` does not already exist** (i.e., wave 2 scaffold did not create it):
1. Create `src/shared/systems/mod.rs`
2. Add `pub(crate) mod systems;` to `src/shared/mod.rs`

This should already exist from wave 2 scaffold (see Prerequisites), but verify before assuming.

#### `src/shared/systems/mod.rs`
Add:
```rust
pub(crate) mod apply_damage;
pub(crate) mod process_despawn_requests;
```

#### `src/shared/queries.rs`
Add `DamageTargetData` and `DeathDetectionData` definitions. If this file does not exist, create it and add `pub(crate) mod queries;` to `src/shared/mod.rs`.

#### `src/shared/sets.rs`
Add `DeathPipelineSystems` enum. If this file does not exist, create it and add `pub(crate) mod sets;` to `src/shared/mod.rs`. Re-export: `pub(crate) use sets::DeathPipelineSystems;` in `src/shared/mod.rs`.

#### `src/cells/systems/mod.rs`
Add:
```rust
pub(crate) mod detect_cell_deaths;
```

#### `src/bolt/systems/mod.rs`
Add:
```rust
pub(crate) mod detect_bolt_deaths;
```

#### `src/walls/systems/mod.rs`
Add:
```rust
pub(crate) mod detect_wall_deaths;
```

#### `src/breaker/systems/mod.rs`
Add:
```rust
pub(crate) mod detect_breaker_deaths;
```

#### `src/game.rs`
`game.rs` uses a `PluginGroupBuilder` chain (not `app.add_plugins()`). Add `DeathPipelinePlugin` using `.add()` in the builder chain:
```rust
.add(DeathPipelinePlugin)
```
Place it after the shared plugin and domain plugins (bolt, cells, walls, breaker) and after the effect plugin (which defines `EffectV3Systems::Tick`). This should already be stubbed from wave 2 scaffold — verify the stub exists and ensure it is wired correctly.

---

### Constraints

#### Off-Limits — Do NOT Modify
- Any existing domain plugin files (`src/bolt/plugin.rs`, `src/cells/plugin.rs`, `src/walls/plugin.rs`, `src/breaker/plugin.rs`) — the death pipeline plugin handles registration
- Any existing system files — this wave only creates new files
- `src/effect_v3/` — effect domain is entirely out of scope for this wave
- Domain kill handlers — out of scope (they consume `KillYourself<T>` and send `Destroyed<T>`, but that is a later wave)
- Death bridge systems (`on_destroyed::<T>`) — out of scope
- VFX / audio systems — out of scope
- Node completion tracking — out of scope

#### Do NOT Add
- Domain kill handler logic — this wave creates the damage and death detection layer only
- `Destroyed<T>` sending — that is the domain kill handler's job (later wave)
- `Dead` insertion — that is the domain kill handler's job (later wave)
- Visual feedback (damage flash, health bars) — separate concern
- Any `Changed<Hp>` filter on detect systems — detect systems deliberately omit `Changed<Hp>`. `Without<Dead>` is the only filter beyond entity markers (`With<Cell>`, `With<Bolt>`, etc.). The detect systems check `hp.current <= 0.0` on all non-Dead entities each frame. This is intentional: `Changed<Hp>` would miss entities whose Hp was set to zero in a previous frame but whose `Dead` component was not yet inserted by the domain kill handler

#### Scope Clarification
- `apply_damage` decrements Hp and sets KilledBy. That is ALL it does.
- `detect_*_deaths` sends `KillYourself<T>`. That is ALL they do.
- `process_despawn_requests` calls `try_despawn`. That is ALL it does.
- No system in this wave inserts `Dead`, sends `Destroyed<T>`, or removes entities from spatial indices.

---

### Killing Blow Logic — Detailed

The "killing blow" detection in `apply_damage` must be precise:

1. Read `hp.current` before applying damage. Call this `hp_before`.
2. Subtract `message.amount` from `hp.current`.
3. If `hp_before > 0.0` AND `hp.current <= 0.0`, this is a killing blow.
4. If this is a killing blow AND `killed_by.dealer.is_none()`, set `killed_by.dealer = message.dealer`.
5. If `killed_by.dealer` is already `Some(_)`, do NOT overwrite — the first message that killed the entity wins.

This means:
- If entity has 10 Hp and receives two 8-damage messages in the same frame, the first message is the killing blow (10 -> 2, then 2 -> -6). Wait — that's wrong. The first brings it to 2, the second brings it to -6. So the second is the killing blow.
- Actually: first message: hp_before=10, hp_after=2, not a killing blow. Second message: hp_before=2, hp_after=-6, IS a killing blow. `killed_by.dealer` is set from the second message.
- If two messages both cross zero in the same frame (impossible since damage is applied sequentially within `messages.read()`), the first one wins.

The key insight: messages are processed sequentially within a single `apply_damage` system invocation. Hp is decremented message by message. Only the message that actually crosses the zero boundary is the killing blow.

---

### Testing Note

Tests should be written to the files listed in "Failing Tests" above. Each system file contains both the production system and its `#[cfg(test)]` block (unless the test spec directs otherwise, e.g., splitting into a directory module).

Tests need `MinimalPlugins` + the relevant entity marker components + `Hp` + `KilledBy` + message registration. They do NOT need full game plugins — headless ECS is sufficient.
