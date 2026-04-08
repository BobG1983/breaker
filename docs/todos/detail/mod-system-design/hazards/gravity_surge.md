# Hazard: Gravity Surge

## Game Design

Destroyed cells spawn short-lived gravity wells that pull the bolt. The wells are visible entities with a telegraphed radius. The player must read the gravity field and compensate. Stacking increases duration and pull strength (with diminishing returns on strength to prevent unplayable force values).

**Stacking formula**:
- Duration: `2.0s + 1.0s * (stack - 1)`
- Pull strength: Base strength + `strength_per_level_diminishing` per stack (diminishing returns, e.g., each stack adds less than the previous)

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct GravitySurgeConfig {
    pub base_duration: f32,                    // 2.0 seconds
    pub duration_per_level: f32,               // 1.0 seconds per additional stack
    pub base_strength: f32,                    // pull force magnitude (tuning TBD)
    pub strength_per_level_diminishing: f32,   // additional strength per stack (diminishing)
}
```

**Diminishing returns on strength**: `effective_strength = base_strength + strength_per_level_diminishing * sqrt(stack - 1)`. Square root provides natural diminishing returns.

| Stack | Duration | Strength multiplier (approx) |
|-------|----------|------------------------------|
| 1     | 2.0s     | 1.0x base                    |
| 2     | 3.0s     | 1.0x + 1.0x dim              |
| 3     | 4.0s     | 1.0x + 1.41x dim             |
| 5     | 6.0s     | 1.0x + 2.0x dim              |

## Components

```rust
/// A gravity well entity spawned when a cell is destroyed under Gravity Surge.
#[derive(Component, Debug)]
pub(crate) struct GravityWell {
    pub strength: f32,
    pub remaining: f32,
}
```

The gravity well is its own entity with a position (at the destroyed cell's location) and a `GravityWell` component. It has no sprite of its own -- the `fx` domain reads `GravityWell` components to render visual effects.

## Messages

**Reads**: `CellDestroyedAt` (or equivalent cell death event with position)
**Sends**: `ApplyBoltForce { bolt: Entity, force: Vec2 }`

## Systems

1. **`spawn_gravity_wells`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::GravitySurge)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_damage` / cell death processing
   - Behavior:
     1. Read `CellDestroyedAt` messages
     2. For each destroyed cell, compute duration and strength from config + stack
     3. Spawn a new entity at the cell's position with `GravityWell { strength, remaining: duration }` and a `Transform`

2. **`gravity_well_pull`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::GravitySurge)` AND `in_state(NodeState::Playing)`
   - Ordering: Before bolt movement / physics step
   - Behavior:
     1. For each `GravityWell` entity, tick down `remaining` by `delta_secs`
     2. For each bolt entity, compute displacement vector from bolt to well
     3. Compute pull force: `direction * strength / distance.max(min_distance)` (inverse-linear or inverse-square, tuning TBD; clamped to prevent infinite force at zero distance)
     4. Sum forces from all active wells
     5. Send `ApplyBoltForce { bolt, force: total_force }` per bolt

3. **`despawn_expired_gravity_wells`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::GravitySurge)` AND `in_state(NodeState::Playing)`
   - Ordering: After `gravity_well_pull`
   - Behavior: Despawn any `GravityWell` entity where `remaining <= 0.0`

## Stacking Behavior

| Stack | Duration | Strength | Wells per node (rough, 20 cells) |
|-------|----------|----------|----------------------------------|
| 1     | 2.0s     | base     | Up to 20 (one per destroyed cell)|
| 2     | 3.0s     | ~1.0x dim stronger | Same count, last longer + pull harder |
| 3     | 4.0s     | ~1.41x dim stronger | At 4s duration, wells overlap -- multiple active simultaneously |

The real danger at high stacks is well overlap. With 4+ second durations, destroying a cluster of cells creates a field of overlapping gravity wells that compound their pull forces.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell destruction events | `CellDestroyedAt` message (read) |
| `bolt`  | Applies force to bolts | `ApplyBoltForce` message (send) |

**Note**: `ApplyBoltForce` is shared with Drift hazard. The `bolt` domain provides a single system that processes all `ApplyBoltForce` messages regardless of source.

## Expected Behaviors (for test specs)

1. **Gravity well spawns on cell destruction at stack=1**
   - Given: Cell at position (100.0, 200.0) is destroyed, stack=1
   - When: `spawn_gravity_wells` runs
   - Then: New entity at (100.0, 200.0) with `GravityWell { strength: base, remaining: 2.0 }`

2. **Gravity well pulls bolt toward itself**
   - Given: Gravity well at (100.0, 200.0), bolt at (150.0, 200.0), strength = 500.0
   - When: `gravity_well_pull` runs
   - Then: `ApplyBoltForce` sent with force pointing from bolt toward well (negative X direction)

3. **Duration scales with stack count at stack=3**
   - Given: Stack=3, cell destroyed
   - When: `spawn_gravity_wells` runs
   - Then: `GravityWell.remaining = 4.0` (2.0 + 1.0 * 2)

4. **Well despawns after duration expires**
   - Given: `GravityWell { remaining: 0.1 }`, delta_secs = 0.2
   - When: `despawn_expired_gravity_wells` runs
   - Then: Entity is despawned

5. **Multiple wells compound forces**
   - Given: Two gravity wells at (100.0, 200.0) and (200.0, 200.0), bolt at (150.0, 200.0)
   - When: `gravity_well_pull` runs
   - Then: Forces from both wells sum (cancel in X if equidistant, or compound in other directions)

## Edge Cases

- **Gravity Surge + Drift synergy**: Both apply `ApplyBoltForce`. The bolt domain sums all forces. The player must compensate for both wind direction and gravity pulls simultaneously. This is intentional -- readable but demanding.
- **Force clamping**: Pull force must be clamped at close range to prevent the bolt from being trapped in a gravity well (infinite force at zero distance). Use a minimum distance floor (e.g., 10.0 units).
- **Many simultaneous wells**: Destroying a large cluster could spawn 10+ wells simultaneously. Performance is not a concern (simple per-well force computation), but the combined pull could be extreme. The diminishing returns on strength helps, but a total force cap per frame may be needed for playability. Flag for playtesting.
- **Wells at node boundary**: Wells should not pull the bolt out of bounds. The bolt domain's boundary clamping handles this -- `ApplyBoltForce` is a suggestion, not a teleport.
- **Echo Cells**: When ghost cells (from Echo Cells hazard) are destroyed, they should also spawn gravity wells. The system reads `CellDestroyedAt` generically -- it doesn't distinguish cell types.
- **Cleanup**: Gravity well entities should be despawned when the node ends (via standard entity cleanup). `GravitySurgeConfig` removed at run end.
