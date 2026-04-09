# MirrorProtocol

## Config
```rust
struct MirrorConfig {
    /// If true, spawned bolt inherits parent's BoundEffects
    inherit: bool,
}
```
**RON**: `MirrorProtocol(inherit: false)`

## Reversible: NO (spawns entity — no-op reverse)

## Target: Bolt (spawns mirrored bolt)

## Fire
1. Read source entity's position, velocity, and bolt definition
2. Spawn new bolt at same position with velocity reflected across the last impact surface normal
3. Tag as `ExtraBolt`
4. If inherit: clone source's `BoundEffects` onto new bolt

## Reverse
No-op — mirrored bolt lives independently.

## Notes
- Creates a "mirror image" bolt traveling in the reflected direction
- Requires impact surface normal from TriggerContext to compute reflection
- Spawned bolt uses same definition as source
