# SpawnBolts

## Config
```rust
struct SpawnBoltsConfig {
    /// Number of bolts to spawn (default 1)
    count: u32,
    /// Optional lifespan in seconds (None = permanent)
    lifespan: Option<f32>,
    /// If true, spawned bolts inherit parent's BoundEffects
    inherit: bool,
}
```
**RON**: `SpawnBolts(count: 2, lifespan: Some(5.0), inherit: false)`

## Reversible: NO (spawned bolts live independently — no-op reverse)

## Target: Bolt (spawns from bolt's position using bolt's definition)

## Fire
1. Read source entity's position
2. Read source's `BoltDefinitionRef` → look up in `BoltRegistry` (fallback to "Bolt" default)
3. If inherit: clone `BoundEffects` from first primary bolt (non-ExtraBolt)
4. For each bolt (count times):
   - Generate random angle via `GameRng`
   - Calculate velocity: `direction * bolt_def.base_speed`
   - Spawn via `Bolt::builder().at_position().definition().with_velocity().extra().headless().birthed()`
   - Insert visual components (mesh + material)
   - If lifespan: insert `BoltLifespan(Timer)`
   - If inherit: insert cloned `BoundEffects`

## Reverse
No-op — spawned bolts persist independently.

## Notes
- Spawned bolts are tagged `ExtraBolt` (not primary)
- Spawned bolts go through `birthed()` animation
- Random velocity direction — each bolt goes a different way
- BoltDefinition lookup chain: source entity's def → fallback to "Bolt" → warn if missing
- inherit clones from first primary bolt's BoundEffects, not the source entity directly
