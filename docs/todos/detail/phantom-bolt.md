# Phantom bolt — builder `.phantom()`, mutate-real-bolt semantics, collision cleanup

## Problem

Phantom bolts today are broken across multiple axes:

- **Two divergent spawn paths.** Afterimage protocol spawns phantom bolts via `Bolt::builder().extra().headless().spawn(...)` (invisible in production) with post-spawn `PhantomBolt` marker insertion. Chip-effect `SpawnPhantomConfig::fire` calls raw `world.spawn(...)` bypassing the builder entirely. Neither path is the intended design.
- **Afterimage spawns a new bolt instead of mutating the real one.** Design intent per `audit/protocols/afterimage.md` Issue 3: on Perfect bump against a phantom breaker, the REAL bolt becomes Phantom (for a duration, then reverts). Current impl spawns a SECOND bolt alongside the real one. A regression-trap test at `tests/spawn_phantom_bolt.rs:124-125` actively pins the wrong behavior ("real bolt must NOT gain PhantomBolt marker").
- **No visual distinction.** `.headless()` makes the afterimage phantom invisible in production.
- **Lifespan dispatch bug.** Both sites call `despawn()` on expiry via a shared `tick_phantom_lifetime` system. That's wrong for afterimage — afterimage's design intent is revert-to-normal-bolt, not despawn. The chip-effect path correctly wants despawn.
- **Post-build marker insertion is error-prone.** Neither path has a single authoritative place where the phantom component set gets inserted.
- **Potential phantom-specific branches in wall/breaker collision** — design intent is "phantom is a normal bolt for every surface except cells," but wall/breaker collision systems may carry stale phantom branches that violate this.

## Scope

**In scope: phantom bolts.** This TODO consolidates three remediation files and delivers the full bolt-side phantom refactor. Reuses shared infra (`Lifespan`, `PhantomFlicker`, `tick_phantom_flicker`) introduced by TODO #4.

Out of scope:
- Phantom breakers (TODO #4).
- Phase 5 polished phantom visual (placeholder flicker only).
- Cross-cutting placeholder VFX for non-phantom items (drift arrow, echo ghost tint, etc. — tracked separately in `placeholder-vfx-for-deferred-items.md`).

## Design

### Phantom-bolt contract

A phantom bolt:

- **Bounces normally off walls.** Same reflection as a non-phantom bolt. NO `With<PhantomBolt>` branch in `bolt_wall_collision`.
- **Bounces normally off real breakers.** Same reflection, same `BoltImpactBreaker` emission, same bump-grading. NO `With<PhantomBolt>` branch in the real-breaker path of `bolt_breaker_collision`.
- **Bounces normally off phantom breakers.** Shared `bolt_breaker_collision` path; the phantom-breaker-specific branches (skip tilt, skip spread override, skip last-impact, skip piercing) are per-phantom-*breaker*, not per-phantom-*bolt*.
- **Passes through cells while damaging them.** THE ONLY phantom-bolt-specific collision branch. When a `With<PhantomBolt>` bolt collides with a cell: emit `DamageDealt<Cell>` once per cell (deduped via `PhantomDamagedCells` hashset component), do NOT flip velocity.
- **May have a lifespan.** Optional via `.with_lifespan(seconds)` on the builder (not baked into `.phantom()`).
- **Has a `LifetimeEndBehavior` component** when a lifespan is set. Two variants:
  - `Despawn` — chip-effect phantom uses this. Entity emits `DespawnEntity` to the death pipeline on expiry.
  - `RevertToNormalBolt` — afterimage phantom uses this. On expiry, phantom markers are stripped; the bolt continues as a normal bolt (lifespan, role, velocity, position, effects, cleanup markers preserved).
- **Is visually distinct when rendered.** Tint + `PhantomFlicker` placeholder. Headless phantoms skip visual components.
- **Role is orthogonal to phantom-ness.** `.phantom()` does NOT force `.primary()` or `.extra()` and does NOT insert `CleanupOnExit<...>`. The caller picks the role via the existing `.primary()`/`.extra()` transition. A phantom primary bolt IS permitted.
- **Has no collision-kind variants.** No `PhantomCollisionKind` enum. Every phantom bolt behaves identically at the collision layer.

### Two spawn paths, two semantics

**Path 1 — Afterimage: mutate the real bolt.** On Perfect bump against a phantom breaker, the REAL bolt becomes Phantom in place. No new entity. Afterimage's `spawn_phantom_bolt` (misleadingly named today) calls:

```rust
Bolt::become_phantom(&mut commands, real_bolt, PhantomDedupKey::Bolt(real_bolt));
commands.entity(real_bolt).insert((
    Lifespan { remaining: config.phantom_duration },
    LifetimeEndBehavior::RevertToNormalBolt,
));
// If Rendered (afterimage-spawned bolts always are), add flicker:
commands.entity(real_bolt).insert(PhantomFlicker::default());
```

"Duration does NOT reset" guard: if `real_bolt` already carries `PhantomBolt`, the whole block is skipped (no-op). With mutate-in-place, the dedup collapses to a `Has<PhantomBolt>` check on the entity — `PhantomDedupKey::Bolt(_)` is kept as informational attribution but is NOT load-bearing for the afterimage uniqueness rule.

On lifespan expiry, the tick system reads `LifetimeEndBehavior::RevertToNormalBolt` and calls `PhantomBolt::become_normal(&mut commands, entity)` — phantom markers stripped, bolt continues.

**Path 2 — Chip-effect: spawn a new bolt via builder.** `SpawnPhantomConfig::fire` legitimately creates a temporary phantom bolt (it's a chip that says "spawn a phantom"). Migrates to the builder:

```rust
Bolt::builder()
    .at_position(pos)
    .with_speed(base_speed, min_speed, max_speed)
    .with_angle(angle_min, angle_max)
    .with_velocity(Velocity2D(vel))
    .with_radius(radius)
    .with_base_damage(base_damage)
    .with_lifespan(config.phantom_bolt_duration)
    .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
    .phantom(PhantomParams {
        dedup_key: PhantomDedupKey::Chip {
            chip: source.to_string(),
            fired_from: entity,
        },
    })
    .extra()
    .rendered(&mut meshes, &mut materials)
    .spawn(&mut commands);
```

The `Fireable::fire` signature adapts to pass `Commands` + `ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>`. `source` (the chip id/name, currently an unused `_source` parameter) is promoted to a load-bearing dedup input. The `max_active` filter in `SpawnPhantomConfig::fire` counts phantoms whose `PhantomDedupKey::Chip { chip, fired_from }` matches this spawn's pair.

### Builder API

`.phantom(PhantomParams)` and `.with_lifetime_end_behavior(LifetimeEndBehavior)` are **optional chainable methods** on the bolt builder alongside `with_lifespan`, `with_radius`, `with_base_damage`. Work on any typestate; stash data in `OptionalBoltData` that the terminal `spawn_inner` consumes.

```rust
// breaker-game/src/bolt/builder/core/types.rs
pub(in crate::bolt::builder) struct OptionalBoltData {
    // ... existing fields ...
    pub(in crate::bolt::builder) phantom: Option<PhantomParams>,
    pub(in crate::bolt::builder) lifetime_end_behavior: Option<LifetimeEndBehavior>,
}

#[derive(Debug, Clone)]
pub struct PhantomParams {
    /// Identifier used to dedup phantom spawns per source.
    pub dedup_key: PhantomDedupKey,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhantomDedupKey {
    /// Afterimage phantom — spawned from a Perfect Bump on this real bolt.
    /// Informational under mutate-real-bolt; the actual dedup is a
    /// `Has<PhantomBolt>` check on the entity itself.
    Bolt(Entity),
    /// Chip-effect phantom — spawned by `SpawnPhantomConfig::fire`.
    /// `(chip, fired_from)` pair is the load-bearing dedup — at most
    /// `max_active` live phantoms per pair.
    Chip { chip: String, fired_from: Entity },
}

/// Behavior when a bolt's `Lifespan` reaches 0. Bolt-only enum
/// (phantom breakers unconditionally despawn).
#[derive(Component, Debug, Clone, Copy)]
pub enum LifetimeEndBehavior {
    /// Emit `DespawnEntity` to the death pipeline.
    Despawn,
    /// Strip `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`,
    /// `PhantomFlicker`, `Lifespan`, and `LifetimeEndBehavior`. Bolt continues.
    RevertToNormalBolt,
}
```

`PhantomDedupKey` is no longer `Copy` (the `Chip` variant owns a `String`). Most usages match-and-compare against a `&PhantomDedupKey` from a query; the component is cloned only at spawn.

```rust
// breaker-game/src/bolt/builder/core/transitions.rs
impl<P, S, A, M, R, V> BoltBuilder<P, S, A, M, R, V> {
    #[must_use]
    pub const fn phantom(mut self, params: PhantomParams) -> Self {
        self.optional.phantom = Some(params);
        self
    }

    #[must_use]
    pub const fn with_lifetime_end_behavior(mut self, behavior: LifetimeEndBehavior) -> Self {
        self.optional.lifetime_end_behavior = Some(behavior);
        self
    }
}
```

### Canonical phantom-insertion API

Two associated functions on the marker components make the flip explicit and reversible. Both are the ONLY places the phantom component set gets inserted or stripped:

```rust
impl Bolt {
    /// Convert this bolt into a phantom in place. Inserts `PhantomBolt`,
    /// the provided `PhantomDedupKey`, and `PhantomDamagedCells::default()`.
    /// Does NOT insert `PhantomFlicker` — the caller adds it for rendered bolts
    /// (the builder terminal does this automatically when `visual == Rendered`).
    pub fn become_phantom(
        commands: &mut Commands,
        bolt: Entity,
        dedup_key: PhantomDedupKey,
    ) { ... }
}

impl PhantomBolt {
    /// Convert this phantom bolt back to normal. Strips `PhantomBolt`,
    /// `PhantomDedupKey`, `PhantomDamagedCells`, `PhantomFlicker`, `Lifespan`,
    /// and `LifetimeEndBehavior`. Role, velocity, position, effects, and
    /// cleanup markers are untouched.
    pub fn become_normal(commands: &mut Commands, phantom: Entity) { ... }
}
```

The builder terminal `spawn_inner` calls `Bolt::become_phantom` internally when `optional.phantom.is_some()` (then separately inserts `PhantomFlicker` for rendered bolts). The lifespan-tick `RevertToNormalBolt` branch calls `PhantomBolt::become_normal`. No ad-hoc `insert((PhantomBolt, PhantomDedupKey, ...))` anywhere else in the codebase.

### Terminal behavior

When `optional.phantom.is_some()`, `spawn_inner` (on top of the standard rendered/headless bolt):

1. Calls `Bolt::become_phantom(commands, entity, params.dedup_key.clone())` — inserts `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`.
2. When `V == Rendered`, inserts `PhantomFlicker::default()`.
3. When `V == Headless`, skips the flicker insert.

When `optional.lifetime_end_behavior.is_some()`, inserts `LifetimeEndBehavior`.

Default collision mask (`CELL_LAYER | WALL_LAYER | BREAKER_LAYER`) is correct for phantoms unchanged. No mask modification.

### Collision-response branching

**Walls (`bolt/systems/bolt_wall_collision/system.rs`):** NO `With<PhantomBolt>` branch. If one exists today, **delete it.** Wall rebound applies to phantom and non-phantom bolts identically.

**Real breakers (`bolt/systems/bolt_breaker_collision/system.rs`):** NO `With<PhantomBolt>` branch in the real-breaker-hit path. If one exists today, **delete it.** Phantoms bounce off real breakers identically to non-phantom bolts; `BoltImpactBreaker` still emits; bump grading runs; `LastImpact` and `PiercingRemaining` update.

**Phantom breakers (same file):** The phantom-breaker-specific skip-tilt/skip-spread/skip-last-impact/skip-piercing branches from TODO #4 are per-phantom-*breaker* (gated `With<PhantomBreaker>` on the breaker side), NOT per-phantom-*bolt*. This TODO does not touch those branches.

**Cells (`bolt/systems/bolt_cell_collision/system.rs`):** THE ONLY phantom-bolt-specific branch.

```rust
if let Ok(mut damaged_set) = phantom_q.get_mut(bolt) {
    if damaged_set.0.insert(cell) {
        damage_writer.write(DamageDealt { target: cell, amount, source: "phantom" });
    }
    continue;  // No velocity flip — phantoms pass through cells.
}
```

`PhantomDamagedCells(HashSet<Entity>)` is a per-phantom component ensuring each cell is damaged at most once per phantom lifetime. Cleared implicitly when the phantom reverts or despawns (component is removed in both cases).

### Lifespan-end dispatch

The existing bolt-lifespan tick system (reads `Lifespan`, currently unconditional `despawn()` on expiry) updates to read `LifetimeEndBehavior`:

```rust
fn tick_bolt_lifespan(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifespan, Option<&LifetimeEndBehavior>), With<Bolt>>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifespan, end_behavior) in &mut query {
        lifespan.remaining -= dt;
        if lifespan.remaining <= 0.0 {
            match end_behavior.copied().unwrap_or(LifetimeEndBehavior::Despawn) {
                LifetimeEndBehavior::Despawn => {
                    despawn_writer.write(DespawnEntity { entity });
                }
                LifetimeEndBehavior::RevertToNormalBolt => {
                    PhantomBolt::become_normal(&mut commands, entity);
                }
            }
        }
    }
}
```

Non-phantom bolts with a lifespan and no `LifetimeEndBehavior` default to `Despawn` (existing behavior preserved via `.unwrap_or(Despawn)`).

Runs in `FixedUpdate`, ordered before `DeathPipelineSystems::ProcessDespawn`. `DespawnEntity` is the unified death pipeline's single despawn primitive (TODO #0).

### Migration: afterimage `spawn_phantom_bolt`

Open `breaker-game/src/protocol/protocols/afterimage/system/spawn_phantom_bolt.rs` (under TODO #1, this path becomes `mutators/protocols/afterimage/system/spawn_phantom_bolt.rs`).

**Delete the extra-bolt spawn entirely.** The current `.extra().headless().spawn(...)` + post-spawn marker insertion block comes out. Replace with the mutate-in-place call on the real bolt:

```rust
fn afterimage_spawn_phantom_bolt(
    mut commands: Commands,
    mut perfect_reader: MessageReader<PerfectBumpOnPhantomBreaker>,
    real_bolts: Query<(Entity, Has<PhantomBolt>), With<Bolt>>,
    config: Res<AfterimageConfig>,
) {
    for msg in perfect_reader.read() {
        let Ok((real_bolt, already_phantom)) = real_bolts.get(msg.bolt) else { continue };
        if already_phantom {
            continue;  // duration does NOT reset
        }
        Bolt::become_phantom(
            &mut commands,
            real_bolt,
            PhantomDedupKey::Bolt(real_bolt),
        );
        commands.entity(real_bolt).insert((
            Lifespan { remaining: config.phantom_duration },
            LifetimeEndBehavior::RevertToNormalBolt,
            PhantomFlicker::default(),
        ));
    }
}
```

(Exact message trigger depends on how afterimage currently detects "Perfect bump on phantom breaker" — match the existing wiring. The important shift is: mutate the real bolt's entity, do NOT spawn a new one.)

Spawn system no longer needs `ResMut<Assets<Mesh>>` / `ResMut<Assets<ColorMaterial>>` (no spawning), but the chip-effect path DOES need them.

### Migration: chip-effect `SpawnPhantomConfig::fire`

Open `breaker-game/src/effect_v3/effects/phantom_bolt/config.rs:26-53`. Replace the raw `world.spawn(...)` with the builder. `Fireable::fire` gains the asset `ResMut`s in its signature.

The dedup filter updates to match the pair:

```rust
let existing_count = world
    .query::<(&PhantomBolt, &PhantomDedupKey)>()
    .iter(world)
    .filter(|(_, key)| matches!(
        key,
        PhantomDedupKey::Chip { chip, fired_from }
            if chip == source && *fired_from == entity
    ))
    .count();
if existing_count >= config.max_active {
    return;
}

// ... then the builder call from the Design > Path 2 block above.
```

### RON and design-doc updates

- `assets/protocols/afterimage.protocol.ron` — update the `description` from "split off an extra bolt" to "bolt becomes Phantom" (match the design doc).
- `docs/design/protocols/afterimage.md` — if any language lingers from the split-bolt interpretation, align to mutate-real-bolt.

### Code deletions

- `with_extra_mask_bits` method on the bolt builder — zero callers after afterimage migrates (afterimage was the only one). Delete the method from `transitions.rs` and the `extra_mask_bits` field from `OptionalBoltData`.
- Raw `world.spawn(...)` path in `effect_v3/effects/phantom_bolt/config.rs` — replaced by builder.
- Post-build marker insertion at afterimage site — replaced by `Bolt::become_phantom`.
- `PhantomLifetime` component — replaced by shared `Lifespan` + `LifetimeEndBehavior`.
- `PhantomOwner(Entity)` component — replaced by `PhantomDedupKey`.
- `afterimage/tests/spawn_phantom_bolt.rs:124-125` regression-trap test that pins the wrong behavior — rewritten to assert the real bolt DOES gain `PhantomBolt` on Perfect bump.
- Any phantom-specific branches discovered in `bolt_wall_collision` or `bolt_breaker_collision` (outside the phantom-breaker side).

## Tests to author

### Builder tests

1. `.phantom(...)` on a `Rendered` terminal inserts: `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, `PhantomFlicker`, `ExtraBolt`, `CleanupOnExit<NodeState>`, `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Bolt`.
2. `.phantom(...)` on a `Headless` terminal inserts: `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, `ExtraBolt`, `CleanupOnExit<NodeState>`. No `PhantomFlicker`, `Mesh2d`, or `MeshMaterial2d`.
3. Spawned phantom collision mask equals the default (`CELL_LAYER | WALL_LAYER | BREAKER_LAYER`).
4. `.with_lifetime_end_behavior(RevertToNormalBolt)` inserts `LifetimeEndBehavior::RevertToNormalBolt` component.
5. `.with_lifetime_end_behavior(Despawn)` inserts `LifetimeEndBehavior::Despawn` component.

### `become_phantom` / `become_normal` tests

6. `Bolt::become_phantom(cmds, e, PhantomDedupKey::Bolt(e))` inserts `PhantomBolt`, `PhantomDedupKey::Bolt(e)`, `PhantomDamagedCells` on entity `e`.
7. `PhantomBolt::become_normal(cmds, e)` strips `PhantomBolt`, `PhantomDedupKey`, `PhantomDamagedCells`, `PhantomFlicker`, `Lifespan`, `LifetimeEndBehavior` from `e`. Role, velocity, position, cleanup markers unchanged.

### Collision tests (generic phantom bolt suite, lives at `bolt/systems/phantom_bolt_tests/`)

8. Phantom bounces off left/right/top wall identically to non-phantom. `BoltImpactWall` emitted.
9. Phantom bounces off real breaker identically to non-phantom. `BoltImpactBreaker` emitted. Bump grading runs. `LastImpact` and `PiercingRemaining` update.
10. Phantom bounces off phantom breaker (via shared `bolt_breaker_collision`). Phantom-breaker branches apply (skip tilt/spread/last-impact/piercing) — verify these are gated on the breaker side (`With<PhantomBreaker>`), not the bolt side (`With<PhantomBolt>`).
11. Phantom impacts a cell: `DamageDealt<Cell>` emitted once; velocity UNCHANGED; position advances through the cell.
12. Phantom impacts the SAME cell on consecutive frames: `DamageDealt<Cell>` emitted only on first contact (`PhantomDamagedCells` dedup).
13. Phantom that reverts via `RevertToNormalBolt` resumes normal cell rebound on its NEXT cell contact.

### Lifespan tests

14. Phantom with `Lifespan(t)` + `LifetimeEndBehavior::Despawn` emits `DespawnEntity` when lifespan reaches 0.
15. Phantom with `Lifespan(t)` + `LifetimeEndBehavior::RevertToNormalBolt` loses phantom component set at expiry; bolt continues with velocity/position/role/effects intact.
16. Non-phantom bolt with `Lifespan(t)` and NO `LifetimeEndBehavior` component defaults to despawn on expiry (back-compat).
17. `CleanupOnExit<NodeState>` despawns an `.extra()` phantom on `OnExit(NodeState::Playing)` regardless of remaining lifespan.

### Afterimage integration tests

18. Perfect bump on phantom breaker → real bolt gains `PhantomBolt`, `Lifespan`, `LifetimeEndBehavior::RevertToNormalBolt`, `PhantomFlicker`. **This is the rewrite of the regression-trap test at `tests/spawn_phantom_bolt.rs:124-125` — old test asserted the inverse.**
19. Non-Perfect bump on phantom breaker → real bolt does NOT gain `PhantomBolt`.
20. Perfect bump on phantom breaker when real bolt ALREADY has `PhantomBolt` → no-op (duration does NOT reset).
21. Phantom-duration expires → real bolt loses `PhantomBolt` via `RevertToNormalBolt` branch; continues as normal.

### Chip-effect integration tests

22. `SpawnPhantomConfig::fire` with `source = "phantom_bolt"`, `entity = real_bolt` produces a phantom with `PhantomDedupKey::Chip { chip: "phantom_bolt", fired_from: real_bolt }`.
23. `SpawnPhantomConfig::fire` skips spawn when `existing_count >= max_active` for the matching `(chip, fired_from)` pair.
24. `SpawnPhantomConfig::fire` phantoms despawn on expiry (`LifetimeEndBehavior::Despawn`).

### Visual

25. `tick_phantom_flicker` modulates `ColorMaterial.color.alpha` over time for entities with `PhantomFlicker` (rendered phantoms only).

## Code change summary

| File | Change |
|------|--------|
| `bolt/builder/core/types.rs` | Add `phantom: Option<PhantomParams>` + `lifetime_end_behavior: Option<LifetimeEndBehavior>` to `OptionalBoltData`. Add `PhantomParams`, `PhantomDedupKey`, `LifetimeEndBehavior`, `PhantomDamagedCells` types. |
| `bolt/builder/core/transitions.rs` | Add `.phantom(...)` + `.with_lifetime_end_behavior(...)` optional-chainable methods. Delete `.with_extra_mask_bits(...)`. Delete `extra_mask_bits` field from `OptionalBoltData`. |
| `bolt/builder/core/terminal.rs` | `spawn_inner` calls `Bolt::become_phantom` when `optional.phantom.is_some()`; inserts `PhantomFlicker` when Rendered; inserts `LifetimeEndBehavior` when set. |
| `bolt/components/phantom.rs` (new or existing — match project convention) | `PhantomBolt` marker, `PhantomDedupKey` enum component, `PhantomDamagedCells` component, `LifetimeEndBehavior` component. |
| `bolt/components/phantom.rs` (same file) | `Bolt::become_phantom(commands, entity, dedup_key)` and `PhantomBolt::become_normal(commands, entity)` associated functions. |
| `bolt/systems/tick_bolt_lifespan.rs` (or wherever the current tick lives) | Read `LifetimeEndBehavior`, branch `Despawn` → `DespawnEntity` message, `RevertToNormalBolt` → `PhantomBolt::become_normal`. Default to `Despawn` when component absent. |
| `bolt/systems/bolt_wall_collision/system.rs` | AUDIT: delete any `With<PhantomBolt>` branch. Phantom bolts rebound off walls identically to normal. |
| `bolt/systems/bolt_breaker_collision/system.rs` | AUDIT: delete any `With<PhantomBolt>` branch in the real-breaker path. Phantom-breaker-side branches (TODO #4) unchanged. |
| `bolt/systems/bolt_cell_collision/system.rs` | Keep phantom branch at `:41,89,101`: dedup via `PhantomDamagedCells`, emit `DamageDealt<Cell>` once, skip velocity flip. |
| `bolt/systems/phantom_bolt_tests/` (new) | Generic phantom-bolt collision suite. |
| `mutators/protocols/afterimage/system/spawn_phantom_bolt.rs` | Rewrite: mutate real bolt via `Bolt::become_phantom` instead of spawning new. Delete all `commands.spawn(...)` + `.extra().headless()` + post-spawn-marker-insertion code. |
| `mutators/protocols/afterimage/tests/spawn_phantom_bolt.rs` | Rewrite regression-trap test at `:124-125` to assert the inverse (real bolt DOES gain `PhantomBolt`). Delete or rewrite tests asserting the separate-bolt shape. |
| `effect_v3/effects/phantom_bolt/config.rs` | Migrate `SpawnPhantomConfig::fire` to builder. Promote `source` to load-bearing dedup input. Update `max_active` filter to match the `(chip, fired_from)` pair. |
| `effect_v3/effects/phantom_bolt/config.rs` (signature) | `Fireable::fire` gains `Commands` + `ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>` params. |
| `assets/protocols/afterimage.protocol.ron` | Update description: "bolt becomes Phantom". |
| `docs/design/protocols/afterimage.md` | Align language if any split-bolt interpretation lingers. |
| `bolt/components/phantom.rs` | Delete `PhantomLifetime`, `PhantomOwner`. |
| `docs/architecture/plugins.md` | Narrow/remove afterimage entry from Velocity2D exception registry (direct `Velocity2D`/`Position2D` writes vanish alongside the deleted split-bolt path). |

## Dependencies

- **TODO #0 (unified death pipeline crate)** — `DespawnEntity` message + `DeathPipelineSystems::ProcessDespawn` set + `process_despawn_requests`. The `LifetimeEndBehavior::Despawn` branch writes the crate's message. Must land after #1.
- **TODO #1 (mutators domain refactor)** — afterimage moves to `mutators/protocols/afterimage/`. The afterimage migration touches files under the new path. Must land after #2.
- **TODO #4 (phantom breaker)** — introduces shared `Lifespan`, `PhantomFlicker`, `tick_phantom_flicker` components/systems. Phantom bolts reuse them without re-introducing. `LifetimeEndBehavior` is NEW in this TODO (bolt-only enum; breakers unconditionally despawn).

## Ordering

Lands after TODO #0, #2, #5. Independent of #3 (greed skip) and #4 (bolt-loss behavior) — can interleave.

## Subsumes

- `audit/remediations/bolt-builder-phantom-transition.md` — this file IS that spec, expanded and aligned to mutate-real-bolt for afterimage.
- `audit/remediations/phantom-bolt-mutate-real-bolt.md` — the mutate-real-bolt design is baked into afterimage's path here.
- `audit/remediations/phantom-bolt-collision-semantics-tests.md` — the "phantom is a normal bolt for every surface except cells" contract + generic test suite is specified here.

## Not subsumed

- `audit/remediations/placeholder-vfx-for-deferred-items.md` — cross-cutting file covering drift arrow, echo ghost tint, resonance/overcharge/renewal tints, etc. Phantom-bolt flicker is ONE item on that list but removing the whole file would drop the others. That file stays; phantom-bolt flicker from this TODO satisfies its phantom-bolt entry.

## Scope boundary

In scope:
- Bolt builder `.phantom()` + `.with_lifetime_end_behavior()` methods
- `Bolt::become_phantom` / `PhantomBolt::become_normal` canonical switch points
- `LifetimeEndBehavior` enum (bolt-only) with `Despawn` and `RevertToNormalBolt`
- `PhantomDedupKey` enum (`Bolt(Entity)` + `Chip { chip, fired_from }`)
- `PhantomDamagedCells` per-phantom cell dedup
- Afterimage rewrite: mutate real bolt, not spawn new
- Chip-effect migration: raw `world.spawn` → builder
- Collision-system audit: delete phantom-bolt branches from wall/real-breaker paths
- Rewrite of the regression-trap test at `tests/spawn_phantom_bolt.rs:124-125`
- Deletions: `with_extra_mask_bits`, `PhantomLifetime`, `PhantomOwner`, split-bolt spawn code

Out of scope:
- Phase 5 polished phantom visual (placeholder flicker only)
- Changes to WHEN afterimage triggers phantom mode (Perfect bump on phantom breaker stays the trigger)
- Changes to `SpawnPhantomConfig` chip behavior beyond spawn-path migration
- Phantom breaker (TODO #4)

## TODO entry

> **[BLOCKED by #1, #2, #5]** Phantom bolt — builder `.phantom(PhantomParams)` + `.with_lifetime_end_behavior(LifetimeEndBehavior)` methods, `Bolt::become_phantom` / `PhantomBolt::become_normal` canonical switch points, `LifetimeEndBehavior { Despawn, RevertToNormalBolt }` enum, `PhantomDedupKey { Bolt(Entity), Chip { chip, fired_from } }` tagged dedup, afterimage rewrite to MUTATE the real bolt (not spawn a new one) with `RevertToNormalBolt` on expiry, chip-effect `SpawnPhantomConfig::fire` migrated to builder with `Despawn` on expiry, generic phantom-bolt collision suite proving "normal bolt for every surface except cells", deletion of phantom-bolt branches in wall/real-breaker collision, regression-trap test at `spawn_phantom_bolt.rs:124-125` rewritten to assert the correct behavior. Subsumes `bolt-builder-phantom-transition.md`, `phantom-bolt-mutate-real-bolt.md`, `phantom-bolt-collision-semantics-tests.md`. — [detail](detail/phantom-bolt.md)
