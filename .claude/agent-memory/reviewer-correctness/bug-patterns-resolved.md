---
name: Resolved bug patterns — do not re-flag
description: Bugs and patterns confirmed FIXED, RESOLVED, or CONFIRMED CORRECT in this codebase — check this before flagging anything here as a new issue
type: project
---

## rantzsoft_stateflow transition effects: elapsed never incremented — FIXED (2026-04-06)

All 12 built-in effects (fade, dissolve, pixelate, wipe, iris, slide) now call
`progress.elapsed += time.delta_secs()` in their run systems.
`TransitionRunComplete` is sent when `elapsed >= duration`.

## Insert-without-increment on first fire() — FIXED in Phase 1 cleanup

`ramping_damage::fire()` now inserts with `accumulated: damage_per_trigger` on first call.
`multi_call_accumulation_is_linear` confirms: 4 calls at 0.5 yield 2.0 (not 1.5).

Watch for this pattern in NEW effect `fire()` functions that accumulate state.

## ccd_sweep_breaker fallback Aabb2D center semantics — CONFIRMED CORRECT

`Aabb2D::new(breaker_pos, expanded_half)` — `Aabb2D::new` stores its first arg
directly as `center`, and `ray_intersect` treats `self.center` as world-space.
Since `breaker_pos` is already world-space, this is correct.

## bolt_wall_collision: query uses query_aabb_filtered — CONFIRMED CORRECT

Current code uses `query_aabb_filtered` (not `query_circle_filtered`).
The AABB query is correct for finding wall candidates within bolt radius.

## gravity_well::fire() infinite loop when max == 0 — FIXED (feature/chip-evolution-ecosystem 2026-03-31)

`gravity_well::fire()` now has `if world.get_entity(entity).is_err() { return; }`
AND `if max == 0 { return; }` guards at lines 47-53.
`spawn_phantom::fire()` also has the despawned-entity guard at line 46.

## Phase 4 effect systems: shockwave/pulse migrated to Position2D — FIXED (full-verification-fixes)

`apply_shockwave_damage`, `apply_pulse_damage`, `process_explode_requests` now read `Position2D` directly.
Spawned shockwave/explode/ring entities now have `Position2D(position)` set at fire-time.
They still do NOT have `GlobalPosition2D` or `Spatial2D` — they are ephemeral effect entities.

## Phase 5 chain_lightning rework: arcs==0 / range<=0 early returns — FIXED

`arc_speed <= 0.0` now triggers an early return in `fire()` (effect.rs line 82-84).
No chain entity is spawned when arc_speed is zero or negative.
`ChainLightningChain`/`tick_chain_lightning` sequential arc design replaced old `ChainLightningRequest`.

## Phase 5 piercing_beam: Position2D fix — FIXED (full-verification-fixes)

`piercing_beam::fire()` now uses `entity_position()` helper which returns `Position2D -> Vec2::ZERO`.
No `Transform` fallback.

## dispatch_chip_effects: effects dispatched even on max-stack add_chip failure — FIXED

`dispatch_chip_effects` now has `continue;` after the `add_chip` max-stacks warning.
Test: `chip_at_max_stacks_does_not_dispatch_effects`.

## apply_pending_bolt_effects: silently drops effects if bolt lacks BoundEffects — FIXED

Now uses `commands.entity(entity).insert_if_new((BoundEffects::default(), StagedEffects::default()))`
before extending — matching the cell/wall variants.

## bypass_menu_to_playing: Target::Breaker initial_effects always dropped — FIXED

`PendingBreakerEffects` resource introduced in scenario-runner `lifecycle/systems/types.rs`.
`apply_pending_breaker_effects` registered in FixedUpdate after `tag_game_entities`.
`bypass_menu_to_playing` uses `commands.insert_resource(PendingBreakerEffects(...))`.
`menu_bypass.rs` handles all 4 target types via `Pending*Effects`.

## TransferCommand silently drops non-Do children when BoundEffects/StagedEffects absent — FIXED (full-verification-fixes)

`ensure_effect_components()` helper now inserts both `BoundEffects` and `StagedEffects`
as defaults before the `get_mut` calls.

## TetherChainActive resource lifecycle — RESOLVED (2026-03-31)

`reverse(chain=true)` correctly calls `world.remove_resource::<TetherChainActive>()` when
the chip effect expires. `cleanup_tether_chain_resource` runs `OnExit(GameState::Playing)`
as a safety net. `TetherChainBeam` entities have `CleanupOnNodeExit`.

Within a run, the resource intentionally persists across nodes. No leak exists.

## Missing cross-domain ordering: EffectSystems::Recalculate — RESOLVED by cache removal

The Phase 3 issue about `Effective*` components needing `.after(EffectSystems::Recalculate)` is
no longer applicable. The cache-removal refactor eliminated all `Effective*` components and
`recalculate_*` systems. Consumer systems now read `Active*` directly each frame.
**Do NOT re-flag this ordering issue.**

## Scale2D stores absolute pixel dimensions, not scale ratios — CONFIRMED CORRECT DESIGN

`spawn_breaker` initializes `Scale2D { x: config.width, y: config.height }` (e.g., 120.0 × 20.0).
`sync_breaker_scale` writes `effective_size()` result (absolute dimensions) to `Scale2D`.
Breaker sprite is `Rectangle::new(1.0, 1.0)` — scaled to pixel dimensions.
This is an absolute-dimension semantic for `Scale2D`, intentionally different from ratio-multiplier.
**Do NOT re-flag as "incorrect use of Scale2D".**

## WallBuilder Floor::compute_position does NOT add half_thickness — CONFIRMED CORRECT

`Floor::compute_position` returns `(0.0, playfield_bottom)` — no `+ ht` offset.
This matches `second_wind/system.rs`. The wall spans BELOW the playfield edge by `ht` (AABB half_extents),
so the center is at the edge. Asymmetry vs Ceiling (which DOES add `ht`) is intentional.

## sync_breaker_scale in Update vs collision systems in FixedUpdate — CONFIRMED CORRECT

`sync_breaker_scale` (Update) writes `Scale2D` (visual only). Collision systems in FixedUpdate
read `BaseWidth`/`BaseHeight`/`ActiveSizeBoosts`/`NodeScalingFactor` DIRECTLY — they do not read `Scale2D`.
No ordering dependency. Intentional split.

## .definition() builder omits BoltRespawnOffsetY and BoltRespawnAngleSpread — RESOLVED (Wave 6)

`BoltRespawnOffsetY`, `BoltRespawnAngleSpread`, and `BoltInitialAngle` were **deleted** from the
codebase in Wave 6. `LostBoltData` now queries `BoltSpawnOffsetY` and `BoltAngleSpread`.
The original latent bug was eliminated by removing the component split. **Do NOT re-flag.**

## PrimaryBreaker marker never inserted on spawned breaker — RESOLVED (Breaker Builder Pattern)

`spawn_or_reuse_breaker` calls `Breaker::builder()...primary()...spawn(&mut commands)`.
The builder's `.primary()` method inserts `PrimaryBreaker` and `CleanupOnRunEnd` alongside `Breaker`.
`DispatchInitialEffects` and `resolve_default` queries for `(With<Breaker>, With<PrimaryBreaker>)`
now find the breaker entity correctly. **Do NOT re-flag.**

## ShieldActive — ELIMINATED (Shield refactor, 2026-04-02)

`ShieldActive` NO LONGER EXISTS. Shield is now a timed visible floor wall (`ShieldWall` + `ShieldWallTimer`).
`bolt_lost` and `handle_cell_hit` no longer reference `ShieldActive`.
**Do NOT re-flag the absence of ShieldActive charge-decrement patterns.**

See `reviewer-architecture/shield_cross_domain_write.md` for the full elimination record.

## death_pipeline: KillYourself<Breaker> dead-letter — RESOLVED (2026-04-14 Wave F1 scope expansion)

`handle_kill<Wall>` is now registered in `plugin.rs` (Wall path handled by generic handler).
`handle_breaker_death` is registered in `RunPlugin` in `DeathPipelineSystems::HandleKill` order set.

**Residual structural notes** (confirmed correct, do not re-flag):
- `GameState::Playing` gate on `handle_kill<Wall>` / `handle_kill<Breaker>` vs. no gate on `handle_kill<Cell>` — intentional scope asymmetry; walls and breaker only die during active play.
- `Destroyed<Breaker>` message is never written — `handle_breaker_death` does not send it. The breaker death path triggers `RunOutcome::Lost` directly; no downstream consumer reads `Destroyed<Breaker>`. Not a bug.
- `KillYourself<Breaker>` sending its own death via `KillYourself` is the correct self-destruct pattern. The component is removed after fire. No infinite loop.
