# Bolt Builder

`Bolt::builder()` — typestate builder for bolt entity construction.

## Why

Bolts are spawned in 5+ locations: the primary bolt spawn system, effect `fire()` functions (SpawnBolts, MirrorProtocol, TetherBeam, ChainLightning), and tests. Before the builder, each site assembled a different component tuple. Missing a component (like `CleanupOnNodeExit` on extra bolts) caused silent entity lifecycle bugs.

The builder ensures every bolt is complete — compile-time verification that position, speed, angle, motion, and role are all specified.

## Dimensions: `BoltBuilder<P, S, A, M, R, V>`

| Dim | Unconfigured | Configured | Transition |
|-----|-------------|-----------|------------|
| **P** (Position) | `NoPosition` | `HasPosition { pos: Vec2 }` | `.at_position(Vec2)` |
| **S** (Speed) | `NoSpeed` | `HasSpeed { base, min, max }` | `.with_speed(base, min, max)` |
| **A** (Angle) | `NoAngle` | `HasAngle { h, v }` | `.with_angle(h, v)` |
| **M** (Motion) | `NoMotion` | `Serving` / `HasVelocity` | `.serving()` / `.with_velocity(Velocity2D)` |
| **R** (Role) | `NoRole` | `Primary` / `Extra` | `.primary()` / `.extra()` |
| **V** (Visual) | `Unvisual` | `Rendered` / `Headless` | `.rendered(&mut meshes, &mut materials)` / `.headless()` |

### Mutually Exclusive

- **Motion**: `Serving` (stationary, `BoltServing` marker) vs `HasVelocity` (launched, has velocity). Can't be both.
- **Role**: `Primary` (persists across nodes, `CleanupOnRunEnd`) vs `Extra` (cleaned up on node exit, `CleanupOnNodeExit`). Can't be both.
- **Visual**: `Rendered` (includes `Mesh2d` + `MeshMaterial2d`) vs `Headless` (omits them).

### Definition Shortcut

`.definition(&BoltDefinition)` transitions **S + A** simultaneously. Also stores radius, min/max radius constraints, base damage, angle spread, spawn offset, and color in optional data.

### Overrides

After `.definition()` or after manually satisfying S/A, individual values can be overridden:
- `.with_radius(f32)` — overrides the radius set by `.definition()`
- `.with_lifespan(f32)` — adds bolt expiry timer

## Optional Methods (any typestate)

- `.with_radius(f32)` — override bolt radius (default 8.0; `BoltDefinition.radius` overrides this when `.definition()` is called)
- `.with_lifespan(f32)` — bolt expires after N seconds
- `.with_effects(Vec<(String, EffectNode)>)` — attach effect nodes
- `.with_inherited_effects(&BoundEffects)` — clone effects from another bolt
- `.spawned_by(&str)` — evolution attribution

## build() Output

Returns `impl Bundle` with: `Bolt`, spatial components (via `Spatial::builder()`), `Velocity2D`, `Scale2D`, `PreviousScale`, `Aabb2D`, `BaseRadius` (aliased as `BoltRadius`), `MinRadius`, `MaxRadius`, `CollisionLayers`, `GameDrawLayer::Bolt`, cleanup marker (`CleanupOnRunEnd` or `CleanupOnNodeExit`), role marker (`PrimaryBolt` or `ExtraBolt`), and optionally `Mesh2d` + `MeshMaterial2d` (if rendered).

`BoltRadius` is a type alias for `BaseRadius` from `shared/size.rs` — the same shared radius component used by all round entities.

## spawn() Behavior

1. `commands.spawn(self.build())`
2. Conditionally inserts via commands: `BoltLifespan`, `BoundEffects`, `SpawnedByEvolution`, config params
3. If effects provided, queues `commands.dispatch_initial_effects(effects, source_chip)` — no entity parameter, resolves targets from world by convention

## Key Files

- `breaker-game/src/bolt/builder/core.rs` — implementation
- `breaker-game/src/bolt/queries.rs` — QueryData structs for bolt systems
- `breaker-game/src/bolt/definition.rs` — `BoltDefinition` fields
- `breaker-game/src/shared/size.rs` — `BaseRadius`, `MinRadius`, `MaxRadius`, `effective_radius()`
- `rantzsoft_spatial2d/src/builder.rs` — nested Spatial builder used internally
- `breaker-game/assets/bolts/bolt.example.ron` — annotated reference showing all fields with defaults
