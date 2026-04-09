# SpawnPhantom

## Config
```rust
struct SpawnPhantomConfig {
    /// Lifespan in seconds
    duration: f32,
    /// Maximum phantoms alive at once
    max_active: u32,
}
```
**RON**: `SpawnPhantom(duration: 3.0, max_active: 2)`

## Reversible: NO (spawns entity — no-op reverse)

## Target: Bolt (spawns phantom from bolt's position)

## Components (on spawned phantom)
```rust
#[derive(Component)]
struct PhantomBoltMarker;  // identifies as phantom

#[derive(Component)]
struct PhantomOwner(Entity);  // points to source entity for cap counting

#[derive(Component)]
struct PhantomSpawnOrder(u64);  // FIFO ordering for max-cap despawn
```

## Fire
1. Count existing phantoms with `PhantomOwner` matching source entity
2. If count >= max_active → despawn oldest (FIFO by `PhantomSpawnOrder`)
3. Read source entity's position and bolt definition
4. Spawn phantom bolt with `PiercingRemaining(u32::MAX)` (infinite piercing), random velocity
5. Insert `BoltLifespan(Timer)` for timed auto-despawn
6. Tag with `PhantomBoltMarker`, `PhantomOwner`, `PhantomSpawnOrder`

## Reverse
No-op — phantom bolts self-despawn via lifespan timer.

## Notes
- Phantom bolts have infinite piercing via `PiercingRemaining(u32::MAX)` — they pass through all cells
- Timed lifespan — auto-despawn after duration
- Max cap enforced per-owner via FIFO despawn (oldest phantom removed when cap exceeded)
- Phantom bolts are visually distinct (implementation determines appearance)
