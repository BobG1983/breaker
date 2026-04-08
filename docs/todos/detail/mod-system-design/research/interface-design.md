# Protocol & Hazard Interface Design

Revised after architecture review and Rust idiom review. See `interface-design-draft.md` for the unrevised draft.

## Design Principles

1. **Match existing patterns exactly** — no new idioms (no traits, no observers, no trait objects)
2. **Tuning enum IS the kind discriminant** — no redundant `kind` field + separate tuning
3. **Per-item config resources** — tuning extracted at activation time into a typed config resource; systems read `Res<Config>` with zero enum matching
4. **`run_if` guards for activation** — no runtime checks inside system bodies
5. **Effect-tree protocols dispatch like chips** — shared effect dispatch helper extracted from `dispatch_chip_effects`
6. **One module per protocol/hazard** — `register(app)` delegation pattern from `effect/`
7. **Message-driven cross-domain communication** — hazards NEVER directly mutate resources owned by other domains

---

## 1. Core Enums

All definition-related types (`*Kind`, `*Tuning`, `*Definition`) live in `definition.rs` per the canonical layout (no `types.rs`).

### `ProtocolKind`

```rust
// protocol/definition.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ProtocolKind {
    Deadline,
    RicochetProtocol,
    Anchor,
    Kickstart,
    TierRegression,
    DebtCollector,
    IronCurtain,
    EchoStrike,
    Siphon,
    Greed,
    RecklessDash,
    Burnout,
    Conductor,
    Afterimage,
    Fission,
}

impl ProtocolKind {
    /// All protocol kinds. Manually maintained — update when adding a variant.
    pub const ALL: &[Self] = &[
        Self::Deadline, Self::RicochetProtocol, Self::Anchor, Self::Kickstart,
        Self::TierRegression, Self::DebtCollector, Self::IronCurtain, Self::EchoStrike,
        Self::Siphon, Self::Greed, Self::RecklessDash, Self::Burnout,
        Self::Conductor, Self::Afterimage, Self::Fission,
    ];
}
```

### `HazardKind`

```rust
// hazard/definition.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum HazardKind {
    Decay, Drift, Haste, EchoCells, Erosion, Cascade, Fracture, Renewal,
    Diffusion, Tether, Volatility, GravitySurge, Overcharge, Resonance,
    Momentum, Sympathy,
}

impl HazardKind {
    pub const ALL: &[Self] = &[
        Self::Decay, Self::Drift, Self::Haste, Self::EchoCells, Self::Erosion,
        Self::Cascade, Self::Fracture, Self::Renewal, Self::Diffusion, Self::Tether,
        Self::Volatility, Self::GravitySurge, Self::Overcharge, Self::Resonance,
        Self::Momentum, Self::Sympathy,
    ];
}
```

---

## 2. Tuning Enums

### `ProtocolTuning`

```rust
// protocol/definition.rs
#[derive(Clone, Debug, Deserialize)]
pub enum ProtocolTuning {
    // ── Effect-tree protocols ──────────────────────────────────────
    Deadline { effects: Vec<RootEffect> },
    RicochetProtocol { effects: Vec<RootEffect> },
    Anchor { effects: Vec<RootEffect> },
    Kickstart { effects: Vec<RootEffect> },

    // ── Custom-system protocols ────────────────────────────────────
    TierRegression { tiers_back: u32 },
    DebtCollector { stack_per_bump: f32 },
    IronCurtain { damage_fraction: f32, falloff_start: f32 },
    EchoStrike { max_echoes: u32, newest_fraction: f32, middle_fraction: f32, oldest_fraction: f32 },
    Siphon { streak_window: f32, time_per_kill: f32 },
    Greed { rarity_boost_per_skip: f32 },
    RecklessDash { risky_zone_start: f32, damage_multiplier: f32, double_penalty: bool },
    Burnout { fill_duration: f32, drain_duration: f32, still_threshold: f32, full_heat_damage_multiplier: f32, speed_boost_duration: f32 },
    Conductor { primary_swap_window: f32 },
    Afterimage { phantom_duration: f32, phantom_bolt_duration: f32 },
    Fission { kills_per_split: u32 },
}

impl ProtocolTuning {
    pub const fn kind(&self) -> ProtocolKind {
        match self {
            Self::Deadline { .. } => ProtocolKind::Deadline,
            Self::RicochetProtocol { .. } => ProtocolKind::RicochetProtocol,
            Self::Anchor { .. } => ProtocolKind::Anchor,
            Self::Kickstart { .. } => ProtocolKind::Kickstart,
            Self::TierRegression { .. } => ProtocolKind::TierRegression,
            Self::DebtCollector { .. } => ProtocolKind::DebtCollector,
            Self::IronCurtain { .. } => ProtocolKind::IronCurtain,
            Self::EchoStrike { .. } => ProtocolKind::EchoStrike,
            Self::Siphon { .. } => ProtocolKind::Siphon,
            Self::Greed { .. } => ProtocolKind::Greed,
            Self::RecklessDash { .. } => ProtocolKind::RecklessDash,
            Self::Burnout { .. } => ProtocolKind::Burnout,
            Self::Conductor { .. } => ProtocolKind::Conductor,
            Self::Afterimage { .. } => ProtocolKind::Afterimage,
            Self::Fission { .. } => ProtocolKind::Fission,
        }
    }

    /// Return the effect tree if this is an effect-tree protocol.
    /// EXHAUSTIVE — no wildcard. Adding a new variant forces a decision here.
    pub fn effects(&self) -> Option<&[RootEffect]> {
        match self {
            Self::Deadline { effects }
            | Self::RicochetProtocol { effects }
            | Self::Anchor { effects }
            | Self::Kickstart { effects } => Some(effects),
            Self::TierRegression { .. }
            | Self::DebtCollector { .. }
            | Self::IronCurtain { .. }
            | Self::EchoStrike { .. }
            | Self::Siphon { .. }
            | Self::Greed { .. }
            | Self::RecklessDash { .. }
            | Self::Burnout { .. }
            | Self::Conductor { .. }
            | Self::Afterimage { .. }
            | Self::Fission { .. } => None,
        }
    }
}
```

**Changes from draft**: `Conductor` now has a `primary_swap_window: f32` field (all protocols must have RON-tunable values per design rules). `effects()` uses exhaustive match — compiler catches missing variants.

### `HazardTuning`

Same as draft (already correct). All 16 variants with struct fields. `kind()` method with exhaustive match.

---

## 3. Definition Types (RON Assets)

### `ProtocolDefinition`

```rust
// protocol/definition.rs
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProtocolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub unlock_tier: u32,
    pub tuning: ProtocolTuning,
}

impl ProtocolDefinition {
    #[must_use]
    pub fn kind(&self) -> ProtocolKind {
        self.tuning.kind()
    }
}
```

### `HazardDefinition`

```rust
// hazard/definition.rs
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HazardDefinition {
    pub name: String,
    pub description: String,
    pub tuning: HazardTuning,
}

impl HazardDefinition {
    #[must_use]
    pub fn kind(&self) -> HazardKind {
        self.tuning.kind()
    }
}
```

### Registry Invariant

**There must be exactly one RON file per enum variant.** Adding a new protocol/hazard requires both a Rust variant in the tuning enum AND a corresponding RON file. The registry `seed()` warns on duplicates (two RON files for the same variant) and the variant has no entry if no RON file exists.

---

## 4. Registry Types

### `ProtocolRegistry`

```rust
// protocol/resources.rs
#[derive(Resource, Debug, Default)]
pub struct ProtocolRegistry {
    protocols: HashMap<ProtocolKind, ProtocolDefinition>,
}

impl ProtocolRegistry {
    #[must_use]
    pub fn get(&self, kind: ProtocolKind) -> Option<&ProtocolDefinition> {
        self.protocols.get(&kind)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ProtocolKind, &ProtocolDefinition)> {
        self.protocols.iter().map(|(&k, v)| (k, v))
    }
}

impl SeedableRegistry for ProtocolRegistry {
    type Asset = ProtocolDefinition;

    fn asset_dir() -> &'static str { "protocols" }
    fn extensions() -> &'static [&'static str] { &["protocol.ron"] }

    fn seed(&mut self, assets: &[(AssetId<ProtocolDefinition>, ProtocolDefinition)]) {
        self.protocols.clear();
        for (_id, def) in assets {
            let kind = def.kind();
            if self.protocols.contains_key(&kind) {
                warn!("duplicate protocol kind {kind:?} — skipping");
                continue;
            }
            self.protocols.insert(kind, def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<ProtocolDefinition>, asset: &ProtocolDefinition) {
        self.protocols.insert(asset.kind(), asset.clone());
    }
}
```

### `HazardRegistry` — identical pattern, keyed on `HazardKind`

---

## 5. Per-Run State Resources

### `ActiveProtocols`

```rust
// protocol/resources.rs
#[derive(Resource, Debug, Default)]
pub struct ActiveProtocols {
    taken: HashSet<ProtocolKind>,
}

impl ActiveProtocols {
    pub fn insert(&mut self, kind: ProtocolKind) -> bool { self.taken.insert(kind) }
    #[must_use]
    pub fn contains(&self, kind: &ProtocolKind) -> bool { self.taken.contains(kind) }
    pub fn clear(&mut self) { self.taken.clear(); }
}
```

### `ActiveHazards`

```rust
// hazard/resources.rs

/// Tracks which hazards are active and their stack counts.
///
/// Invariant: the map never contains a 0-value entry. `add_stack` always
/// produces >= 1, and hazards are never decremented. `stacks()` returns 0
/// for absent entries, which is equivalent to "not active."
#[derive(Resource, Debug, Default)]
pub struct ActiveHazards {
    stacks: HashMap<HazardKind, u32>,
}

impl ActiveHazards {
    pub fn add_stack(&mut self, kind: HazardKind) -> u32 {
        let count = self.stacks.entry(kind).or_insert(0);
        *count += 1;
        *count
    }

    /// Returns 0 if the hazard is not active.
    #[must_use]
    pub fn stacks(&self, kind: HazardKind) -> u32 {
        self.stacks.get(&kind).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn is_active(&self, kind: HazardKind) -> bool {
        self.stacks.get(&kind).is_some_and(|&s| s > 0)
    }

    pub fn clear(&mut self) { self.stacks.clear(); }
}
```

### `UnlockedProtocols`

```rust
// protocol/resources.rs
#[derive(Resource, Debug)]
pub struct UnlockedProtocols {
    unlocked: HashSet<ProtocolKind>,
}

impl Default for UnlockedProtocols {
    fn default() -> Self {
        Self { unlocked: ProtocolKind::ALL.iter().copied().collect() }
    }
}
```

### Per-run reset

`reset_run_state` already crosses into `chips` to clear `ChipInventory`. Adding `active_protocols.clear()` and `active_hazards.clear()` follows the same precedent. This is an accepted cross-domain write exception for run-level cleanup. A future refactor could move each domain's cleanup to its own `OnExit` system, but that changes the existing pattern and is out of scope.

---

## 6. Messages

```rust
// protocol/messages.rs
#[derive(Message, Clone, Debug)]
pub struct ProtocolSelected {
    pub kind: ProtocolKind,
}

// hazard/messages.rs
#[derive(Message, Clone, Debug)]
pub struct HazardSelected {
    pub kind: HazardKind,
}
```

Both are command messages — owned by the receiving domain, sent by the UI domain. This matches how `ChipSelected` works.

---

## 7. Offering Resources

### `ProtocolOffer` — owned by protocol domain

```rust
// protocol/resources.rs (NOT chip_select — protocol domain owns this)
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct ProtocolOffer(pub(crate) Option<ProtocolDefinition>);
```

The chip select UI reads `Res<ProtocolOffer>` (cross-domain read — allowed). The protocol domain's `generate_protocol_offering` writes it.

### `HazardOffers` — owned by hazard domain

```rust
// hazard/resources.rs
#[derive(Resource, Debug, Clone)]
pub(crate) struct HazardOffers(pub(crate) Vec<HazardDefinition>);
```

---

## 8. System Gating

```rust
// protocol/resources.rs
pub fn protocol_active(kind: ProtocolKind) -> impl Fn(Res<ActiveProtocols>) -> bool {
    move |active: Res<ActiveProtocols>| active.contains(&kind)
}

// hazard/resources.rs
pub fn hazard_active(kind: HazardKind) -> impl Fn(Res<ActiveHazards>) -> bool {
    move |active: Res<ActiveHazards>| active.is_active(kind)
}
```

No `+ Clone` on the return type — `run_if` doesn't require it (only `distributive_run_if` does).

---

## 9. Per-Protocol Module Pattern

Each custom-system protocol defines a **config resource** (tuning values) and a **runtime system**. The config is populated once at activation time by `dispatch_protocol_selection` — systems never touch the registry or match on tuning enums.

### Config resource

```rust
// protocol/protocols/debt_collector.rs

/// Tuning values for Debt Collector, extracted from ProtocolTuning at activation time.
#[derive(Resource, Debug, Clone)]
pub(crate) struct DebtCollectorConfig {
    pub stack_per_bump: f32,
}

/// Per-bolt debt multiplier stack.
#[derive(Component, Debug, Default)]
pub(crate) struct DebtStack(pub f32);
```

### Activation function

Each module provides an `activate` function called by `dispatch_protocol_selection`:

```rust
// protocol/protocols/debt_collector.rs

pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::DebtCollector { stack_per_bump } = tuning else { return };
    commands.insert_resource(DebtCollectorConfig {
        stack_per_bump: *stack_per_bump,
    });
}
```

The `let-else { return }` is a safety guard that runs exactly once at activation, not per-frame. In practice, `dispatch_protocol_selection` only calls this with matching tuning.

### Runtime system — reads config directly, no registry lookup, no enum match

```rust
// protocol/protocols/debt_collector.rs

pub(crate) fn debt_collector_on_bump(
    config: Res<DebtCollectorConfig>,
    mut reader: MessageReader<BumpPerformed>,
    mut bolts: Query<&mut DebtStack>,
) {
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok(mut stack) = bolts.get_mut(bolt) else { continue };
        match msg.grade {
            BumpGrade::Early | BumpGrade::Late => { stack.0 += config.stack_per_bump; }
            BumpGrade::Perfect => { stack.0 = 0.0; }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        debt_collector_on_bump
            .run_if(protocol_active(ProtocolKind::DebtCollector))
            .run_if(in_state(NodeState::Playing)),
    );
}
```

**Hot-reload**: If tuning values change in the RON file, the registry updates, but the config resource is stale until the next run (config is extracted once at activation). This is acceptable — tuning changes take effect on the next run. If mid-run hot-reload is needed, a `propagate_protocol_configs` system could re-extract from the registry, but YAGNI.

`DebtStack` is co-located with its system (following `effect/` pattern). If other systems need to read it, extract to `protocol/components.rs`.

---

## 10. Dispatch Systems

### `dispatch_protocol_selection` — activates a protocol + populates config

```rust
// protocol/systems/dispatch_protocol_selection.rs

pub(crate) fn dispatch_protocol_selection(
    mut reader: MessageReader<ProtocolSelected>,
    registry: Res<ProtocolRegistry>,
    mut active: ResMut<ActiveProtocols>,
    breaker_query: Query<Entity, With<Breaker>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if !active.insert(msg.kind) {
            warn!("protocol {:?} already active", msg.kind);
            continue;
        }

        let Some(def) = registry.get(msg.kind) else {
            warn!("protocol {:?} not in registry", msg.kind);
            continue;
        };

        // Effect-tree protocols: dispatch effects to breaker
        if let Some(effects) = def.tuning.effects() {
            for breaker in &breaker_query {
                dispatch_effects(effects, breaker, &mut commands, &def.name);
            }
        }

        // Custom-system protocols: populate per-protocol config resource
        protocols::activate(msg.kind, &def.tuning, &mut commands);
    }
}
```

### `protocols::activate` — routes to per-module activate functions

```rust
// protocol/protocols/mod.rs

pub(crate) fn activate(kind: ProtocolKind, tuning: &ProtocolTuning, commands: &mut Commands) {
    match kind {
        ProtocolKind::DebtCollector => debt_collector::activate(tuning, commands),
        ProtocolKind::Siphon => siphon::activate(tuning, commands),
        ProtocolKind::EchoStrike => echo_strike::activate(tuning, commands),
        ProtocolKind::Fission => fission::activate(tuning, commands),
        ProtocolKind::Burnout => burnout::activate(tuning, commands),
        ProtocolKind::Conductor => conductor::activate(tuning, commands),
        ProtocolKind::Afterimage => afterimage::activate(tuning, commands),
        ProtocolKind::RecklessDash => reckless_dash::activate(tuning, commands),
        ProtocolKind::Greed => greed::activate(tuning, commands),
        ProtocolKind::IronCurtain => iron_curtain::activate(tuning, commands),
        ProtocolKind::TierRegression => tier_regression::activate(tuning, commands),
        // Effect-tree protocols have no config to populate
        ProtocolKind::Deadline
        | ProtocolKind::RicochetProtocol
        | ProtocolKind::Anchor
        | ProtocolKind::Kickstart => {}
    }
}
```

### `dispatch_hazard_selection` — activates a hazard stack + populates config on first stack

```rust
// hazard/systems/dispatch_hazard_selection.rs

pub(crate) fn dispatch_hazard_selection(
    mut reader: MessageReader<HazardSelected>,
    registry: Res<HazardRegistry>,
    mut active: ResMut<ActiveHazards>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let new_count = active.add_stack(msg.kind);

        // First stack: populate config resource from registry
        if new_count == 1 {
            let Some(def) = registry.get(msg.kind) else { continue };
            hazards::activate(msg.kind, &def.tuning, &mut commands);
        }
        // Subsequent stacks: config already populated, only stack count changes
    }
}
```

### Per-run cleanup

The protocol and hazard plugins register cleanup systems that remove config resources at run end:

```rust
// protocol/protocols/mod.rs
pub(crate) fn cleanup(commands: &mut Commands) {
    commands.remove_resource::<DebtCollectorConfig>();
    commands.remove_resource::<SiphonConfig>();
    // ... all 11 custom-system protocol configs
}
```

Registered on `OnExit(MenuState::Main)` alongside `reset_run_state` (keeps cleanup in the protocol domain rather than adding 11 `ResMut` params to `reset_run_state`).

---

## 11. Cross-Domain Hazard Communication

**CRITICAL RULE**: Hazards MUST NOT directly mutate resources or components owned by other domains. All cross-domain mutation goes through messages.

### Message inventory for hazards

After the effect refactor (todo #2), `DamageCell` becomes `DamageDealt<Cell>`. The cell damage system (`apply_damage::<Cell>`) processes all `DamageDealt<Cell>` messages. Hazards that affect the damage pipeline (Diffusion, Tether, Sympathy, Momentum) work by having the cell damage system read hazard state and handle redistribution/healing — the hazards do NOT intercept messages themselves.

| Hazard | Cross-domain effect | Message pattern |
|--------|-------------------|-----------------|
| Decay | Timer ticks faster | `ApplyTimePenalty { seconds }` (existing message) |
| Drift | Wind pushes bolt | New `ApplyBoltForce { bolt: Entity, force: Vec2 }` — owned by `bolt` |
| Haste | Bolt speed increase | Effect system `SpeedBoost` via `During(HazardActive, ...)` or new message |
| Echo Cells | Ghost cells spawn | New `SpawnGhostCell { position: Vec2, hp: f32 }` — owned by `cells` |
| Erosion | Breaker shrinks | New `ApplyBreakerShrink { amount: f32 }` — owned by `breaker` |
| Cascade | Destroyed cell heals adjacents | New `HealCell { cell: Entity, amount: f32 }` — owned by `cells` |
| Fracture | Destroyed cells split | Cell spawn message — owned by `cells` |
| Renewal | Cells regen on timer | `HealCell` — owned by `cells` |
| Diffusion | Damage shared with adjacents | **No hazard message** — `apply_damage::<Cell>` reads `ActiveHazards` + `DiffusionConfig` and handles redistribution internally, sending additional `DamageDealt<Cell>` for adjacent cells |
| Tether | Linked cells share damage | **No hazard message** — `apply_damage::<Cell>` reads `ActiveHazards` + `TetherConfig` + `TetherLink` components and sends additional `DamageDealt<Cell>` for partners |
| Volatility | Cells gain HP when idle | `HealCell` — owned by `cells`. Per-cell idle timer tracked by hazard domain. |
| GravitySurge | Gravity wells pull bolt | `ApplyBoltForce` — same as Drift |
| Overcharge | Bolt speed per kill | Effect system `SpeedBoost` or per-bolt component tracked by hazard domain |
| Resonance | Slow-waves toward breaker | Spawns wave entities in hazard domain, sends `DamageDealt<Cell>` for affected cells |
| Momentum | Non-lethal hits add HP, split at 2x | `apply_damage::<Cell>` reads `MomentumConfig` on non-lethal hits, sends `HealCell` + cell spawn |
| Sympathy | Damage dealt heals adjacents | `apply_damage::<Cell>` reads `SympathyConfig` and sends `HealCell` for adjacent cells |

**Key insight for Diffusion/Tether/Momentum/Sympathy**: These hazards modify how the cell damage system behaves. Rather than intercepting messages, the `apply_damage::<Cell>` system (in the cells domain, after the effect refactor) reads hazard config resources and handles the redistribution/healing as part of its own logic. The hazard domain provides the config resources and any components (like `TetherLink`); the cells domain does the actual processing.

**New messages needed** (owned by the consuming domain):
- `HealCell { cell: Entity, amount: f32 }` — owned by `cells`
- `SpawnGhostCell { position: Vec2, hp: f32 }` — owned by `cells`
- `ApplyBoltForce { bolt: Entity, force: Vec2 }` — owned by `bolt`
- `ApplyBreakerShrink { amount: f32 }` — owned by `breaker`

Several hazards (Haste, Overcharge) may be expressible through the effect system's `SpeedBoost` rather than new messages. This depends on whether the effect system supports "ambient" effects not tied to a trigger — likely via `During(HazardActive, Fire(SpeedBoost(...)))` in the new system.

### Hazard system pattern (with config + messages)

```rust
// hazard/hazards/decay.rs

/// Tuning values for Decay, extracted from HazardTuning at activation time.
#[derive(Resource, Debug, Clone)]
pub(crate) struct DecayConfig {
    pub base_percent: f32,
    pub per_level_percent: f32,
}

pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Decay { base_percent, per_level_percent } = tuning else { return };
    commands.insert_resource(DecayConfig {
        base_percent: *base_percent,
        per_level_percent: *per_level_percent,
    });
}

pub(crate) fn decay_tick(
    config: Res<DecayConfig>,
    active: Res<ActiveHazards>,
    mut penalty: MessageWriter<ApplyTimePenalty>,
    time: Res<Time>,
) {
    let stack = active.stacks(HazardKind::Decay);
    // Linear scaling: base + per_level * (stack - 1)
    let speedup_percent = config.base_percent + config.per_level_percent * (stack - 1) as f32;
    let extra_drain = time.delta_secs() * speedup_percent / 100.0;
    // Message-driven: sends to run/node domain, does NOT write NodeTimer directly
    penalty.write(ApplyTimePenalty { seconds: extra_drain });
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        decay_tick
            .run_if(hazard_active(HazardKind::Decay))
            .run_if(in_state(NodeState::Playing))
            .after(NodeSystems::TickTimer),
    );
}
```

**Key properties**: Config resource holds base tuning (populated once). `ActiveHazards` holds stack count (increments on each selection). System reads both. Cross-domain mutation goes through `ApplyTimePenalty` message. Zero enum matching at runtime.

---

## 12. Effect Dispatch for Protocols

Effect-tree protocols are dispatched through the NEW effect system (todo #2), not the current one. The new system's `Route`/`Stamp`/`Transfer` primitives and the new dispatch API will handle protocol effect installation. No extraction from the current `dispatch_chip_effects` is needed — both chips and protocols will use the new effect system's dispatch API after the refactor.

---

## 13. Plugin Structure

### `ProtocolPlugin`

```rust
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveProtocols>()
           .init_resource::<UnlockedProtocols>()
           .init_resource::<ProtocolOffer>()
           .add_message::<ProtocolSelected>();

        app.add_systems(
            Update,
            dispatch_protocol_selection.run_if(in_state(ChipSelectState::Selecting)),
        );

        super::protocols::register(app);
    }
}
```

### `HazardPlugin`

```rust
pub struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveHazards>()
           .add_message::<HazardSelected>();

        app.add_systems(
            Update,
            dispatch_hazard_selection.run_if(in_state(HazardSelectState::Selecting)),
        );

        super::hazards::register(app);
    }
}
```

---

## Design Decisions Log

| Decision | Rationale |
|----------|-----------|
| Tuning enum IS the kind discriminant | Eliminates kind/tuning mismatch. Kind derived via `tuning.kind()`. Follows `EffectKind` pattern. |
| Per-item config resources extracted at activation | Systems read `Res<DebtCollectorConfig>` — no registry lookup, no enum matching per-frame. One `let-else` in `activate()` at selection time (runs once). Hot-reload of tuning takes effect next run. |
| Registry keyed on `ProtocolKind` enum, not `String` | Type-safe lookups. Invariant: one RON file per variant. Consistent with `HashMap<Rarity, f32>` in offering system. |
| All definition types in `definition.rs` | Canonical layout prohibits `types.rs`. Content enums and asset types belong in `definition.rs`. |
| `ProtocolOffer` owned by protocol domain | Writer-owns-the-resource pattern. Chip select UI reads it cross-domain (allowed). |
| Exhaustive match in `effects()` | Compiler catches missing variants when new protocols are added. No wildcards. |
| `unreachable!()` with message in system tuning match | Correct for programmer errors (registry is keyed by kind, so mismatch is structurally impossible). Message aids debugging. |
| `stacks() -> u32` (not `Option`) | Invariant: map never contains 0. Callers use `run_if(hazard_active(...))` for presence check. Returning `Option` adds unwrap noise for no safety benefit. |
| No `+ Clone` on `run_if` return types | `run_if` doesn't require `Clone`. Only `distributive_run_if` does. Closure captures `Copy` kind anyway. |
| Message-driven cross-domain mutation for hazards | Architecture requires message-only cross-domain communication. Each hazard sends messages to the owning domain, never directly mutates foreign resources. |
| Config resources removed at run end, not cleared | Unlike per-run resources that always exist (`ChipInventory`), config resources are inserted on activation and removed at run end. This avoids needing Default values that mean "inactive." Protocol/hazard domains own their cleanup. |
| Shared effect dispatch extraction | `dispatch_chip_effects` and `dispatch_protocol_selection` share the same dispatch logic. Extract once, call from both. |
| `ProtocolKind::ALL` as manually maintained const | No `strum` dependency for 15 variants. Compiler won't catch missing entries, but tests can assert `ALL.len() == variant_count`. |
| Per-run resources cleared in `reset_run_state` | Accepted cross-domain exception, consistent with existing `ChipInventory.clear()` pattern. |

---

## Resolved Questions

1. **Protocol + chip selection**: Picking the protocol **closes the chip select screen**. It's protocol OR chip, not both. `handle_chip_input` sends `ProtocolSelected` + `ChangeState<ChipSelectState>` (same flow as picking a chip).
2. **Hazard select timer**: **Timed** — same pattern as chip select. On timer expiry, a hazard is **auto-picked at random** from the 3 offers. Same `tick_hazard_timer` → `ChangeState<HazardSelectState>` pattern.
3. **Effect-tree protocol timing**: Protocol implementation **waits for the effect refactor** (todo #2). `ProtocolDefinition.tuning` effect-tree variants will use the new system's `ValidDef` types, not `RootEffect`. This is a hard dependency for Waves 4-5.
4. **Protocol offering per tier**: **Random from seeded `GameRng`**. The entire run is deterministic from its seed, so the random pick is reproducible. `generate_protocol_offering` draws from the available pool using `GameRng`.
