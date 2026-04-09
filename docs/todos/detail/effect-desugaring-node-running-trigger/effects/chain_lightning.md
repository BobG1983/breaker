# ChainLightning

## Config
```rust
struct ChainLightningConfig {
    /// Number of jumps
    arcs: u32,
    /// Maximum jump distance between targets
    range: f32,
    /// Damage multiplier per arc (applied to base_damage * damage_boost)
    damage_mult: f32,
    /// Arc travel speed in world units per second (default 200.0)
    arc_speed: f32,
}
```
**RON**: `ChainLightning(arcs: 3, range: 80.0, damage_mult: 0.5, arc_speed: 200.0)`

## Reversible: NO (spawns entities — no-op reverse)

## Target: Bolt (spawns chain from bolt's position)

## Fire
1. Guard: if arcs == 0, range <= 0, or arc_speed <= 0 → return
2. Read source entity's position, damage multiplier, base damage
3. Calculate per-arc damage: `base_damage * damage_mult * effective_damage_multiplier`
4. Query quadtree for cells within range of source position
5. Pick random target from candidates
6. Send `DamageDealt<Cell>` for first target immediately
7. If arcs > 1: spawn `ChainLightningChain` entity to handle remaining jumps

## Chain Entity Components
```rust
#[derive(Component)]
struct ChainLightningChain {
    source: Vec2,           // position of last-hit target
    remaining_jumps: u32,   // jumps left
    damage: f32,            // pre-computed damage per hit
    hit_set: HashSet<Entity>, // already-hit cells (excluded from targeting)
    state: ChainState,      // Idle or ArcTraveling
    range: f32,             // max jump distance
    arc_speed: f32,         // arc travel speed
}

enum ChainState {
    Idle,                   // ready to pick next target
    ArcTraveling {          // arc moving toward target
        target: Entity,
        target_pos: Vec2,
        arc_entity: Entity,
        arc_pos: Vec2,
    },
}
```

## Runtime System: `tick_chain_lightning`
**Schedule**: FixedUpdate, after PhysicsSystems::MaintainQuadtree, run_if NodeState::Playing

State machine per chain entity:
1. **Idle**: query quadtree for valid targets (in range, not in hit_set). Pick random. Spawn arc entity at source position. Transition to ArcTraveling.
2. **ArcTraveling**: advance arc toward target by `arc_speed * dt`. When arc reaches target: send `DamageDealt<Cell>`, add target to hit_set, update source to target position, decrement remaining_jumps, despawn arc entity. If jumps remain → back to Idle. If no jumps → despawn chain entity.
3. **No valid targets found**: despawn chain entity (chain ends early)

## Messages Sent
- `DamageDealt<Cell> { dealer: Some(source_bolt), target: cell, amount: damage, source_chip }` — once per arc hit

## Notes
- Sequential arc-based chaining — each arc visually travels between cells over time
- First hit is instant (no arc entity), subsequent hits travel
- Each cell can only be hit once per chain (hit_set tracking)
- Random target selection from valid candidates within range
- Arc visual: small circle mesh, purple HDR color
