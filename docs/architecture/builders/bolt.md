# Bolt Builder

`Bolt::builder()` — typestate builder for bolt entity construction.

## Why

Bolts are spawned in 5+ locations: the primary bolt spawn system, effect `fire()` functions (SpawnBolts, MirrorProtocol, TetherBeam, ChainLightning), and tests. Before the builder, each site assembled a different component tuple. Missing a component (like `CleanupOnExit<NodeState>` on extra bolts) caused silent entity lifecycle bugs.

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
- **Role**: `Primary` (persists across nodes, `CleanupOnExit<RunState>`) vs `Extra` (cleaned up on node exit, `CleanupOnExit<NodeState>`). Can't be both.
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
- `.with_lifetime_end_behavior(LifetimeEndBehavior)` — picks the lifespan-expiry dispatch: `Despawn` (emit `DespawnEntity`) or `RevertToNormalBolt` (call `PhantomBolt::become_normal`). Default (component absent) is `Despawn`. See `bolt/systems/tick_bolt_lifespan/`.
- `.phantom(PhantomParams)` — stamp the spawned bolt as a phantom. Terminal calls `Bolt::become_phantom`, inserting `PhantomBolt`, `PhantomDedupKey`, and empty `PhantomDamagedCells`. When `V == Rendered`, also inserts `PhantomFlicker::default()`. Compose with `.with_lifespan(...)` + `.with_lifetime_end_behavior(...)` for a self-expiring phantom.
- `.with_effects(Vec<(String, Tree)>)` — attach effect trees (one `(name, tree)` pair per entry — populates the bolt's `BoundEffects`)
- `.with_inherited_effects(&BoundEffects)` — clone effects from another bolt
- `.spawned_by(&str)` — evolution attribution

## build() Output

Returns `impl Bundle` with: `Bolt`, spatial components (via `Spatial::builder()`), `Velocity2D`, `Scale2D`, `PreviousScale`, `Aabb2D`, `BaseRadius` (aliased as `BoltRadius`), `MinRadius`, `MaxRadius`, `CollisionLayers`, `GameDrawLayer::Bolt`, cleanup marker (`CleanupOnExit<RunState>` or `CleanupOnExit<NodeState>`), role marker (`PrimaryBolt` or `ExtraBolt`), and optionally `Mesh2d` + `MeshMaterial2d` (if rendered).

`BoltRadius` is a type alias for `BaseRadius` from `shared/size.rs` — the same shared radius component used by all round entities.

## spawn() Behavior

The bolt builder's `spawn()` takes `&mut Commands`. Effect modules that spawn extra bolts inside `fire()` (which holds `&mut World`) bridge through a `CommandQueue`: they create `Commands::new(&mut queue, world)`, call `.spawn(&mut commands)`, then `queue.apply(world)`.

1. `commands.spawn(self.build())` — spawns core bundle
2. Conditionally inserts onto the spawned entity: `BoltLifespan`, `BoundEffects` (from `with_effects` or `with_inherited_effects`), `SpawnedByEvolution`, definition-derived params
3. Bolt-definition effects are **not** dispatched here. The separate `dispatch_bolt_effects` system processes `Added<BoltDefinitionRef>` each FixedUpdate tick and dispatches effects from the definition.

## Phantom-bolt canonical switch points

`PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, and `LifetimeEndBehavior` live at `bolt/components/phantom/inner.rs`. Two associated functions are the ONLY places the phantom component set gets inserted or stripped:

- `Bolt::become_phantom(commands, entity, dedup_key)` — inserts `PhantomBolt`, the supplied `PhantomDedupKey`, and an empty `PhantomDamagedCells`. Used by the builder terminal and by `mutators/protocols/afterimage/system/spawn_phantom_bolt.rs` (which mutates the real bolt in place).
- `PhantomBolt::become_normal(commands, entity)` — strips `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, `PhantomFlicker`, `Lifespan`, and `LifetimeEndBehavior`. Used by `tick_bolt_lifespan` on the `RevertToNormalBolt` branch.

No ad-hoc `insert((PhantomBolt, PhantomDedupKey, ...))` anywhere else in the codebase.

### Installer pattern for `Lifespan` on existing entities

Systems that install `Lifespan` onto an already-spawned entity in the SAME `FixedUpdate` pass as `tick_bolt_lifespan` (no apply-deferred boundary between them) must pre-subtract one `Time::<Fixed>::delta_secs()` from the initial `Lifespan::remaining`. The deferred-insert is invisible to `tick_bolt_lifespan` on the install tick, so without the pre-subtract the entity gets a free first tick. Callers that spawn a fresh entity carrying `Lifespan` (e.g., the chip-effect path through the builder) do NOT need this compensation because the entity itself is invisible until the next flush. See `bolt/systems/tick_bolt_lifespan/system.rs` doc comment.

## Key Files

- `breaker-game/src/bolt/builder/core/` — implementation (types, transitions, terminal)
- `breaker-game/src/bolt/components/phantom/` — `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, `LifetimeEndBehavior`, `PhantomParams`, `Bolt::become_phantom`, `PhantomBolt::become_normal`
- `breaker-game/src/bolt/systems/tick_bolt_lifespan/` — Lifespan tick + `LifetimeEndBehavior` dispatch
- `breaker-game/src/bolt/queries.rs` — QueryData structs for bolt systems
- `breaker-game/src/bolt/definition.rs` — `BoltDefinition` fields
- `breaker-game/src/shared/size.rs` — `BaseRadius`, `MinRadius`, `MaxRadius`, `effective_radius()`
- `rantzsoft_spatial2d/src/builder.rs` — nested Spatial builder used internally
- `breaker-game/assets/bolts/bolt.example.ron` — annotated reference showing all fields with defaults
