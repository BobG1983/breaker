# Name
ChainLightning

# Enum Variant
- `EffectType::ChainLightning(ChainLightningConfig)`

# Config
`ChainLightningConfig { arcs: u32, range: f32, damage_mult: f32, arc_speed: f32 }`

# Fire
1. Read the source entity's position.
2. Snapshot damage as `damage_mult * BoltBaseDamage * EffectStack<DamageBoostConfig>.aggregate()`.
3. Query the quadtree for cells within `range` of the source position.
4. Pick a random target from the results.
5. Send `DamageDealt<Cell>` for the first target immediately.
6. If `arcs > 1`, spawn a `ChainLightningChain` entity with `remaining_jumps: arcs - 1`, the snapshotted damage, a `hit_set` containing the first target, `state: Idle`, `range`, `arc_speed`, `source_pos` set to the first target's position, and `CleanupOnExit<NodeState>`.
7. Fire does NOT handle subsequent arcs -- the `tick_chain_lightning` system does.
8. Fire does NOT pick all targets at once -- targets are picked one at a time as arcs travel.

# Reverse
Not reversible.

# Source Location
`src/effect_v3/effects/chain_lightning/config.rs`

# New Types
- `ChainLightningChain` -- component tracking the chain state:
  - `remaining_jumps: u32`
  - `damage: f32`
  - `hit_set: HashSet<Entity>`
  - `state: ChainState`
  - `range: f32`
  - `arc_speed: f32`
  - `source_pos: Vec2`
- `ChainState` -- enum:
  - `Idle` -- ready to pick next target
  - `ArcTraveling { target: Entity, target_pos: Vec2, arc_entity: Entity, arc_pos: Vec2 }` -- arc in flight

# New Systems

## tick_chain_lightning
- **What it does**: State machine per `ChainLightningChain` entity.
  - `Idle`: query quadtree for cells within `range` of `source_pos` that are not in `hit_set`. Pick a random valid target. Spawn an arc VFX entity at `source_pos`. Transition to `ArcTraveling` with the target, target position, arc entity, and arc entity position.
  - `ArcTraveling`: advance `arc_pos` toward `target_pos` by `arc_speed * dt`. When `arc_pos` reaches `target_pos`: send `DamageDealt<Cell>` for the target, add target to `hit_set`, set `source_pos` to `target_pos`, decrement `remaining_jumps`, despawn the arc VFX entity, transition to `Idle`.
  - If `Idle` finds no valid targets, or `remaining_jumps == 0`: despawn the chain entity and any active arc VFX entity.
- **What it does NOT do**: Does not handle the first arc (fire does that). Does not deal damage directly -- sends the message.
- **Schedule**: FixedUpdate, in `EffectV3Systems::Tick`, with `run_if(in_state(NodeState::Playing))`.
