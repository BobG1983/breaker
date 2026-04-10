# Shockwave

## Config
```rust
struct ShockwaveConfig {
    /// Base radius before stacking
    base_range: f32,
    /// Extra radius per stack beyond the first
    range_per_level: f32,
    /// Current stack count
    stacks: u32,
    /// Expansion speed in world units/sec
    speed: f32,
}
```
**RON**: `Shockwave(base_range: 64.0, range_per_level: 8.0, stacks: 1, speed: 500.0)`

Effective range: `base_range + range_per_level * (stacks - 1).max(0)`

## Reversible: NO (spawns entity — no-op reverse)

## Target: Any entity with position (spawns shockwave at entity's position — could be bolt, breaker, cell, etc.)

## Spawned Entity Components
```rust
ShockwaveSource           // marker
ShockwaveRadius(0.0)      // current radius, starts at 0
ShockwaveMaxRadius(range) // effective range
ShockwaveSpeed(speed)     // expansion speed
ShockwaveDamaged(HashSet) // tracks which cells already damaged (at-most-once)
ShockwaveDamageMultiplier // snapshotted from source's ActiveDamageBoosts
ShockwaveBaseDamage       // snapshotted from source's BoltBaseDamage
EffectSourceChip          // chip attribution for damage
Spatial (at source pos)   // position
Scale2D { x: 0, y: 0 }   // visual scale (synced to radius)
GameDrawLayer::Fx         // draw layer
CleanupOnExit<NodeState>  // cleaned up on node end
Mesh2d + MeshMaterial2d   // visual (circle mesh, orange HDR color)
```

## Fire
1. Calculate effective range from base_range, range_per_level, stacks
2. Read source entity's position, damage multiplier, base damage
3. Spawn shockwave entity with all components above

## Reverse
No-op — shockwave entities live independently once spawned.

## Runtime Systems (4, chained in FixedUpdate)
1. **tick_shockwave**: expand radius by `speed * dt`
2. **sync_shockwave_visual**: set `Scale2D` to match radius
3. **apply_shockwave_damage**: query quadtree for cells within radius, send `DamageDealt<Cell>` for cells not in `ShockwaveDamaged` set (at-most-once per cell per shockwave)
4. **despawn_finished_shockwave**: despawn when radius >= max_radius

## Messages Sent
- `DamageDealt<Cell> { dealer: Some(source_bolt), target: cell, amount: damage, source_chip }` — for each cell first entering the shockwave radius

## Notes
- Damage = `base_damage * damage_multiplier` (snapshotted at fire time, not live)
- Uses quadtree circle query for spatial cell lookup
- At-most-once damage per cell per shockwave instance via HashSet tracking
