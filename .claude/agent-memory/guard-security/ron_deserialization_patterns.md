---
name: ron_deserialization_patterns
description: Confirmed safe RON deserialization patterns and production panic surface audit
type: project
---

Audited 2026-03-19 (develop, commit 7256360). Updated 2026-03-20 (feature/overclock-trigger-chain) to add chip/overclock RON patterns. Updated 2026-03-21 (develop, post-SpeedBoost refactor) to add SpeedBoost.multiplier finding.

## Summary

All RON parsing uses the `ron` 0.12 crate via `ron::de::from_str` (typed deserialization with
serde-derived `Deserialize` impls). No custom deserializers. No call site has unvalidated
`from_str` on runtime user input.

## Production path — via Bevy asset system

All game data files are loaded via `RonAssetPlugin` + `bevy_asset_loader` using hard-coded asset
paths (declared in `DefaultsCollection` with `#[asset(path = "...")]` macros). There is no runtime
path construction using user input.

Font paths from RON (`font_path`, `title_font_path`, `menu_font_path`) are passed to
`asset_server.load()`. These paths come from Bevy config RON files, not from user input, and
Bevy's asset server restricts paths to the assets directory. No path traversal risk in practice.

## Production path — scenario runner

The scenario runner uses `fs::read_to_string` + `ron::de::from_str` on scenario files discovered
by walking `scenarios/` at a compile-time-pinned path (`env!("CARGO_MANIFEST_DIR")/scenarios`).
Malformed RON files cause an error log + skip, not a panic (`discovery.rs:73-75` uses `.ok()?`).

## Test-only panics

All `expect()`/`unwrap()` on RON parsing in `cells/resources.rs`, `run/node/definition.rs`,
`breaker/resources.rs`, `bolt/resources.rs`, `chips/definition.rs`, etc. are inside
`#[cfg(test)] mod tests` blocks. They do not execute at runtime.

## Warning: RON hp/regen_rate not validated at runtime

`CellTypeDefinition.hp` and `CellBehavior.regen_rate` are deserialized from `.cell.ron` files
without bounds checks in the asset loader path. A `.cell.ron` with `hp: -1.0` or
`regen_rate: Some(999999.0)` would load silently and cause downstream logic issues. Test code
validates `hp > 0.0` but only in `#[cfg(test)]`.

**Status as of 2026-03-20 audit:** Still unvalidated. No runtime validation added yet.

**How to apply:** On future audits, check whether validation has been added to the runtime
asset-loaded path (the system that processes `CellTypeAsset` events and populates `CellTypeRegistry`).

## Warning: TriggerChain stacking fields have no bounds validation (added 2026-03-20)

`TriggerChain::Shockwave.base_range`, `range_per_level`, `MultiBolt.base_count`, `count_per_level`,
`Shield.base_duration`, `duration_per_level`, and `stacks` are all deserialized from `.overclock.ron`
files (and `initial_overclocks` in `.scenario.ron`) without any bounds check.

Concrete risks:
- `base_range: 1e30` in a shockwave hits all cells in the scene simultaneously. Safe at the entity
  level (handle_cell_hit dedup prevents double CellDestroyed), but all cells destroyed in one frame.
- `base_count: u32::MAX` in MultiBolt queues a huge number of bolt spawns — potential hang/OOM.
- `AmpEffect::DamageBoost(f32)` accepts negative values. `DamageBoost(-2.0)` makes shockwave damage
  negative (`BASE_BOLT_DAMAGE * (1 + (-2.0)) < 0`), which heals cells via `take_damage`. Cells
  with `hp = max` would never be destroyed — node completion impossible.

**Status as of 2026-03-20:** All unvalidated. First-party data only, no external input path.
OnPerfectBump → OnImpact nesting means deeply nested chains are author-controlled. RON parser
recursion limit (~128) provides a practical cap on chain depth.

**How to apply:** On future audits, check for runtime validation added to the chip asset loader
path and to `CellHealth::take_damage` (negative amount guard).

## Warning: TriggerChain::SpeedBoost.multiplier has no bounds validation (added 2026-03-21)

`TriggerChain::SpeedBoost { multiplier: f32 }` is deserialized from `.archetype.ron` files without
any bounds check. The handler in `src/behaviors/effects/speed_boost.rs` applies `bolt_velocity.value
*= *multiplier` directly.

Concrete risks:
- `multiplier: 0.0` collapses velocity to zero. The `speed > 0.0` floor guard (`line 57`) correctly
  skips re-normalizing a zero vector (uses `normalize_or_zero`), so no NaN — but the bolt becomes
  motionless and the game soft-locks.
- `multiplier: -1.0` reverses the velocity direction. The `speed > 0.0` floor guard only checks
  magnitude, not sign, so a negative multiplier passes through. The bolt now travels in the
  opposite direction. The `BoltInBounds` invariant and OOB detection should eventually catch it, but
  this is an authored foot-gun.
- `multiplier: 1e10` hits the `max_speed` clamp and is safe (no panic, no NaN). Max-speed clamp
  path is exercised by test `handle_speed_boost_clamps_to_max_speed`.

**Status as of 2026-03-21:** Unvalidated. First-party data only. All production RON files use
safe positive values (1.1, 1.5). The zero-velocity path is explicitly covered by test
`handle_speed_boost_zero_velocity_remains_zero` and does not panic. Scenario invariant
`BoltSpeedInRange` would catch a motionless bolt at runtime.

**How to apply:** On future audits, check if a `multiplier > 0.0` assertion has been added to the
archetype asset loader path or inside `handle_speed_boost`.
