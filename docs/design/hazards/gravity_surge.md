# Hazard: Gravity Surge

## Game Design

Destroyed cells spawn short-lived gravity wells that pull the bolt. Wells are visible entities with a telegraphed radius. Player reads the field and compensates. Stacking increases duration (linear) and pull strength (multiplicative with diminishing returns).

## Config Resource

```rust
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct GravitySurgeConfig {
    pub base_duration_secs: f32,         // 2.0 seconds
    pub per_level_duration_secs: f32,    // 1.0 s per additional stack
    pub base_strength: f32,              // pull force magnitude at stack 1
    pub per_level_strength_frac: f32,    // fractional scaling per stack (e.g., 0.5)
}

// Distance clamp floor is a module-level const: MIN_PULL_DISTANCE = 20.0
```

**Multiplicative strength formula**: `strength_for_stacks(N) = base_strength * (1.0 + per_level_strength_frac * sqrt(N - 1))`.

At stack 1 the multiplier is 1.0 (strength = base); at stack 3 with `per_level_strength_frac = 0.5`, multiplier is `1 + 0.5 * sqrt(2) ≈ 1.707`. Square root provides natural diminishing returns on strength without capping duration scaling.

| Stack | Duration | Strength multiplier |
|-------|----------|---------------------|
| 1 | 2.0s | 1.000× |
| 2 | 3.0s | 1.500× (frac 0.5) |
| 3 | 4.0s | 1.707× |
| 5 | 6.0s | 2.000× |

Populated from `HazardTuning::GravitySurge`.

## Components

```rust
#[derive(Component, Debug)]
pub(crate) struct GravityWell {
    pub strength: f32,
    pub remaining: f32,
}
```

Gravity-well entity has a `Position2D` at the destroyed cell's world position and a `GravityWell` component, plus `CleanupOnExit::<NodeState>` to ensure stale wells don't leak across nodes. The `fx` domain reads `GravityWell` to render the telegraph; no sprite lives on the core component.

## Messages
**Reads**: `Destroyed<Cell>` (from `rantzsoft_dmg`).
**Sends**: `ApplyBoltForce { bolt: Entity, force: Vec2 }` per bolt per tick — one pre-summed message per bolt (aggregating all active wells). Bolt-domain consumer (`apply_bolt_forces` in `BoltSystems::ApplyForces`) multiplies by `dt` and writes to `Velocity2D` before `SpatialSystems::ApplyVelocity`. Gravity Surge does NOT write `Velocity2D` directly.

## Systems

### `spawn_gravity_wells`
- **Schedule**: `FixedUpdate`, `.before(gravity_well_pull)` (no `.run_if` at the registration level — gated in-body to avoid message buffering leaks).
- **run_if**: In-body check — drains via `reader.clear()` and returns early when `GravitySurge` is not active, `NodeState` is not `Playing`, `GravitySurgeConfig` is absent, or computed duration/strength is zero.
- **Behavior**: Reads `Destroyed<Cell>`. For each victim, computes `duration = config.duration_secs(stacks)` and `strength = config.strength(stacks)`. Spawns a new entity at the victim's `victim_pos` with `GravityWell { strength, remaining: duration }` + `Position2D` + `CleanupOnExit::<NodeState>`.

### `gravity_well_pull`
- **Schedule**: `FixedUpdate`, `.before(BoltSystems::ApplyForces)` (from chained tuple with `despawn_expired_gravity_wells`).
- **run_if**: `hazard_active(HazardKind::GravitySurge)` + `in_state(NodeState::Playing)` (from chained tuple).
- **Behavior**: Ticks each `GravityWell.remaining -= delta_secs`; snapshots only still-live wells (`remaining > 0.0`) into a local buffer. For each bolt, sums `force = sum_over_live_wells(direction * strength / distance.max(MIN_PULL_DISTANCE))` then emits ONE `ApplyBoltForce { bolt, force }` with the aggregate acceleration (no `dt` multiply — consumer handles that). Wells that expired this tick contribute no force.

### `despawn_expired_gravity_wells`
- **Schedule**: `FixedUpdate`, `.after(gravity_well_pull)` (enforced by `.chain()` in the same tuple as `gravity_well_pull`).
- **run_if**: `hazard_active(HazardKind::GravitySurge)` + `in_state(NodeState::Playing)` (from chained tuple).
- **Behavior**: Despawns entities with `GravityWell.remaining <= 0.0`.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Gravity Surge does not participate in any `DeathPipelineSystems` set.
- **Trigger**: reads `Destroyed<Cell>` from `rantzsoft_dmg` — every cell death spawns a gravity-well entity at the killed cell's position with lifetime + strength per stack.
- **Force emission**: emits `ApplyBoltForce { bolt, force: Vec2 }` (one per bolt, pre-summed across all wells) in `FixedUpdate` before `BoltSystems::ApplyForces`. Bolt-domain consumer applies `force * dt` to `Velocity2D` before `SpatialSystems::ApplyVelocity` — NOT part of the damage pipeline.
- **No** `DamageBoostStack` / `VulnerableStack` interaction. **No** direct `Velocity2D` write.

## Stacking Behavior

Linear duration, multiplicative-diminishing strength. `duration = base_duration + duration_per_level * (stack - 1)`; `strength = base_strength * (1.0 + per_level_strength_frac * sqrt(stack - 1))`.

The real danger at high stacks is well overlap. With 4+ second durations, destroying a cluster creates a field of overlapping wells whose forces compound. `min_distance` clamp prevents infinite force at zero distance.

## Cross-Domain Dependencies
- **cells / damage crate**: Reads `Destroyed<Cell>`.
- **bolt**: Consumes `ApplyBoltForce` via `apply_bolt_forces` in `BoltSystems::ApplyForces`. Owns force-aggregation + `Velocity2D` write. `ApplyBoltForce` is shared with Drift — the bolt consumer sums forces across sources.
- **fx**: Reads `GravityWell` to render the telegraph.

## Expected Behaviors (for test specs)

1. **Gravity well spawns on cell destruction at stack 1** — cell at world (100, 200) destroyed: entity spawned at (100, 200) with `GravityWell { strength: base_strength, remaining: 2.0 }`.
2. **Strength at stack 1 equals base** — `base_strength: 100.0`, any `per_level_strength_frac`, stack 1: `strength_for_stacks(1) = 100.0 * (1.0 + frac * sqrt(0)) = 100.0`.
3. **Strength scales multiplicatively at stack 3** — `base_strength: 100.0`, `per_level_strength_frac: 0.5`, stack 3: `strength_for_stacks(3) = 100.0 * (1.0 + 0.5 * sqrt(2)) ≈ 170.7`.
4. **Duration scales linearly at stack 3** — stack 3, `base_duration: 2.0`, `duration_per_level: 1.0`: `remaining = 4.0`.
5. **Bolt pulled toward well** — well at (100, 200) strength 500, bolt at (150, 200): `ApplyBoltForce` points in `-x` direction.
6. **Well despawns after duration expires** — `remaining: 0.1`, `delta_secs: 0.2`: entity despawned.
7. **Multiple wells compound forces** — two wells equidistant from bolt: X-components cancel, net force is the vector sum (zero X if symmetric).

## Edge Cases
- **Gravity Surge + Drift**: both emit `ApplyBoltForce`. Bolt consumer sums forces — player compensates for both simultaneously. Intentional readable-but-demanding synergy.
- **Force clamping**: `distance.max(min_distance)` prevents infinite force at near-zero distance (e.g., bolt passes through the well's center).
- **Many simultaneous wells**: destroying a large cluster spawns many wells; diminishing-returns on per-well strength helps but total force can still be extreme. Flag for playtesting.
- **Wells at node boundary**: `ApplyBoltForce` is an additive force, not a teleport — the bolt's boundary clamping keeps the bolt in-bounds.
- **Ghost cells (Echo Cells) destroyed**: also trigger gravity wells (the system reads `Destroyed<Cell>` generically; no type discrimination).
- **Cleanup**: `GravityWell` entities despawned at node end via cleanup markers. `GravitySurgeConfig` removed at run end.
