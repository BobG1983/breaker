# Message, Component, and Resource Patterns

Bevy version: **0.18.1** (from `breaker-game/Cargo.toml`)

---

## 1. Message Definitions

### Registration: `app.add_message::<T>()`

Messages use Bevy 0.18's native `Message` trait, derived with `#[derive(Message, ...)]`. Registration is `app.add_message::<T>()` — no additional trait bound beyond `Message` is required by the call site. The derive handles the trait implementation.

Every domain plugin registers its own messages. Example from `BoltPlugin::build`:

```rust
// breaker-game/src/bolt/plugin.rs
app.add_message::<BoltSpawned>()
   .add_message::<BoltImpactBreaker>()
   .add_message::<BoltImpactCell>()
   .add_message::<BoltLost>()
   .add_message::<BoltImpactWall>()
   .add_message::<RequestBoltDestroyed>()
```

---

### Simple message — zero-field unit struct

```rust
// breaker-game/src/bolt/messages.rs
/// Sent when the bolt falls below the breaker.
/// Consumed by the breaker plugin (applies penalty per breaker type).
#[derive(Message, Clone, Debug)]
pub struct BoltLost;
```

```rust
// breaker-game/src/state/run/node/messages.rs
/// Sent when all target cells in a node have been destroyed.
/// Consumed by the run state machine and UI.
#[derive(Message, Clone, Debug)]
pub struct NodeCleared;
```

Pattern: `#[derive(Message, Clone, Debug)]`, unit struct. `pub` for cross-crate consumption, `pub(crate)` for within-game-crate.

---

### Message with Entity fields

```rust
// breaker-game/src/bolt/messages.rs
/// Sent when the bolt collides with the breaker.
/// Consumed by breaker (`grade_bump`).
#[derive(Message, Clone, Debug)]
pub(crate) struct BoltImpactBreaker {
    /// The bolt entity that hit the breaker.
    pub bolt: Entity,
    /// The breaker entity that was hit.
    pub breaker: Entity,
}
```

```rust
// breaker-game/src/bolt/messages.rs
/// Sent by `bolt_lost` when an extra bolt falls off screen. Entity is still alive.
#[derive(Message, Clone, Debug)]
pub(crate) struct RequestBoltDestroyed {
    /// The bolt entity to be destroyed.
    pub bolt: Entity,
}
```

---

### Message with mixed fields

```rust
// breaker-game/src/cells/messages.rs
/// A "command" message — owned by the receiving domain (cells), written by
/// multiple senders. The `damage` field is pre-calculated by the sender.
#[derive(Message, Clone, Debug)]
pub(crate) struct DamageCell {
    /// The cell entity to damage.
    pub cell: Entity,
    /// Pre-calculated damage amount.
    pub damage: f32,
    /// The chip name that originated this damage, for evolution attribution.
    pub source_chip: Option<String>,
}
```

```rust
// breaker-game/src/cells/messages.rs
/// Sent by `cleanup_cell` after extracting entity data from the still-alive cell.
#[derive(Message, Clone, Debug)]
pub(crate) struct CellDestroyedAt {
    /// Whether this cell counted toward node completion.
    pub was_required_to_clear: bool,
}
```

---

### Message with enum field

```rust
// breaker-game/src/breaker/messages.rs
/// Sent when the breaker performs a bump.
#[derive(Message, Clone, Debug)]
pub struct BumpPerformed {
    /// The timing grade of the bump.
    pub grade: BumpGrade,
    /// The bolt entity involved in this bump, if known.
    pub bolt: Option<Entity>,
    /// The breaker entity that performed the bump.
    pub breaker: Entity,
}
```

```rust
// breaker-game/src/state/run/chip_select/messages.rs
/// Sent when the player selects a chip from the selection screen.
#[derive(Message, Clone, Debug)]
pub struct ChipSelected {
    /// Display name of the selected chip.
    pub name: String,
}
```

---

### Message with timer-control fields

```rust
// breaker-game/src/state/run/node/messages.rs
/// Sent by the breaker behavior system to subtract time from the node timer.
#[derive(Message, Clone, Debug)]
pub struct ApplyTimePenalty {
    /// Seconds to subtract from the node timer.
    pub seconds: f32,
}
```

---

## 2. Component Definitions

### Marker component — no fields

```rust
// breaker-game/src/cells/components/types.rs
/// Marker component identifying a cell entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, CleanupOnExit<NodeState>)]
pub struct Cell;

/// Marker for cells that count toward node completion.
#[derive(Component, Debug)]
pub struct RequiredToClear;

/// Marker component — cell is locked and immune to damage.
/// Removed by `check_lock_release` when all adjacent cells are destroyed.
#[derive(Component, Debug)]
pub(crate) struct Locked;
```

```rust
// breaker-game/src/breaker/components/core.rs
/// Marker component identifying the breaker entity.
#[derive(Component, Debug, Default)]
#[require(Spatial2D, InterpolateTransform2D)]
pub struct Breaker;

/// Marker: the primary breaker (persists across nodes, cleaned up on run end).
#[derive(Component, Debug)]
pub struct PrimaryBreaker;

/// Marker: an extra breaker (cleaned up on node exit).
#[derive(Component, Debug)]
pub struct ExtraBreaker;

/// Marker: breaker entity has been initialized by `init_breaker`.
/// Prevents duplicate chain pushes on node re-entry.
#[derive(Component)]
pub struct BreakerInitialized;
```

The `#[require(...)]` attribute (Bevy 0.18) auto-inserts listed components when the marker is spawned.

---

### Newtype component — single wrapped value

```rust
// breaker-game/src/breaker/components/core.rs
/// Y position of the breaker at rest.
#[derive(Component, Debug)]
pub struct BreakerBaseY(pub f32);

/// Maximum reflection angle from vertical in radians.
#[derive(Component, Debug)]
pub struct BreakerReflectionSpread(pub f32);
```

```rust
// breaker-game/src/breaker/components/bump.rs
/// Perfect bump timing window (seconds, each side of T=0).
#[derive(Component, Debug)]
pub struct BumpPerfectWindow(pub f32);

/// Early bump window (seconds, before perfect zone).
#[derive(Component, Debug)]
pub struct BumpEarlyWindow(pub f32);
```

```rust
// breaker-game/src/cells/components/types.rs
/// Tracks which adjacent cells must be destroyed to unlock this cell.
#[derive(Component, Debug)]
pub(crate) struct LockAdjacents(pub Vec<Entity>);

/// Current angular position of an orbit cell around its parent shield.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct OrbitAngle(pub f32);
```

---

### Component with multiple named fields

```rust
// breaker-game/src/breaker/components/movement.rs
/// The breaker's current tilt angle in radians.
#[derive(Component, Debug, Default)]
pub struct BreakerTilt {
    /// Current tilt angle in radians.
    pub angle: f32,
    /// Start angle for the current ease animation.
    pub ease_start: f32,
    /// Target angle for the current ease animation.
    pub ease_target: f32,
}
```

```rust
// breaker-game/src/cells/components/types.rs
/// Health of a cell — hit points remaining before destruction.
#[derive(Component, Debug, Clone)]
pub(crate) struct CellHealth {
    /// Current hit points.
    pub current: f32,
    /// Maximum hit points (used for visual damage feedback).
    pub max: f32,
}
```

```rust
// breaker-game/src/cells/components/types.rs
/// Configuration for an orbit cell's circular motion.
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct OrbitConfig {
    /// Distance from shield center to orbit cell center.
    pub radius: f32,
    /// Angular speed in radians per second.
    pub speed: f32,
}
```

```rust
// breaker-game/src/breaker/components/bump.rs
/// Tracks the bump state for timing-grade calculations.
#[derive(Component, Debug)]
pub struct BumpState {
    pub active: bool,
    pub timer: f32,
    pub post_hit_timer: f32,
    pub cooldown: f32,
    pub last_hit_bolt: Option<Entity>,
}
```

---

### Component with conditional fields (`#[cfg]`)

```rust
// breaker-game/src/cells/components/types.rs
/// Full width of a cell in world units.
#[derive(Component, Debug)]
pub(crate) struct CellWidth {
    #[cfg(any(test, feature = "dev"))]
    pub value: f32,
}
```

The `value` field exists only in test/dev builds. In release the struct is zero-sized. The constructor `CellWidth::new(value)` accepts the parameter regardless, discarding it in release.

---

### Component with external type field

```rust
// breaker-game/src/breaker/components/movement.rs
/// Easing applied to deceleration based on speed ratio.
#[derive(Component, Debug)]
pub struct DecelEasing {
    pub ease: EaseFunction,
    pub strength: f32,
}
```

```rust
// breaker-game/src/effect/core/types/definitions/enums/types.rs
/// Permanent effect trees on an entity.
#[derive(Component, Debug, Default, Clone)]
pub struct BoundEffects(pub Vec<(String, EffectNode)>);

/// Working set of partially-resolved chains.
#[derive(Component, Debug, Default, Clone)]
pub struct StagedEffects(pub Vec<(String, EffectNode)>);
```

---

### Typical component derives

| Usage | Derives |
|-------|---------|
| Marker | `Component, Debug` |
| Marker with #[require] | `Component, Debug, Default` |
| Newtype (f32) | `Component, Debug` |
| Newtype (Copy) | `Component, Debug, Clone, Copy` |
| Multi-field struct | `Component, Debug` or `Component, Debug, Clone` |
| Default-constructible | `Component, Debug, Default` |
| Hot-reload comparison | `Component, Debug, Clone, PartialEq, Eq` |

`Copy` is added only when all fields are `Copy`. `Clone` is added when the component must be cloned (e.g., stored in `Vec`, used in hot-reload). `Default` is added when required by `#[require]` or when the component is frequently default-constructed.

---

## 3. Per-Run Resources

### `ChipInventory` — held for the full run

```rust
// breaker-game/src/chips/inventory/data.rs
/// Tracks chips the player has acquired during a run.
#[derive(Resource, Debug, Default)]
pub struct ChipInventory {
    held: HashMap<String, ChipEntry>,
    decay_weights: HashMap<String, f32>,
    template_taken: HashMap<String, u32>,
    template_maxes: HashMap<String, u32>,
}
```

Inserted via `init_resource::<ChipInventory>()` in one of the chip-domain plugins (not the run plugin — it is a dependency injected via the test in `RunPlugin::build`'s test). Cleared — not removed — between runs by `reset_run_state`:

```rust
// breaker-game/src/state/run/loading/systems/reset_run_state.rs
pub(crate) fn reset_run_state(
    mut chip_inventory: ResMut<ChipInventory>,
    // ...
) {
    chip_inventory.clear();
    // ...
}
```

`clear()` wipes `held`, `decay_weights`, `template_taken`, `template_maxes` in place — the `Resource` itself is never removed from the world.

---

### `NodeOutcome` — per-run result tracking

```rust
// breaker-game/src/state/run/resources/definitions.rs
#[derive(Resource, Debug, Clone, Default)]
pub struct NodeOutcome {
    /// Zero-indexed node within the current run.
    pub node_index: u32,
    /// How the current node ended.
    pub result: NodeResult,
    /// Set to `true` when `handle_node_cleared` queues a state transition this frame.
    pub transition_queued: bool,
}
```

Inserted via `init_resource::<NodeOutcome>()` in `RunPlugin`. Reset to `NodeOutcome::default()` at run start by `reset_run_state`. Never removed.

---

### `RunStats` — cumulative run statistics

```rust
// breaker-game/src/state/run/resources/definitions.rs
#[derive(Resource, Debug, Clone, Default)]
pub struct RunStats {
    pub nodes_cleared: u32,
    pub cells_destroyed: u32,
    pub bumps_performed: u32,
    pub perfect_bumps: u32,
    pub bolts_lost: u32,
    pub chips_collected: Vec<String>,
    pub evolutions_performed: u32,
    pub time_elapsed: f32,
    pub seed: u64,
    pub highlights: Vec<RunHighlight>,
}
```

Inserted via `init_resource::<RunStats>()` in `RunPlugin`. Reset to `RunStats::default()` at run start by `reset_run_state`. Never removed.

---

### `NodeTimer` — countdown for the current node

```rust
// breaker-game/src/state/run/node/resources/definitions.rs
#[derive(Resource, Debug, Clone, Default)]
pub struct NodeTimer {
    /// Seconds remaining.
    pub remaining: f32,
    /// Total seconds for this node (used for ratio calculations).
    pub total: f32,
}
```

Inserted via `init_resource::<NodeTimer>()` in `NodePlugin`. Reset each node entry by `init_node_timer` (runs `OnEnter(NodeState::Loading)`). Never removed.

---

### `ClearRemainingCount` — tracks required cells

```rust
// breaker-game/src/state/run/node/resources/definitions.rs
#[derive(Resource, Debug, Default)]
pub struct ClearRemainingCount {
    /// Number of `RequiredToClear` cells still alive.
    pub remaining: u32,
}
```

Inserted via `init_resource::<ClearRemainingCount>()` in `NodePlugin`. Reset each node by `init_clear_remaining` (`OnEnter(NodeState::Loading)`). Never removed.

---

### Per-run cleanup pattern

Resources are **never removed** between runs. The pattern is:

1. `init_resource::<T>()` in the domain plugin's `build` — one-time insertion
2. `reset_run_state` system runs `OnExit(MenuState::Main)` — resets all per-run resources to `Default` or calls a `.clear()` method on them

```rust
// breaker-game/src/state/run/plugin.rs (OnExit(MenuState::Main) systems)
.add_systems(
    OnExit(MenuState::Main),
    (
        reset_run_state,
        generate_node_sequence_system.after(reset_run_state),
    ),
)
```

Entity cleanup (cells, walls, bolts, HUD) uses `CleanupOnExit<S>` from `rantzsoft_stateflow`:

```rust
// rantzsoft_stateflow/src/cleanup.rs
#[derive(Component)]
pub struct CleanupOnExit<S: States> {
    _marker: PhantomData<S>,
}

pub fn cleanup_on_exit<S: States>(
    mut commands: Commands,
    query: Query<Entity, With<CleanupOnExit<S>>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
```

Example: `Cell` has `#[require(CleanupOnExit<NodeState>)]`, so every cell entity is automatically marked for despawn when `cleanup_on_exit::<NodeState>` runs (wired to `OnEnter(NodeState::Teardown)`).

---

## 4. Enum Patterns

### Simple C-style enum

```rust
// breaker-game/src/breaker/messages.rs
/// Grade of a bump timing relative to bolt contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpGrade {
    /// Bump pressed before the perfect zone.
    Early,
    /// Bump timed within the perfect window.
    Perfect,
    /// Bump pressed after the bolt hit.
    Late,
}
```

Pattern: `Debug, Clone, Copy, PartialEq, Eq`. Used as a field in `BumpPerformed`. No `Deserialize` — pure runtime type.

---

### Enum with default variant

```rust
// breaker-game/src/state/run/resources/definitions.rs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NodeResult {
    #[default]
    InProgress,
    Won,
    TimerExpired,
    LivesDepleted,
    Quit,
}
```

Pattern: `Debug, Clone, Copy, Default, PartialEq, Eq`. `Default` is derived on the enum itself, with `#[default]` marking the default variant.

---

### RON-deserialized enum (no data)

```rust
// breaker-game/src/state/run/node/definition/types.rs
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum NodePool {
    #[default]
    Passive,
    Active,
    Boss,
}
```

Pattern: `Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default`. Used as a field in `NodeLayout` which is `#[serde(default)]`.

---

### RON-deserialized enum with tuple variants

```rust
// breaker-game/src/state/run/definition/types.rs
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TierNodeCount {
    Fixed(u32),
    Range(u32, u32),
}
```

---

### Enum with struct variants and associated data (large example)

```rust
// breaker-game/src/effect/core/types/definitions/enums/types.rs
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectKind {
    Shockwave {
        base_range: f32,
        range_per_level: f32,
        stacks: u32,
        speed: f32,
    },
    SpeedBoost {
        multiplier: f32,
    },
    DamageBoost(f32),
    Piercing(u32),
    SpawnBolts {
        #[serde(default = "one")]
        count: u32,
        #[serde(default)]
        lifespan: Option<f32>,
        #[serde(default)]
        inherit: bool,
    },
    // ... 20+ more variants
}
```

Pattern: `Clone, Debug, PartialEq, Deserialize`. Mix of unit, tuple, and struct variants. `#[serde(default)]` and `#[serde(default = "fn")]` for optional RON fields.

---

### Enum used as component discriminant

```rust
// breaker-game/src/effect/core/types/definitions/enums/types.rs
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectNode {
    When {
        trigger: Trigger,
        then: Vec<Self>,
    },
    Do(EffectKind),
    Once(Vec<Self>),
    On {
        target: Target,
        #[serde(default)]
        permanent: bool,
        then: Vec<Self>,
    },
    Until {
        trigger: Trigger,
        then: Vec<Self>,
    },
    Reverse {
        effects: Vec<EffectKind>,
        chains: Vec<Self>,
    },
}
```

This recursive enum is stored inside `BoundEffects` and `StagedEffects` components as `Vec<(String, EffectNode)>`.

---

### Enum with `Hash` for use as map key

```rust
// breaker-game/src/state/run/resources/definitions.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HighlightCategory {
    Execution,
    Endurance,
    Progression,
    Clutch,
}
```

`Hash` is added when the enum is used as a `HashMap` key. `NodePool` also has `Hash` for `HashMap<NodePool, Vec<String>>` in `NodeLayoutRegistry`.

---

### Enum with methods

```rust
// breaker-game/src/state/run/resources/definitions.rs
impl HighlightKind {
    #[must_use]
    pub const fn category(&self) -> HighlightCategory {
        match self {
            Self::MassDestruction | Self::ComboKing | ... => HighlightCategory::Execution,
            Self::NoDamageNode | ... => HighlightCategory::Endurance,
            // ...
        }
    }
}
```

`const fn` method on an enum via exhaustive `match`. `#[must_use]` is required by project lints.

---

## 5. State Types

State enums use Bevy 0.18's `States` and `SubStates` derives:

```rust
// breaker-game/src/state/types/app_state.rs
#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AppState {
    #[default]
    Loading,
    Game,
    Teardown,
}
```

```rust
// breaker-game/src/state/types/node_state.rs
#[derive(SubStates, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[source(RunState = RunState::Node)]
pub enum NodeState {
    #[default]
    Loading,
    AnimateIn,
    Playing,
    AnimateOut,
    Teardown,
}
```

State hierarchy (each is a sub-state of the one above):
- `AppState` (root, `States`)
- `GameState` (sub of `AppState::Game`)
- `RunState` (sub of `GameState::Run`)
- `NodeState` (sub of `RunState::Node`)
- `ChipSelectState` (sub of `RunState::ChipSelect`)
- `MenuState` (sub of `GameState::Menu`)
- `RunEndState` (sub of `RunState::RunEnd`)

---

## Key Files

- `breaker-game/src/bolt/messages.rs` — canonical message file showing unit, entity, and two-entity patterns
- `breaker-game/src/cells/messages.rs` — `DamageCell` (multi-field command message), `CellDestroyedAt`
- `breaker-game/src/breaker/messages.rs` — `BumpGrade` enum, `BumpPerformed` (enum field message)
- `breaker-game/src/cells/components/types.rs` — full spread of component patterns including `#[require]`, `#[cfg]` fields, newtypes, multi-field
- `breaker-game/src/breaker/components/core.rs` — marker components with `#[require]`
- `breaker-game/src/breaker/components/bump.rs` — multi-field component with `Option<Entity>`
- `breaker-game/src/chips/inventory/data.rs` — `ChipInventory` resource, `ChipEntry` helper struct
- `breaker-game/src/state/run/resources/definitions.rs` — `NodeOutcome`, `NodeResult`, `RunStats`, `HighlightKind`, `HighlightCategory`
- `breaker-game/src/state/run/node/resources/definitions.rs` — `NodeTimer`, `ClearRemainingCount`, `NodeLayoutRegistry`
- `breaker-game/src/state/run/loading/systems/reset_run_state.rs` — per-run reset pattern
- `rantzsoft_stateflow/src/cleanup.rs` — `CleanupOnExit<S>` entity cleanup component
- `breaker-game/src/effect/core/types/definitions/enums/types.rs` — large enum with struct/tuple/unit variants, `BoundEffects`, `StagedEffects` components
- `breaker-game/src/state/types/node_state.rs` — `SubStates` derive pattern
