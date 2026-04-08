# Protocol & Hazard Interface Design — Draft

## Design Principles

1. **Match existing patterns exactly** — no new idioms (no traits, no observers, no trait objects)
2. **Tuning enum IS the kind discriminant** — no redundant `kind` field + separate tuning
3. **Systems read registry directly** — no per-protocol config resources (registry is always available, hot-reloadable)
4. **`run_if` guards for activation** — no runtime checks inside system bodies
5. **Effect-tree protocols dispatch like chips** — same `dispatch_chip_effects` pattern
6. **One module per protocol/hazard** — `register(app)` delegation pattern from `effect/`

---

## 1. Core Enums

### `ProtocolKind` — C-style enum for HashMap keys and active tracking

```rust
// protocol/types.rs
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
```

Derives: `Deserialize` for RON, `Hash + Eq` for HashMap keys, `Copy` because it's a fieldless enum.

### `HazardKind` — same pattern

```rust
// hazard/types.rs
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum HazardKind {
    Decay,
    Drift,
    Haste,
    EchoCells,
    Erosion,
    Cascade,
    Fracture,
    Renewal,
    Diffusion,
    Tether,
    Volatility,
    GravitySurge,
    Overcharge,
    Resonance,
    Momentum,
    Sympathy,
}
```

---

## 2. Tuning Enums

### `ProtocolTuning` — enum with struct variants, each variant has kind-specific fields

```rust
// protocol/types.rs
#[derive(Clone, Debug, Deserialize)]
pub enum ProtocolTuning {
    // ── Effect-tree protocols ──────────────────────────────────────
    // Tuning values are embedded in the effect tree. The `effects` field
    // contains the full tree dispatched through the effect system.
    Deadline {
        effects: Vec<RootEffect>,
    },
    RicochetProtocol {
        effects: Vec<RootEffect>,
    },
    Anchor {
        effects: Vec<RootEffect>,
    },
    Kickstart {
        effects: Vec<RootEffect>,
    },

    // ── Custom-system protocols ────────────────────────────────────
    // Each variant contains the RON-tunable values read by its system.
    TierRegression {
        tiers_back: u32,
    },
    DebtCollector {
        stack_per_bump: f32,
    },
    IronCurtain {
        damage_fraction: f32,
        falloff_start: f32,
    },
    EchoStrike {
        max_echoes: u32,
        newest_fraction: f32,
        middle_fraction: f32,
        oldest_fraction: f32,
    },
    Siphon {
        streak_window: f32,
        time_per_kill: f32,
    },
    Greed {
        rarity_boost_per_skip: f32,
    },
    RecklessDash {
        risky_zone_start: f32,
        damage_multiplier: f32,
        double_penalty: bool,
    },
    Burnout {
        fill_duration: f32,
        drain_duration: f32,
        still_threshold: f32,
        full_heat_damage_multiplier: f32,
        speed_boost_duration: f32,
    },
    Conductor,
    Afterimage {
        phantom_duration: f32,
        phantom_bolt_duration: f32,
    },
    Fission {
        kills_per_split: u32,
    },
}
```

**Why this shape**: Follows the `EffectKind` pattern (26 variants, each with different fields). The variant IS the kind — no redundant discriminant field. Systems pattern-match to extract their tuning values.

**`ProtocolTuning` methods**:

```rust
impl ProtocolTuning {
    /// Derive the ProtocolKind from the tuning variant.
    pub const fn kind(&self) -> ProtocolKind {
        match self {
            Self::Deadline { .. } => ProtocolKind::Deadline,
            Self::RicochetProtocol { .. } => ProtocolKind::RicochetProtocol,
            // ... exhaustive
        }
    }

    /// Return the effect tree if this is an effect-tree protocol.
    /// Custom-system protocols return None.
    pub fn effects(&self) -> Option<&[RootEffect]> {
        match self {
            Self::Deadline { effects }
            | Self::RicochetProtocol { effects }
            | Self::Anchor { effects }
            | Self::Kickstart { effects } => Some(effects),
            _ => None,
        }
    }
}
```

### `HazardTuning` — same pattern

```rust
// hazard/types.rs
#[derive(Clone, Debug, Deserialize)]
pub enum HazardTuning {
    Decay {
        base_percent: f32,
        per_level_percent: f32,
    },
    Drift {
        base_force: f32,
        force_per_level: f32,
        change_interval: f32,
    },
    Haste {
        base_percent: f32,
        per_level_percent: f32,
    },
    EchoCells {
        respawn_delay: f32,
        base_hp: f32,
        hp_doubles_per_level: bool,
    },
    Erosion {
        shrink_rate: f32,
        min_width_fraction: f32,
        non_whiff_restore: f32,
        perfect_restore: f32,
    },
    Cascade {
        base_heal: f32,
        heal_per_level: f32,
    },
    Fracture {
        base_splits: u32,
        splits_per_level: u32,
    },
    Renewal {
        base_timer: f32,
        per_level_reduction_percent: f32,
    },
    Diffusion {
        base_share_percent: f32,
        share_per_level_percent: f32,
        depth_increase_interval: u32,
    },
    Tether {
        base_damage_percent: f32,
        damage_per_level_percent: f32,
        base_coverage_percent: f32,
        coverage_per_level_percent: f32,
    },
    Volatility {
        hp_per_interval: f32,
        interval_secs: f32,
        max_multiplier: f32,
    },
    GravitySurge {
        base_duration: f32,
        duration_per_level: f32,
        base_strength: f32,
        strength_per_level_diminishing: f32,
    },
    Overcharge {
        base_speed_per_kill: f32,
        speed_per_level: f32,
    },
    Resonance {
        kills_to_trigger: u32,
        base_window: f32,
        window_per_level: f32,
        wave_speed: f32,
    },
    Momentum {
        base_hp_per_hit: f32,
        hp_per_level: f32,
        split_threshold_multiplier: f32,
    },
    Sympathy {
        base_heal_percent: f32,
        heal_per_level_percent: f32,
        depth_increase_interval: u32,
    },
}
```

**`HazardTuning` methods**:

```rust
impl HazardTuning {
    pub const fn kind(&self) -> HazardKind {
        match self {
            Self::Decay { .. } => HazardKind::Decay,
            // ... exhaustive
        }
    }
}
```

---

## 3. Definition Types (RON Assets)

### `ProtocolDefinition`

```rust
// protocol/definition.rs
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProtocolDefinition {
    /// Display name shown on the protocol card.
    pub name: String,
    /// Flavor text shown below the name.
    pub description: String,
    /// Meta-progression tier required to unlock. 0 = always available.
    #[serde(default)]
    pub unlock_tier: u32,
    /// Kind-specific tuning values (and effect trees for effect-tree protocols).
    pub tuning: ProtocolTuning,
}

impl ProtocolDefinition {
    /// Derive the ProtocolKind from the tuning variant.
    #[must_use]
    pub fn kind(&self) -> ProtocolKind {
        self.tuning.kind()
    }
}
```

Derives: `Asset, TypePath, Deserialize, Clone, Debug` — matches `BreakerDefinition` exactly.

### `HazardDefinition`

```rust
// hazard/definition.rs
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct HazardDefinition {
    /// Display name shown on the hazard card.
    pub name: String,
    /// Flavor text describing the hazard.
    pub description: String,
    /// Kind-specific tuning values and scaling parameters.
    pub tuning: HazardTuning,
}

impl HazardDefinition {
    #[must_use]
    pub fn kind(&self) -> HazardKind {
        self.tuning.kind()
    }
}
```

### Example RON files

**`assets/protocols/deadline.protocol.ron`**:
```ron
(
    name: "Deadline",
    description: "When the node timer drops below 25%, all bolts get 2x speed and 2x damage.",
    unlock_tier: 0,
    tuning: Deadline(
        effects: [
            Route(Bolt, When(NodeTimerThresholdOccurred(0.25),
                During(NodeActive, Fire(SpeedBoost(multiplier: 2.0))))),
            Route(Bolt, When(NodeTimerThresholdOccurred(0.25),
                During(NodeActive, Fire(DamageBoost(2.0))))),
        ],
    ),
)
```

**`assets/protocols/debt_collector.protocol.ron`**:
```ron
(
    name: "Debt Collector",
    description: "Early/Late bumps build a damage multiplier. Perfect bump cashes it out.",
    unlock_tier: 0,
    tuning: DebtCollector(
        stack_per_bump: 0.5,
    ),
)
```

**`assets/hazards/decay.hazard.ron`**:
```ron
(
    name: "Decay",
    description: "Node timer ticks faster. Stacks intensify.",
    tuning: Decay(
        base_percent: 15.0,
        per_level_percent: 5.0,
    ),
)
```

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

    #[must_use]
    pub fn len(&self) -> usize {
        self.protocols.len()
    }
}

impl SeedableRegistry for ProtocolRegistry {
    type Asset = ProtocolDefinition;

    fn asset_dir() -> &'static str {
        "protocols"
    }

    fn extensions() -> &'static [&'static str] {
        &["protocol.ron"]
    }

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

**Key difference from BreakerRegistry**: keyed on `ProtocolKind` enum instead of `String`. Type-safe lookups. The kind is derived from the definition's tuning variant — no separate key field in the RON.

### `HazardRegistry` — identical pattern

```rust
// hazard/resources.rs
#[derive(Resource, Debug, Default)]
pub struct HazardRegistry {
    hazards: HashMap<HazardKind, HazardDefinition>,
}

impl SeedableRegistry for HazardRegistry {
    type Asset = HazardDefinition;

    fn asset_dir() -> &'static str {
        "hazards"
    }

    fn extensions() -> &'static [&'static str] {
        &["hazard.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<HazardDefinition>, HazardDefinition)]) {
        self.hazards.clear();
        for (_id, def) in assets {
            let kind = def.kind();
            if self.hazards.contains_key(&kind) {
                warn!("duplicate hazard kind {kind:?} — skipping");
                continue;
            }
            self.hazards.insert(kind, def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<HazardDefinition>, asset: &HazardDefinition) {
        self.hazards.insert(asset.kind(), asset.clone());
    }
}
```

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
    pub fn insert(&mut self, kind: ProtocolKind) -> bool {
        self.taken.insert(kind)
    }

    #[must_use]
    pub fn contains(&self, kind: &ProtocolKind) -> bool {
        self.taken.contains(kind)
    }

    pub fn clear(&mut self) {
        self.taken.clear();
    }
}
```

**Lifecycle**: `init_resource::<ActiveProtocols>()` in `ProtocolPlugin::build()`. Cleared by `reset_run_state` (add `active_protocols.clear()` alongside the existing `chip_inventory.clear()`). Never removed.

### `ActiveHazards`

```rust
// hazard/resources.rs
#[derive(Resource, Debug, Default)]
pub struct ActiveHazards {
    stacks: HashMap<HazardKind, u32>,
}

impl ActiveHazards {
    /// Increment the stack count for a hazard. Returns the new count.
    pub fn add_stack(&mut self, kind: HazardKind) -> u32 {
        let count = self.stacks.entry(kind).or_insert(0);
        *count += 1;
        *count
    }

    /// Get the stack count for a hazard. Returns 0 if not active.
    #[must_use]
    pub fn stacks(&self, kind: HazardKind) -> u32 {
        self.stacks.get(&kind).copied().unwrap_or(0)
    }

    #[must_use]
    pub fn is_active(&self, kind: HazardKind) -> bool {
        self.stacks.get(&kind).is_some_and(|&s| s > 0)
    }

    pub fn clear(&mut self) {
        self.stacks.clear();
    }
}
```

**Lifecycle**: Same pattern as `ActiveProtocols`. Cleared by `reset_run_state`.

### `UnlockedProtocols` — meta-progression state

```rust
// protocol/resources.rs
#[derive(Resource, Debug)]
pub struct UnlockedProtocols {
    unlocked: HashSet<ProtocolKind>,
}

impl Default for UnlockedProtocols {
    fn default() -> Self {
        // All protocols unlocked by default until meta-progression is implemented
        Self {
            unlocked: ProtocolKind::ALL.iter().copied().collect(),
        }
    }
}

impl UnlockedProtocols {
    #[must_use]
    pub fn is_unlocked(&self, kind: &ProtocolKind) -> bool {
        self.unlocked.contains(kind)
    }
}
```

**Lifecycle**: `init_resource::<UnlockedProtocols>()` in `ProtocolPlugin::build()`. Persists across runs (meta-progression). Updated by a future save/load system.

---

## 6. Messages

### `ProtocolSelected`

```rust
// protocol/messages.rs
#[derive(Message, Clone, Debug)]
pub struct ProtocolSelected {
    pub kind: ProtocolKind,
}
```

Sent by: `handle_chip_input` (in chip select UI) when the player picks the protocol card.
Consumed by: `dispatch_protocol_selection` (in `ProtocolPlugin`).

### `HazardSelected`

```rust
// hazard/messages.rs
#[derive(Message, Clone, Debug)]
pub struct HazardSelected {
    pub kind: HazardKind,
}
```

Sent by: `handle_hazard_input` (in hazard select UI).
Consumed by: `dispatch_hazard_selection` (in `HazardPlugin`).

---

## 7. Offering Resources

### `ProtocolOffer`

```rust
// state/run/chip_select/resources.rs (alongside ChipOffers)
#[derive(Resource, Debug, Clone, Default)]
pub(crate) struct ProtocolOffer(pub Option<ProtocolDefinition>);
```

`None` = no protocol available (all taken or none unlocked) → protocol row hidden.
`Some(def)` = protocol available → UI checks `ActiveProtocols.contains(def.kind())` to show as taken or available.

Generated by `generate_protocol_offering` on `OnEnter(ChipSelectState::Selecting)`.

### `HazardOffers`

```rust
// state/run/hazard_select/resources.rs
#[derive(Resource, Debug, Clone)]
pub(crate) struct HazardOffers(pub Vec<HazardDefinition>);

#[derive(Resource, Debug, Default)]
pub(crate) struct HazardSelectSelection {
    pub index: usize,
}
```

Always 3 entries. Generated by `generate_hazard_offerings` on `OnEnter(HazardSelectState::Selecting)`.

---

## 8. System Gating Pattern

### `protocol_active` — shared run_if helper

```rust
// protocol/resources.rs
pub fn protocol_active(kind: ProtocolKind) -> impl Fn(Res<ActiveProtocols>) -> bool + Clone {
    move |active: Res<ActiveProtocols>| active.contains(&kind)
}
```

Used in every custom-system protocol's `register()`:
```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        debt_collector_system
            .run_if(protocol_active(ProtocolKind::DebtCollector))
            .run_if(in_state(NodeState::Playing)),
    );
}
```

### `hazard_active` — same pattern for hazards

```rust
// hazard/resources.rs
pub fn hazard_active(kind: HazardKind) -> impl Fn(Res<ActiveHazards>) -> bool + Clone {
    move |active: Res<ActiveHazards>| active.is_active(kind)
}
```

---

## 9. Per-Protocol Module Pattern

### Simple custom-system protocol (single file)

```rust
// protocol/protocols/debt_collector.rs

use bevy::prelude::*;
use crate::prelude::*;

use super::super::resources::{ActiveProtocols, protocol_active};
use super::super::types::ProtocolKind;

/// Tracks the current debt multiplier stack on a bolt.
#[derive(Component, Debug, Default)]
pub(crate) struct DebtStack(pub f32);

pub(crate) fn debt_collector_on_bump(
    registry: Res<ProtocolRegistry>,
    mut reader: MessageReader<BumpPerformed>,
    mut bolts: Query<&mut DebtStack>,
) {
    let def = registry.get(ProtocolKind::DebtCollector).unwrap();
    let ProtocolTuning::DebtCollector { stack_per_bump } = &def.tuning else {
        unreachable!()
    };

    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok(mut stack) = bolts.get_mut(bolt) else { continue };

        match msg.grade {
            BumpGrade::Early | BumpGrade::Late => {
                stack.0 += stack_per_bump;
            }
            BumpGrade::Perfect => {
                // Cash out happens in the damage system, stack resets
                stack.0 = 0.0;
            }
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

#[cfg(test)]
mod tests {
    // ...
}
```

### Effect-tree protocol (minimal)

```rust
// protocol/protocols/deadline.rs

// No register() needed — effect-tree protocols have no runtime systems.
// Their effects are dispatched through the effect system by
// dispatch_protocol_selection when the protocol is selected.
// The RON file contains the full effect tree.
```

Effect-tree protocols may not even need a module file. Their behavior is entirely in the RON effect tree. They're dispatched by the shared `dispatch_protocol_selection` system. If they need no custom code, they don't need a module.

### Complex custom-system protocol (directory)

```rust
// protocol/protocols/afterimage/mod.rs
pub(crate) use system::register;

mod system;

#[cfg(test)]
mod tests;
```

```rust
// protocol/protocols/afterimage/system.rs
// Components, systems, register() — same pattern as shockwave/effect.rs
```

---

## 10. Dispatch Systems

### `dispatch_protocol_selection` — activates a protocol

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

        // Effect-tree protocols: dispatch effects to breaker (same as chips)
        if let Some(effects) = def.tuning.effects() {
            for breaker in &breaker_query {
                // dispatch_effects mirrors the chip dispatch pattern
                // wraps non-Breaker targets in When(NodeStart, On(target, ...))
                dispatch_protocol_effects(effects, breaker, &mut commands, &def.name);
            }
        }

        // Custom-system protocols: no dispatch needed.
        // Their systems activate via the run_if(protocol_active(...)) guard.
    }
}
```

Runs in: `Update`, `run_if(in_state(ChipSelectState::Selecting))`.
Registered by: `ProtocolPlugin::build()`.

### `dispatch_hazard_selection` — activates a hazard stack

```rust
// hazard/systems/dispatch_hazard_selection.rs

pub(crate) fn dispatch_hazard_selection(
    mut reader: MessageReader<HazardSelected>,
    mut active: ResMut<ActiveHazards>,
) {
    for msg in reader.read() {
        let new_count = active.add_stack(msg.kind);
        info!("hazard {:?} now at stack {new_count}", msg.kind);
    }
}
```

Runs in: `Update`, `run_if(in_state(HazardSelectState::Selecting))`.
Registered by: `HazardPlugin::build()`.

---

## 11. Plugin Structure

### `ProtocolPlugin`

```rust
// protocol/plugin.rs

pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveProtocols>()
           .init_resource::<UnlockedProtocols>()
           .add_message::<ProtocolSelected>();

        // Dispatch system — runs during chip select
        app.add_systems(
            Update,
            dispatch_protocol_selection
                .run_if(in_state(ChipSelectState::Selecting)),
        );

        // Per-protocol runtime systems
        super::protocols::register(app);
    }
}
```

### `HazardPlugin`

```rust
// hazard/plugin.rs

pub struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveHazards>()
           .add_message::<HazardSelected>();

        // Dispatch system — runs during hazard select
        app.add_systems(
            Update,
            dispatch_hazard_selection
                .run_if(in_state(HazardSelectState::Selecting)),
        );

        // Per-hazard runtime systems
        super::hazards::register(app);
    }
}
```

### Top-level `protocols::register()` / `hazards::register()`

```rust
// protocol/protocols/mod.rs
pub(crate) fn register(app: &mut App) {
    debt_collector::register(app);
    echo_strike::register(app);
    siphon::register(app);
    fission::register(app);
    burnout::register(app);
    conductor::register(app);
    afterimage::register(app);
    reckless_dash::register(app);
    greed::register(app);
    iron_curtain::register(app);
    tier_regression::register(app);
    // Effect-tree protocols (deadline, ricochet, anchor, kickstart)
    // have no register() — their effects are dispatched by
    // dispatch_protocol_selection, not by runtime systems.
}
```

---

## 12. Hazard System Pattern (with Stacking)

Each hazard system reads its stack count from `ActiveHazards` and scales accordingly.

```rust
// hazard/hazards/decay.rs

pub(crate) fn decay_tick(
    registry: Res<HazardRegistry>,
    active: Res<ActiveHazards>,
    mut timer: ResMut<NodeTimer>,
    time: Res<Time>,
) {
    let stack = active.stacks(HazardKind::Decay);
    let def = registry.get(HazardKind::Decay).unwrap();
    let HazardTuning::Decay { base_percent, per_level_percent } = &def.tuning else {
        unreachable!()
    };

    // Linear scaling: base + per_level * (stack - 1)
    // stack=1: 15%, stack=2: 20%, stack=3: 25%
    let speedup_percent = base_percent + per_level_percent * (stack - 1) as f32;
    let extra_drain = time.delta_secs() * speedup_percent / 100.0;
    timer.remaining -= extra_drain;
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

---

## 13. Integration with Existing Systems

### Protocol offering (chip select screen integration)

**New system**: `generate_protocol_offering`
- Runs: `OnEnter(ChipSelectState::Selecting)`, alongside `generate_chip_offerings`
- Reads: `ProtocolRegistry`, `ActiveProtocols`, `UnlockedProtocols`, `GameRng`
- Writes: inserts `ProtocolOffer` resource
- Logic: filter registry for unlocked + not-taken, pick one at random, insert as `ProtocolOffer(Some(def))`

**Modified**: `spawn_chip_select` — spawns protocol card row below chip cards
**Modified**: `handle_chip_input` — up/down navigation between chip row and protocol row; protocol confirm sends `ProtocolSelected` instead of `ChipSelected`
**Modified**: `tick_chip_timer` — timer expiry skips protocol (protocol is never auto-selected)

### Hazard select screen (new state)

**New state**: `RunState::HazardSelect` + `HazardSelectState` (5 variants)
**New route**: `RunState::ChipSelect` → `to_dynamic(resolve_post_chip_state)`
**New systems**: `generate_hazard_offerings`, `spawn_hazard_select`, `handle_hazard_input`
**New plugin**: `HazardSelectPlugin` (UI systems, registered in `StatePlugin`)

### Per-run reset

Add to `reset_run_state`:
```rust
active_protocols.clear();
active_hazards.clear();
```

### Legendary removal

- Remove `Rarity::Legendary` variant
- Remove `rarity_weight_legendary` from `ChipSelectConfig` + RON
- Remove `legendary:` slots from 13 `.chip.ron` files
- Retune 13 chips as additional Rare variants
- Remove Legendary color config entries

### Anchor evolution removal

- Delete `assets/chips/evolutions/anchor.evolution.ron`
- Create `assets/protocols/anchor.protocol.ron` with effect tree

---

## Design Decisions Log

| Decision | Rationale |
|----------|-----------|
| Tuning enum IS the kind discriminant (no separate `kind` field) | Eliminates kind/tuning mismatch. Kind is derived via `tuning.kind()`. Follows `EffectKind` pattern. |
| Registry keyed on enum, not String | Type-safe lookups. Kind enum is used for active tracking, run_if guards, and system dispatch. String keys offer no advantage when the enum exists. |
| Systems read from registry, not per-protocol config resources | Registry is always available, hot-reloadable, and doesn't require insertion/removal of extra resources. Follows Occam's razor. |
| `run_if` guard instead of runtime check | Bevy skips the entire system when the condition is false — zero overhead. No runtime branching inside system bodies. |
| Effect-tree protocols have no module file | Their behavior is entirely in the RON effect tree. No custom Rust code needed. The shared `dispatch_protocol_selection` handles them. |
| `ActiveProtocols` uses `HashSet`, `ActiveHazards` uses `HashMap<_, u32>` | Protocols can't stack (taken or not). Hazards stack (count matters). Different data structures reflect different semantics. |
| Per-run resources are cleared, never removed | Matches the existing codebase pattern (`ChipInventory.clear()`, `NodeOutcome::default()`). |
| `ProtocolOffer(Option<ProtocolDefinition>)` not a variant of `ChipOffering` | Protocols are displayed separately from chips (different row, different orientation). They're not chip offerings — they're a parallel choice. Separate resource keeps the chip offering system untouched. |
| No `Protocol` or `Hazard` trait | The codebase uses no traits for dispatch. Effects use free functions. Protocols follow the same `register(app)` delegation pattern. |

---

## Open Questions

1. **Protocol offering per tier**: Is each tier assigned a specific protocol at run start (deterministic from seed), or is it randomly picked at offering time?
2. **Conductor tuning**: `Conductor` variant has no tuning fields. Is this intentional (behavior is entirely in code), or does it need tuning values?
3. **Effect-tree protocol RON format**: The examples use the CURRENT effect system's `RootEffect` RON format. After the effect refactor (todo #2), these will use `ValidDef` / `Route(...)` format. Should the `ProtocolDefinition` use `Vec<RootEffect>` now and migrate later, or should protocol implementation wait for the refactor?
4. **Hazard select timer**: Does the hazard select screen have a countdown timer like chip select, or is it untimed?
5. **Protocol card interaction with timer**: If the player selects the protocol, does the chip timer stop? Or can the player pick a chip after picking a protocol (i.e., you get BOTH a chip and a protocol)?
