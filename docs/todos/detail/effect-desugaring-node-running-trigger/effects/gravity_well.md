# GravityWell

## Config
```rust
struct GravityWellConfig {
    /// Pull strength applied to bolts within radius
    strength: f32,
    /// Duration in seconds before self-despawn
    duration: f32,
    /// Attraction radius in world units
    radius: f32,
    /// Maximum active wells per owner entity
    max: u32,
}
```
**RON**: `GravityWell(strength: 100.0, duration: 5.0, radius: 80.0, max: 3)`

## Reversible: NO (spawns entity — no-op reverse, wells self-despawn via timer)

## Target: Any entity with position (typically bolt)

## Fire
1. Guard: if entity despawned or max == 0 → return
2. Read source entity's position
3. If owner already has >= max wells: despawn oldest (FIFO by `GravityWellSpawnOrder`)
4. Spawn well entity at source position with all config + visual components
5. Increment per-owner spawn counter in `GravityWellSpawnCounter` resource

## Well Entity Components
```rust
GravityWell              // marker
GravityWellConfig { strength, radius, remaining, owner }
GravityWellSpawnOrder(u64) // FIFO ordering for max-cap despawn
Spatial (at source pos)
Scale2D { x: radius, y: radius }
GameDrawLayer::Fx
CleanupOnExit<NodeState>
Mesh2d + MeshMaterial2d  // purple HDR circle
```

## Runtime Systems (3, chained in FixedUpdate)
1. **tick_gravity_well**: decrement `remaining` by dt, despawn when <= 0
2. **sync_gravity_well_visual**: sync `Scale2D` to radius
3. **apply_gravity_pull**: for each well, for each bolt within radius: direction-only steering with magnitude falloff: `let to_well = (well_pos - bolt_pos).normalize_or_zero(); let speed = velocity.length(); let steered = velocity + to_well * strength * dt; velocity = steered.normalize() * speed;` Uses existing velocity helper function. Bolt speed preserved, only direction bends toward well. Strength falls off with distance.

## Resource
```rust
#[derive(Resource, Default)]
struct GravityWellSpawnCounter(HashMap<Entity, u64>);
```
Monotonically increasing per-owner counter. Enables deterministic FIFO despawn when max cap is exceeded.

## Notes
- Wells pull all bolts within radius, not just the owner's bolts
- Strength falls off with distance from well center
- Wells from the same owner share a max cap — oldest is despawned first
- Bolts in `Birthing` state are excluded from pull (query filter)
