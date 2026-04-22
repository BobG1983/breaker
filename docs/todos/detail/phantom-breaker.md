# Phantom breaker — builder transition, lifespan dispatch, shared infra

## Problem

Phantom breakers today are broken: invisible (no mesh/material/draw layer), lack the `Breaker` marker (so `With<Breaker>` queries miss them), use raw `Width`/`Height` (bypass SizeBoost), fall back to hardcoded `DEFAULT_PHANTOM_BASE_*` constants when components are missing, use a parallel `PhantomBreakerLifetime` type instead of a shared lifespan component, duplicate bolt-breaker bounce logic in `afterimage_check_phantom_bounce`, and get spawned via raw `commands.spawn((...))` that bypasses the breaker builder entirely.

Nine failure modes documented in `audit/builders/breaker_phantom.md` and `audit/protocols/afterimage.md` all resolve via a single transition: make `Breaker::builder().phantom(...)` the one spawn path.

## Scope

**In scope: phantom breaker alone.** Phantom bolts (`bolt-builder-phantom-transition.md`, `phantom-bolt-mutate-real-bolt.md`, `phantom-bolt-collision-semantics-tests.md`, `placeholder-vfx-for-deferred-items.md`) stay as their own remediations. This TODO introduces the *shared* phantom infra (`Lifespan`, `PhantomFlicker`, `tick_phantom_flicker`) because phantom breakers need it; phantom bolts will adopt the same infra later without re-introducing it.

## Design

### Phantom-breaker contract

A phantom breaker:

- **IS a `Breaker`.** Carries the `Breaker` marker. Existing `With<Breaker>` queries MUST see it. No parallel `Or<(With<Breaker>, With<PhantomBreaker>)>` queries.
- **Has fixed position.** Spawned at the real breaker's location (typically at a Perfect Bump moment). Horizontal movement input is gated out; does not slide, does not dash.
- **Receives bump input.** Participates in `update_bump` → `grade_bump` → `BumpPerformed`. A single `InputActions::Bump` press opens windows on ALL breakers (real + phantoms), each grading independently against its own `BumpState`.
- **Bounces the bolt.** `bolt_breaker_collision` reflects bolts off phantoms via `With<Breaker>`. Real-breaker-only side effects (tilt, spread override, last-impact offset, piercing-bolt handling) gate via `Without<PhantomBreaker>`.
- **Always has a lifespan.** `.phantom(...)` takes a required `lifespan: f32` field (unlike phantom bolts where lifespan is optional).
- **Always despawns on expiry.** No `LifetimeEndBehavior` enum for breakers — unconditional despawn at lifespan end. `LifetimeEndBehavior` stays bolt-only.
- **Is visually distinct when rendered.** Tinted material + `PhantomFlicker` placeholder. Headless phantoms skip visual components.
- **Cleanup orthogonal to phantom-ness.** `.phantom()` does NOT force `.primary()` or `.extra()`. Afterimage spawns phantoms as `.extra()` so they clean up at node exit; that's the caller's choice, not the method's.

### Builder API

Add `.phantom(BreakerPhantomParams)` as an **optional chainable method** on the breaker builder (alongside `with_lives`, `with_effects`, `with_color`). Works on any typestate — stashes params in `OptionalBreakerData` for the terminal to consume.

```rust
// breaker-game/src/breaker/builder/core/types.rs
pub(crate) struct OptionalBreakerData {
    // ... existing fields ...
    pub(crate) phantom: Option<BreakerPhantomParams>,
}

#[derive(Debug, Clone, Copy)]
pub struct BreakerPhantomParams {
    /// Lifespan in seconds. REQUIRED — every phantom breaker has a lifespan.
    pub lifespan: f32,
    /// Color tint mixed with the base breaker color (e.g., pale cyan).
    pub phantom_color_rgb: [f32; 3],
    /// Flicker frequency in Hz (placeholder for Phase 5 VFX).
    pub flicker_frequency: f32,
    /// Minimum alpha during flicker.
    pub flicker_min_alpha: f32,
}
```

```rust
// breaker-game/src/breaker/builder/core/transitions.rs
impl<D, Mv, Da, Sp, Bm, V, R> BreakerBuilder<D, Mv, Da, Sp, Bm, V, R> {
    #[must_use]
    pub const fn phantom(mut self, params: BreakerPhantomParams) -> Self {
        self.optional.phantom = Some(params);
        self
    }
}
```

### Terminal behavior

When `optional.phantom.is_some()`, the terminal (on top of the standard rendered/headless breaker it already produces) additionally:

1. Inserts `PhantomBreaker` marker.
2. Inserts `Lifespan { remaining: params.lifespan }`.
3. When visual typestate is `Rendered`: mixes `phantom_color_rgb` into the spawned material and inserts `PhantomFlicker { frequency, min_alpha }`.
4. When visual typestate is `Headless`: skips material mix and `PhantomFlicker` insert.

The default `CollisionLayers { layer: BREAKER_LAYER, mask: BOLT_LAYER }` applies unchanged — phantoms participate in the quadtree identically to real breakers.

### Gating in existing systems

Mechanical rule: systems whose effects are real-breaker-only add `Without<PhantomBreaker>` to their `With<Breaker>` queries. Systems where phantom participation is intentional stay untouched.

| System | File | Gate |
|--------|------|------|
| `move_breaker` horizontal velocity | `breaker/systems/move_breaker/` | `Without<PhantomBreaker>` |
| Dash-input transition | dash input system | `Without<PhantomBreaker>` |
| `perfect_bump_dash_cancel` | `breaker/systems/bump/` | `Without<PhantomBreaker>` |
| `update_breaker_state`, `trigger_bump_visual`, `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale`, `breaker_cell_collision`, `breaker_wall_collision`, node-reset systems | various | `Without<PhantomBreaker>` |
| `bolt_breaker_collision` real-breaker-only branches (tilt, spread override, last-impact offset, piercing-bolt) | `bolt/systems/bolt_breaker_collision/system.rs` | inline `Without<PhantomBreaker>` branches within the collision response |
| `bolt_breaker_collision` core reflection | same | **no gate** — phantoms bounce bolts |
| `update_bump` | `breaker/systems/bump/system.rs` | **no gate** — phantoms receive bump input |
| `grade_bump` | `breaker/systems/bump/system.rs` | **no gate** — but see migration below |

### CRITICAL: `grade_bump` `.single_mut()` → `.iter_mut()` migration

`grade_bump` currently calls `bump_query.single_mut()` — asserts exactly one breaker. As soon as a phantom spawns, this panics. Migration is mandatory:

```rust
// Before
let (entity, mut bump_state) = bump_query.single_mut();
// process messages targeting this one breaker

// After
for (entity, mut bump_state) in &mut bump_query {
    // match incoming BoltImpactBreaker messages by breaker entity:
    //   if msg.breaker == entity { grade using bump_state }
}
```

Each breaker (real or phantom) grades against its own `BoltImpactBreaker` messages using its own `BumpState`. No cross-referencing between breakers. Write `BumpPerformed { grade, bolt, breaker: entity }` with the per-breaker entity so downstream consumers can distinguish hits on phantoms vs real.

This migration has a regression-test requirement: spawn a phantom, fire a bolt, assert no panic. See tests #8 below.

### Lifespan-end dispatch via unified death pipeline

Phantom breakers integrate with the death pipeline introduced in TODO #1 (`rantzsoft_dmg`). The pipeline defines `DespawnEntity { entity: Entity }` as the single despawn primitive consumed by `process_despawn_requests` in `FixedPostUpdate`. This is the one authorized despawn path.

**`KillYourself<Breaker>` cannot be used.** `handle_breaker_death` (the specialized Breaker kill handler) emits `RunLost` on every `KillYourself<Breaker>` — a phantom expiring would end the player's run. Phantom death is not a run-ending event.

The lifespan-tick system emits `DespawnEntity` directly:

```rust
fn tick_phantom_breaker_lifespan(
    time: Res<Time<Fixed>>,
    mut query: Query<(Entity, &mut Lifespan), (With<Breaker>, With<PhantomBreaker>)>,
    mut despawn_writer: MessageWriter<DespawnEntity>,
) {
    let dt = time.delta_secs();
    for (entity, mut lifespan) in &mut query {
        lifespan.remaining -= dt;
        if lifespan.remaining <= 0.0 {
            despawn_writer.write(DespawnEntity { entity });
        }
    }
}
```

Runs in `FixedUpdate`, ordered before `DeathPipelineSystems::ProcessDespawn`. No `KillYourself<Breaker>`, no `Destroyed<Breaker>` (phantom death does not fire the effect bridges wired for real-breaker death), no `RunLost`. The pipeline's single despawn system removes the entity.

The `With<PhantomBreaker>` filter is mandatory — without it, a real breaker carrying a `Lifespan` (if one ever exists) would silently despawn mid-run. Real breakers don't carry `Lifespan` today and the design does not add one, but the filter makes the contract explicit.

### Shared phantom infrastructure

Introduced by this TODO, reused by phantom bolts later:

- **`Lifespan { remaining: f32 }`** component. Shared type, canonical location under `shared::lifespan::` (or equivalent).
- **`PhantomFlicker { frequency, min_alpha }`** component. Shared type, canonical location under `shared::phantom::` (or equivalent).
- **`tick_phantom_flicker`** FX system. Iterates entities with `PhantomFlicker`, modulates material alpha. Runs in `Update`. Registered once by `fx::plugin`.

**Not shared**:

- `LifetimeEndBehavior` — bolt-only (phantoms don't need the enum; they always despawn).
- Lifespan-tick dispatchers — breakers get `tick_phantom_breaker_lifespan`; bolts will get their own separately.

### Afterimage spawn path migration

Open `breaker-game/src/protocol/protocols/afterimage/system/spawn_phantom_breaker.rs`. Replace the raw `commands.spawn((...))` with the builder, reading the real breaker's runtime components (width, height, movement, dash, spread, bump, color) so the phantom matches current state (post-SizeBoost, post-any-other-modifier):

```rust
Breaker::builder()
    .dimensions(real.width, real.height, real.y_position)
    .movement(real.movement_settings)
    .dashing(real.dash_settings)
    .spread(real.spread_degrees)
    .bump(real.bump_settings)
    .with_color(real.color_rgb)
    .phantom(BreakerPhantomParams {
        lifespan:          config.phantom_duration,
        phantom_color_rgb: [0.4, 0.8, 1.0],
        flicker_frequency: 4.0,
        flicker_min_alpha: 0.3,
    })
    .extra()
    .rendered(&mut meshes, &mut materials)
    .spawn(&mut commands);
```

Spawn system gains `ResMut<Assets<Mesh>>` + `ResMut<Assets<ColorMaterial>>` params (currently missing).

Global-singleton enforcement (despawn existing phantoms before spawning a new one) stays as-is — the despawn loop runs before the builder call.

### Code deletions

- `DEFAULT_PHANTOM_BASE_WIDTH` / `DEFAULT_PHANTOM_BASE_HEIGHT` constants — builder's dimension path covers this.
- `afterimage_check_phantom_bounce` system + file + 880-line `tests/check_phantom_bounce.rs` — the phantom is a `Breaker` now; `bolt_breaker_collision` handles the bounce.
- `PhantomBreakerLifetime` type — replaced by shared `Lifespan`.
- Afterimage entry in `docs/architecture/plugins.md` Velocity2D exception registry — the raw `Velocity2D` writes vanish with `check_phantom_bounce`; the exception can be narrowed or removed in lockstep.

## Tests to author

1. `.phantom(...)` on a `Rendered` terminal inserts: `Breaker`, `PhantomBreaker`, `PhantomFlicker`, `Lifespan`, `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Breaker`, `Width`, `Height`, `BumpState`, `HasBump`, `CollisionLayers`.
2. `.phantom(...)` on a `Headless` terminal inserts: `Breaker`, `PhantomBreaker`, `Lifespan`, `Width`, `Height`, `BumpState`, `HasBump`, `CollisionLayers`. No `PhantomFlicker`, `Mesh2d`, or `MeshMaterial2d`.
3. `With<Breaker>` queries include the phantom.
4. Movement input drives the real breaker but NOT the phantom.
5. Dash input triggers dash on the real breaker but NOT the phantom.
6. Bump input triggers bump on BOTH the real breaker and the phantom (no gating on bump).
7. Bolt→phantom-breaker collision produces a standard velocity reflection (no tilt, no spread override, no last-impact offset, no piercing-bolt handling).
8. **Regression:** `grade_bump` runs without panicking when a phantom is present (`.single_mut()` → `.iter_mut()` migration).
9. `grade_bump` grades a bump against the phantom's own `BumpState`; `BumpPerformed { breaker: <phantom entity> }` is emitted when a bolt hits the phantom within the window.
10. Phantom with `BreakerPhantomParams { lifespan: t, ... }` emits `DespawnEntity { entity }` and is removed by the death pipeline when `Lifespan` reaches 0.
11. `CleanupOnExit<NodeState>` despawns an `.extra()` phantom on `OnExit(NodeState::Playing)` regardless of remaining lifespan.
12. Phantom expiry does NOT emit `KillYourself<Breaker>`, `Destroyed<Breaker>`, or `RunLost`.
13. Rendered phantom's material color differs from real breaker's base color (phantom tint mixed in).
14. `perfect_bump_dash_cancel` does NOT cancel the real breaker's dash when a Perfect is registered on a phantom-only hit. (Design call per `delete-check-phantom-bounce.md` — default is "phantom perfect does not cancel real dash.")
15. Spawning a phantom when the real breaker has `Width(150.0)` + `Height(30.0)` produces a phantom with the same dimensions (reads live components, not definitions).

## Code change summary

| File | Change |
|------|--------|
| `breaker/builder/core/types.rs` | Add `phantom: Option<BreakerPhantomParams>` to `OptionalBreakerData`; add `BreakerPhantomParams` struct. |
| `breaker/builder/core/transitions.rs` | Add `.phantom(...)` optional-chainable method. |
| `breaker/builder/core/terminal.rs` | Phantom-params integration: color mix when Rendered, marker + `Lifespan` + `PhantomFlicker` insert. |
| `breaker/systems/bump/system.rs` | `grade_bump` migration `.single_mut()` → `.iter_mut()`; match `BoltImpactBreaker` by breaker entity. |
| `breaker/systems/move_breaker/system.rs` | Add `Without<PhantomBreaker>` to horizontal-velocity query. |
| Dash input system | Add `Without<PhantomBreaker>` to dash-transition query. |
| `breaker/systems/bump/perfect_bump_dash_cancel.rs` (or wherever) | Add `Without<PhantomBreaker>`. |
| `bolt/systems/bolt_breaker_collision/system.rs` | Narrow `Without<PhantomBreaker>` branches for tilt, spread override, last-impact offset, piercing bolt; core reflection stays on `With<Breaker>`. |
| `breaker/systems/update_breaker_state`, `trigger_bump_visual`, `animate_bump_visual`, `animate_tilt_visual`, `sync_breaker_scale`, `breaker_cell_collision`, `breaker_wall_collision`, node-reset systems | Add `Without<PhantomBreaker>` where they operate on the real breaker only. |
| `breaker/systems/tick_phantom_breaker_lifespan.rs` | New system. `FixedUpdate`, before `DeathPipelineSystems::ProcessDespawn`. Emits `DespawnEntity`. |
| `shared::phantom` (or `shared::lifespan`) | New shared module: `Lifespan` + `PhantomFlicker` components. |
| `fx::plugin` | Register `tick_phantom_flicker`. |
| `protocol/protocols/afterimage/system/spawn_phantom_breaker.rs` | Migrate to builder; add mesh/material asset params; read real breaker's runtime components. |
| `protocol/protocols/afterimage/system/check_phantom_bounce.rs` | **DELETED.** |
| `protocol/protocols/afterimage/system/register.rs` | Remove `check_phantom_bounce` registration. |
| `protocol/protocols/afterimage/system/components.rs` | Remove `PhantomBreakerLifetime`. |
| `breaker-game/src/protocol/protocols/afterimage/tests/check_phantom_bounce.rs` | **DELETED** (880 lines). |
| `docs/architecture/plugins.md` | Narrow/remove afterimage entry from Velocity2D exception registry. |

## Dependencies

- **TODO #1 (unified death pipeline crate)** — `DespawnEntity` message, `DeathPipelineSystems::ProcessDespawn` set, and `process_despawn_requests` system all move to the crate. `tick_phantom_breaker_lifespan` writes the crate's message. Must land after #1.
- **TODO #2 (mutators domain refactor)** — afterimage moves to `mutators/protocols/afterimage/`. The phantom-builder migration touches files under the new path. Must land after #2.

## Ordering

Lands after TODO #1 and TODO #2. Independent of TODO #3 (greed skip) and TODO #4 (bolt-loss behavior) — can interleave with them.

Companion: `bolt-builder-phantom-transition.md` shares infra (`Lifespan`, `PhantomFlicker`, `tick_phantom_flicker`). Not bundled into this TODO per scope decision, but once this lands the bolt side is a smaller follow-up (builder method + bolt-specific lifespan tick; shared infra already in place).

## Subsumes

- `audit/remediations/phantom-breaker-dimension-fallback.md` — builder reads live `Width`/`Height` components; `DEFAULT_PHANTOM_BASE_*` constants deleted.
- `audit/remediations/delete-check-phantom-bounce.md` — `afterimage_check_phantom_bounce` deleted; phantoms flow through `bolt_breaker_collision` with `Without<PhantomBreaker>` branches.
- `audit/remediations/breaker-builder-phantom-transition.md` — this file IS that spec, expanded and lifted to a TODO.

## Scope boundary

In scope:
- Breaker builder `.phantom()` method + terminal integration
- Gating `Without<PhantomBreaker>` across real-breaker-only systems
- `grade_bump` `.single_mut()` → `.iter_mut()` migration
- Shared `Lifespan` + `PhantomFlicker` components
- `tick_phantom_breaker_lifespan` dispatcher
- Afterimage spawn-path migration to builder
- Deletions: `check_phantom_bounce`, `DEFAULT_PHANTOM_BASE_*`, `PhantomBreakerLifetime`

Out of scope:
- Phantom bolts (separate remediations)
- Phantom bolt mutate-real-bolt logic
- Phase 5 VFX polish for phantom visuals (placeholder flicker only)
- Changes to when/why afterimage spawns phantoms (the mechanic stays unchanged; only the spawn *mechanism* changes)

## TODO entry

> **[ready / BLOCKED by #1, #2]** Phantom breaker — builder transition `Breaker::builder().phantom(BreakerPhantomParams { lifespan, ... })`, shared `Lifespan` + `PhantomFlicker` infra, `grade_bump` `.single_mut()` → `.iter_mut()` migration (panics on phantom spawn otherwise), lifespan dispatch via `DespawnEntity` (not `KillYourself<Breaker>` which would trigger `RunLost`), `Without<PhantomBreaker>` gating across movement/dash/tilt systems, afterimage spawn path migrated to builder, `afterimage_check_phantom_bounce` deleted (880-line test file too). Subsumes `breaker-builder-phantom-transition.md`, `phantom-breaker-dimension-fallback.md`, `delete-check-phantom-bounce.md`. Depends on #1 (`DespawnEntity`) and #2 (afterimage relocation). — [detail](detail/phantom-breaker.md)
