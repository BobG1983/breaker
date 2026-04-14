# Name
ChainLightningChain, ChainState

# Struct
```rust
/// Tracks the state of a chain lightning effect as it arcs between cells.
#[derive(Component)]
pub struct ChainLightningChain {
    /// Number of jumps remaining before the chain ends.
    pub remaining_jumps: u32,
    /// Damage dealt per arc.
    pub damage: f32,
    /// Entities already hit by this chain (prevents revisiting).
    pub hit_set: HashSet<Entity>,
    /// Current state of the chain (idle or traveling).
    pub state: ChainState,
    /// Maximum distance the chain can jump between targets.
    pub range: f32,
    /// Travel speed of the arc visual between targets.
    pub arc_speed: f32,
    /// Position the chain originated from.
    pub source_pos: Vec2,
}

/// State machine for a single chain lightning arc.
#[derive(Clone)]
pub enum ChainState {
    /// Waiting to select the next target.
    Idle,
    /// Arc is traveling toward a target.
    ArcTraveling {
        target: Entity,
        target_pos: Vec2,
        arc_entity: Entity,
        arc_pos: Vec2,
    },
}
```

# Location
`src/effect_v3/effects/chain_lightning/`

# Description
`ChainLightningChain` is the primary component for a chain lightning effect entity. It drives a multi-hop damage chain that arcs between nearby cells.

- **Spawned by**: `ChainLightningConfig.fire()` — creates a new entity with `ChainLightningChain` initialized to `ChainState::Idle`, full jumps, and an empty hit set.
- **Tick**: `tick_chain_lightning` advances the state machine each frame. In `Idle`, it finds the nearest unhit cell within `range` and transitions to `ArcTraveling`. In `ArcTraveling`, it moves the arc toward the target, deals damage on arrival, adds the target to `hit_set`, decrements `remaining_jumps`, and returns to `Idle`.
- **Despawned by**: `tick_chain_lightning` — despawns the entity when `remaining_jumps == 0` or no valid target is found in `Idle`.
