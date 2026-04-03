---
name: ron_deserialization_patterns
description: RON deserialization patterns confirmed safe or flagged in the codebase
type: project
---

## Confirmed safe patterns (as of 2026-03-28)

### Chip templates (amp.chip.ron etc.)
- Loaded via Bevy's asset pipeline using `SeedableRegistry` trait
- Deserialization errors propagate through the asset system; they do not panic production code
- Field renames (e.g., `bonus_per_hit` → `damage_per_trigger` in RampingDamage) must be
  applied consistently: Rust struct field + all RON files + all tests. The Phase 1 cleanup
  did this correctly — confirmed by grepping for old field name (none found).
- Path construction uses `asset_dir()` string constants (no user input, no path traversal risk)

### Config defaults (include_str! macros)
- `defaults.cells.ron`, `defaults.breaker.ron`, `defaults.bolt.ron`, etc. are embedded
  at compile time with `include_str!`. Deserialization happens in `#[test]` functions with
  `.expect()`. A malformed file causes a test failure (not a production panic) since
  `include_str!` bakes the content at compile time.
- Production deserialization uses Bevy's asset loader with error handling.

### No injection risk
- RON files are shipped with the game binary; there is no mechanism for end-user-supplied
  RON to reach the deserializer at runtime (no save file loading, no mod system yet).
- Asset paths are all hardcoded string literals — no user-provided path components.

## Panic surface in RON handling

### Production code: none (all via Bevy asset system)
### Test-only panics (expected/by design):
- `cells/resources.rs:168` — `.expect("cells RON should parse")` — test function only
- `cells/resources.rs:180-181` — `unwrap_or_else(|e| panic!(...))` — test function only

## Phase 3 new findings (effect domain, 2026-03-28)

### Known-safe division sites in formula systems
- `bolt_breaker_collision/system.rs:195` — `bolt_velocity.0 / speed` — guarded by
  `if speed < f32::EPSILON { continue; }` at line 191. Safe.
- `dash/system.rs:123, 185, 207` — timer progress divisions — all guarded by
  `> f32::EPSILON` before dividing. Safe.
- `dash/system.rs:253` — `speed / reference_speed` in `eased_decel` — guarded. Safe.

### Unguarded division: hit_fraction (Warning-level)
- `bolt_breaker_collision/system.rs:80` — `((clamped_x - breaker_x) / half_w).clamp(-1.0, 1.0)`
- `half_w` = `breaker_width.half_width() * EffectiveSizeMultiplier * EntityScale`
- `EffectiveSizeMultiplier` is the product of all `ActiveSizeBoosts` entries
- Chip templates include negative SizeBoost values (e.g. `SizeBoost(-0.5)` in splinter.chip.ron)
- An odd number of negative-factor SizeBoost applications makes `EffectiveSizeMultiplier`
  negative; when negative half_w feeds into `impact_x.clamp(lo, hi)` with lo > hi,
  Rust debug builds PANIC. Release builds return `lo` (garbage value).
- This is a Warning-level issue: requires non-obvious chip stacking, game design likely
  prevents it in practice, but the guard is missing at the formula layer.

### Semantic bug: negative damage (Warning-level)
- `handle_cell_hit/system.rs:45` — `health.take_damage(msg.damage)` with no sign guard
- `effective_damage = BASE_BOLT_DAMAGE * ActiveDamageBoosts::multiplier()` (EffectiveDamageMultiplier removed; multiplier computed on demand)
- `gauntlet.chip.ron` uses `DamageBoost(-0.5)` — a negative multiplier factor
- Product of two `DamageBoost(-0.5)` stacks = 0.25 (fine); product with `DamageBoost(-0.5)`
  and `DamageBoost(0.1)` could produce negative effective damage
- Negative damage heals cells (not a panic, but a logic error)

### RampingDamageState::accumulated not yet consumed (Info-level)
- `ramping_damage.rs` accumulates damage bonus in `RampingDamageState::accumulated`
- As of Phase 3, no system reads `accumulated` to add it to effective damage
- This is expected for Phase 3 (state tracking complete, consumption wired later)

## Phase 4+5 new findings (runtime effects, 2026-03-28, feature/runtime-effects)

### Evolution chip RON files — no new panic risk
- New `.evolution.ron` files (phantom_breaker, voltchain, supernova, gravity_well,
  second_wind, split_decision, nova_lance, etc.) all loaded via Bevy asset pipeline
- All numeric values are bounded literals, no user-provided strings that feed into
  paths or format strings. No injection risk.
- `SpawnPhantom(duration: 5.0, max_active: 1)` — `max_active: 1` is a `u32` field;
  worst-case a large RON value causes excessive phantom despawning but not a panic.

### spawn_phantom.rs: max_active=0 is a silent no-op (Info-level)
- `spawn_phantom::fire`: `while owned.len() >= max_active as usize` — if `max_active=0`,
  the condition is `owned.len() >= 0` which is always true, so it despawns all existing
  phantoms then immediately spawns one, leaving count=1 instead of 0.
- The RON files set `max_active: 1`; this edge case doesn't occur in practice.
- Not a panic, not a security issue. Info-level only.

### spawn_phantom.rs: Vec::remove(0) is O(n) (Info-level)
- `while owned.len() >= max_active as usize { owned.remove(0); }` — O(n) per removal.
- `max_active` caps at the RON-configured value (1 in the current evolution chip),
  so at worst removes a handful of elements. Not a panic surface.

### PulseDamaged / ShockwaveDamaged HashSets — unbounded growth (Info-level)
- `PulseDamaged(pub HashSet<Entity>)` and `ShockwaveDamaged(pub HashSet<Entity>)`
  grow by one entry per unique cell hit. Cell counts are bounded by the level layout
  (small fixed grid); not a memory concern in practice.

### apply_pulse_damage / apply_shockwave_damage: radius > 0.0 guard is correct (Safe)
- Both systems guard `if radius.0 <= 0.0 { continue; }` before querying the quadtree.
  Zero-radius query would query nothing; the guard is defensive and safe.

### Confirmed safe: no division by zero in new effect code
- `effective_max_radius` in pulse.rs: `base_range + f32::from(extra) * range_per_level`
  — pure multiply-add, no division.
- `shockwave::fire` effective range computation: same pattern, no division.
- `apply_attraction` uses `normalize_or_zero()` — zero vector returns Vec2::ZERO safely.

### attraction.rs: velocity accumulates unboundedly (Warning-level)
- `apply_attraction` adds `direction * nearest_force * dt` to velocity each fixed tick.
- No speed cap is applied inside attraction.rs itself. The bolt's speed-clamping system
  (`clamp_bolt_speed`) is expected to cap velocity after attraction runs.
- If attraction runs WITHOUT the speed clamp (e.g., in isolated unit tests or if system
  ordering is wrong), velocity can grow without bound. Not a crash, but a gameplay logic
  issue with security-adjacent risk (unbounded velocity could bypass CCD bounds).
- Confirmed: in production the systems are ordered correctly (attraction before physics
  resolution, speed clamp also in FixedUpdate). Info/Warning-level only.

## Phase 6 new findings (source-chip threading + shield absorption, 2026-03-29, feature/source-chip-shield-absorption)

### arc_speed serde default — zero/negative arc_speed is guarded at fire() (Safe)
- `EffectKind::ChainLightning::arc_speed` is a new RON field with `#[serde(default =
  "default_chain_lightning_arc_speed")]` → default value 200.0.
- The only existing RON using ChainLightning (`voltchain.evolution.ron`) omits `arc_speed`,
  so it will receive the serde default of 200.0 at load time. Correct and safe.
- `fire()` in chain_lightning/effect.rs guards `if arc_speed <= 0.0 { return; }` before
  any use. A RON file setting `arc_speed: 0.0` or negative results in a silent no-op,
  not a panic. This is the same defensive pattern used for `arcs` and `range`.

### EffectSourceChip::new with whitespace-only strings (Info-level)
- `chip_attribution(" ")` returns `Some(" ".to_string())` (tested explicitly).
- A chip RON file whose `name` field is a single space would produce a
  `EffectSourceChip(Some(" "))` attribution. This is not a panic or crash — the string is
  stored as-is in the DamageCell message and used for display/scoring only.
- No user-controlled input reaches chip names; they are hardcoded RON asset strings.

### remaining_jumps underflow in tick_chain_lightning (Safe)
- `chain.remaining_jumps -= 1` at effect.rs:254 is only reached in the `ArcTraveling`
  branch, after `fire()` has already verified `remaining_jumps >= 1` by spawning the chain
  only when `arcs > 1` (remaining_jumps = arcs - 1 ≥ 1). The `Idle` branch guards
  `remaining_jumps == 0` before selecting a target — it despawns without entering
  `ArcTraveling`. No underflow path exists. Safe.

### std::mem::replace pattern in tick_chain_lightning (Safe)
- `std::mem::replace(&mut chain.state, ChainState::Idle)` is used to move state out of the
  mutable borrow for matching. This is idiomatic safe Rust; no unsafe involved.

### ShieldActive::charges underflow on cell absorb (Safe)
- `shield.charges -= 1` in handle_cell_hit/system.rs is guarded by `shield.charges > 0`
  immediately above it. No underflow possible.

### source_chip String allocation per fire/reverse call (Info-level)
- `fire_effect` and `reverse_effect` now accept `source_chip: String` (owned) rather than
  `&str`. This allocates one String per queued command. In a typical frame with a handful
  of triggered effects this is negligible. No security concern; noted for performance
  awareness if this becomes a hot path.

## feature/missing-unit-tests (2026-03-30)

No new RON files, no new deserialization sites. Carry-forward warnings from Phase 3
(hit_fraction unguarded division, negative damage) remain unchanged. No new panic surface
in production code.

## Refactor (2026-03-30, develop post-merge, c9964b7)

### File-splitting structural refactor — no new panic surface introduced (Safe)
- 23 .rs files were split into directory modules (mod.rs wiring + system.rs + tests.rs).
- All production RON deserialization sites are unchanged; only file/module layout changed.
- Specifically confirmed: bolt_breaker_collision/system.rs:80 hit_fraction division
  (Warning-level, carry-forward), cells/components/types.rs:114 take_damage with no
  sign guard (Warning-level, carry-forward) — both unchanged by the refactor.
- No new .expect() or .unwrap() calls in production (non-test, non-debug) code.
- All new .expect()/.unwrap() occurrences are inside #[cfg(test)] modules or
  debug/hot_reload/ (dev-feature only, never in release build).

## Wave 3 audit (2026-03-30, feature/scenario-coverage — tether_beam chain mode, spawn_bolts inherit fix, TetherBeam RON variant)

### TetherBeam chain field serde default — zero-bolt edge case is safe (Info-level)
- `EffectKind::TetherBeam::chain` uses `#[serde(default)]` → defaults to `false` when omitted.
- RON serde tests in enums.rs confirm round-trip for both `chain: true` and omitted `chain`.
- `fire_chain` with zero bolts: `bolts.windows(2)` produces an empty iterator — no beams
  spawned, but `TetherChainActive` is still inserted with `last_bolt_count: 0`.
  A malformed RON file or pathological scenario that fires chain mode when no bolts exist
  produces a dangling resource with zero last_bolt_count. This is not a panic: the
  `maintain_tether_chain` system re-runs each frame but `bolt_count == 0 == last_bolt_count`
  so the body is skipped. Resource is removed by `cleanup_tether_chain_resource` on
  `OnExit(GameState::Playing)`. Benign, but a TetherChainActive resource with no beams
  and no bolts persists until node exit. Info-level only.

### TetherChainActive resource cleanup: OnExit(GameState::Playing) is correct (Safe)
- `cleanup_tether_chain_resource` is registered on `OnExit(GameState::Playing)`.
- `CleanupOnNodeExit` entities (including all TetherBeamComponent entities) are also
  despawned on `OnExit(GameState::Playing)` in the state cleanup system (previously `screen/plugin.rs` — the screen domain was eliminated in the state lifecycle refactor; cleanup is now in `state/cleanup.rs`).
- Both resource and beam entities are cleaned up at the same state-exit hook.
  No race condition between resource removal and entity despawn.
- `reverse()` for chain mode removes `TetherChainActive` and despawns chain beams
  immediately at `Until` trigger resolution — this is the normal in-gameplay path.
  The `OnExit` hook is a safety net for abnormal exits (e.g., game over, back to menu).

### spawn_bolts inherit fix: query_filtered now includes `Without<ExtraBolt>` (Safe)
- `spawn_bolts/effect.rs:27` — `query_filtered::<&BoundEffects, (With<Bolt>, Without<ExtraBolt>)>()`
- The `Without<ExtraBolt>` filter prevents infinite effect-inheritance chains when
  `SpawnBolts(inherit: true)` fires on a bolt that is itself an extra bolt.
- No panic surface: if no primary bolt entity exists the iterator is empty and
  `bound_effects` is `None` — spawned bolts get no inherited effects. Silent no-op.
- `lifespan: None` → `BoltLifespan` not inserted → extra bolt lives until node exit.
  This is correct and intentional.

### damage computation in tick_tether_beam: no zero-bolt-distance guard (Warning-level)
- `breaker-game/src/effect/effects/tether_beam/effect.rs:220-221`
  ```rust
  let beam_vec = pos_b - pos_a;
  let max_dist = beam_vec.length();
  let direction = beam_vec.normalize_or_zero();
  ```
- If `bolt_a` and `bolt_b` are at the same position, `max_dist = 0.0` and
  `direction = Vec2::ZERO`. `ray_vs_aabb` is called with `direction = Vec2::ZERO`
  and `max_dist = 0.0`. This is not a panic — `normalize_or_zero()` prevents NaN —
  but the ray test result depends on the physics library's handling of a zero-length
  ray. If `ray_vs_aabb` returns `Some` for a zero-length ray starting inside the
  AABB, all cells near the coincident bolt positions will be damaged every tick.
  The `origin_inside` check on line 245 also fires in that case, so damage is
  delivered regardless. Not a crash, but unexpected behavior when two tether bolts
  collide to the same pixel. Design-level note: the game likely prevents this via
  DistanceConstraint or natural physics separation. Warning-level.

### No new RON deserialization panic surface (Safe)
- `arcwelder.evolution.ron` uses `TetherBeam(damage_mult: 1.5, chain: true)` —
  both fields are validated f32/bool. No injection risk.
- `tether_beam_stress.scenario.ron` uses `TetherBeam(damage_mult: 1.5)` — chain
  defaults to false via serde default. Correct.
- All scenario RON files loaded via `load_scenario()` which returns `Option` — no panic.

## Cache removal refactor (2026-03-30, commits d6d9b80 + 2bdb81b)

### InjectWrongEffectiveSpeed and InjectWrongSizeMultiplier mutations removed (Safe)
- Two frame mutation variants removed from scenario runner: InjectWrongEffectiveSpeed
  and InjectWrongSizeMultiplier. Both were scenario runner-internal test tools.
- Their corresponding RON self-test files also deleted:
  effective_speed_consistent.scenario.ron, size_boost_in_range.scenario.ron.
- These were internal to the scenario runner tool (never user-facing data).
  No production panic surface affected.

### Confirmed safe: on-demand multiplier computation now used everywhere (Safe)
- All systems that previously may have used EffectiveSpeedMultiplier / EffectiveSizeMultiplier
  components now call ActiveSpeedBoosts::multiplier() / ActiveSizeBoosts::multiplier() on demand.
- Specifically: apply_velocity_formula() in bolt/queries.rs (replace for prepare_bolt_velocity, which was DELETED),
  bolt_breaker_collision, move_breaker, dash/system.rs.
- Both multiplier() methods use if self.0.is_empty() { 1.0 } else { self.0.iter().product() }
  — no division, no panic surface.

### Stale BUG comment (MOOT — file deleted)
- `src/bolt/systems/prepare_bolt_velocity/tests.rs:288` — this file and its tests were
  DELETED in the bolt builder migration (feature/chip-evolution-ecosystem). The stale BUG
  comment no longer exists. No action needed.

## feature/chip-evolution-ecosystem (2026-03-31) — bolt builder migration

### No new RON deserialization sites (Safe)
- defaults.bolt.ron: unchanged schema — all existing fields, no new fields added.
- defaults.breaker.ron: added reflection_spread field. Tested in
  breaker/resources.rs::tests::breaker_defaults_ron_parses with .expect() inside
  #[cfg(test)] — not production panic surface. The serde field has no default
  attribute, so a malformed breaker RON omitting reflection_spread would fail
  deserialization. Since the RON is include_str! at compile time in tests and loaded
  via Bevy's asset pipeline at runtime (returns error, not panic), this is safe.
- BreakerConfig: new `reflection_spread: f32` field has no bounds validation.
  Values <= 0.0 or > 360.0 are not rejected. However, reflection_spread is used
  in bolt_breaker_collision/system.rs — an invalid value (zero or negative) would
  produce degenerate angle ranges. Since this is a compile-time asset, not
  user-controlled input, risk is Info-level only.

### OptionalBoltData::radius uses unwrap_or(DEFAULT_RADIUS) in production (Safe)
- build_core() at builder.rs:281 — `optional.radius.unwrap_or(DEFAULT_RADIUS)`.
  This is Option::unwrap_or, not .unwrap() — the default (8.0) is always returned
  if radius is None. No panic path. Safe.

### All .unwrap()/.expect() in bolt/builder.rs are inside #[cfg(test)] (Safe)
- 100+ unwrap/expect calls in builder.rs are test assertions on world.get::<T>(entity).
  All enclosed in #[cfg(test)] mod tests block. Not production panic surface.

## feature/chip-evolution-ecosystem (2026-04-01) — chip ecosystem + new effects

### New evolution and template RON files — no new panic risk (Safe)
- 12 new .evolution.ron files and 5 modified .chip.ron files. All loaded via Bevy
  asset pipeline. No new user-controlled input reaches any deserializer.
- All numeric fields are bounded literals. No injection risk.
- New EffectKind variants in RON: Anchor, CircuitBreaker, MirrorProtocol, EntropyEngine.
  All well-formed in shipped files.

### circuit_breaker.evolution.ron bumps_required: 3 (Safe as-is; Warning at code layer)
- The current RON sets bumps_required: 3. Safe.
- However: circuit_breaker/effect.rs:73 computes `config.bumps_required - 1` as bare u32
  subtraction with no guard. If a RON file ever sets bumps_required: 0, this underflows
  in debug mode (panic) and wraps in release (u32::MAX → immediate "reward" on first bump,
  gameplay nonsense). See Warning finding below.
- The only production RON file uses bumps_required: 3. Risk is theoretical but real.

### EntropyEngine pool: empty pool and all-zero weights guarded (Safe)
- entropy_engine/effect.rs:47 — `if pool.is_empty() { warn!(...); return; }` guards empty pool.
- entropy_engine/effect.rs:60-62 — `let Ok(dist) = WeightedIndex::new(...) else { warn!(...); return; }` guards all-zero weights.
- No panic surface. Both edge cases produce a `warn!` log and silent no-op.

### EntropyEngine max_effects: 0 is a silent no-op (Info-level)
- effects_to_fire = cells_destroyed.min(max_effects). If max_effects = 0, effects_to_fire = 0
  and the function returns early at line 52. No panic, no behavior.

### Anchor plant_delay: 0.0 is a silent instant-plant (Info-level)
- AnchorTimer(0.0) is inserted on first stationary frame; the `t.0 <= 0.0` check fires
  on the next tick. Instant plant. Not a panic, not harmful.

### All new .expect()/.unwrap() in production code — Safe
- bolt/builder.rs:281 — `unwrap_or(DEFAULT_RADIUS)` — not a panic path.
- All other .expect()/.unwrap() in changed files are inside #[cfg(test)] blocks.
  Verified by checking #[cfg(test)] block start lines vs. first .expect() line.

### defaults.breaker.ron: new reflection_spread field (Info-level, carry-forward)
- Confirmed in prior audit note (2026-03-31). No new findings.

## wall builder feature (2026-04-02) — wall builder migration

### WallDefinition RON schema — all safe (Safe)
- `#[serde(deny_unknown_fields)]` on WallDefinition — unknown fields are errors, not silently swallowed.
- All fields except `name` have `#[serde(default)]`. A RON file with only `(name: "Wall")` is valid.
- `half_thickness: f32` defaults to 90.0 via a default function. No division by zero risk: half_thickness
  is used only in arithmetic (addition, subtraction) for wall position and extents — never as a divisor.
  A RON file setting `half_thickness: 0.0` produces a zero-extent wall (invisible, physics degenerate),
  but does not panic. No validation guard exists — Info-level only.
- `color_rgb: Option<[f32; 3]>` — HDR values exceeding 1.0 are permitted by design for bloom.
  Values are passed to `color_from_rgb()` and into Bevy's ColorMaterial. No injection risk.
- `effects: Vec<RootEffect>` — nested via existing RootEffect deserialization, which is already vetted.
- Production deserialization via Bevy asset pipeline (returns asset error, not panic).
- Test deserialization via ron::de::from_str(...).expect() inside #[cfg(test)] only.
- One shipped RON: assets/walls/wall.wall.ron — contains only `name: "Wall"`, all defaults apply.

### WallRegistry::seed() — warn-and-skip for duplicates (Safe — better than BreakerRegistry)
- Unlike BreakerRegistry::seed() which uses `assert!` on duplicate names (panic on collision),
  WallRegistry::seed() uses `warn!(...); continue;` — a duplicate wall name logs a warning and
  the second definition is silently skipped. No production panic path. The first-wins behavior
  is tested and documented.

### Wall::builder() half_thickness resolution — no panic path (Safe)
- resolve_half_thickness() uses Option::or().unwrap_or(DEFAULT_HALF_THICKNESS).
  unwrap_or is not unwrap() — always returns a fallback. No panic.

### second_wind/system.rs — no panic path (Safe)
- fire() uses world.resource::<PlayfieldConfig>().clone() — panics if PlayfieldConfig is absent,
  but this resource is always initialized at app startup. Same pattern as other effect fire() fns.
- despawn_second_wind_on_contact: wall_query.get(msg.wall).is_ok() before any entity access. Safe.
- No RON deserialization in second_wind — pure runtime spawning via Wall::builder().

## feature/breaker-builder-pattern (2026-04-02) — breaker builder migration

### BreakerDefinition expanded to 35+ serde-defaulted fields (Safe)
- `#[serde(deny_unknown_fields)]` remains on the struct — unknown fields are errors, not silently swallowed.
- All 35+ fields have `#[serde(default = "default_fn")]` or `#[serde(default)]`.
- RON files only need `name:` to be valid. Malformed field values propagate as Bevy asset errors
  (not production panics).
- Three new assets: aegis.breaker.ron, chrono.breaker.ron, prism.breaker.ron. All include_str!
  verified in #[cfg(test)] and confirmed parseable.
- Extension changed from `bdef.ron` to `breaker.ron` in BreakerRegistry::extensions().

### spawn_bolt: remove_resource + unwrap_or_default pattern (Safe)
- `breaker-game/src/bolt/systems/spawn_bolt/system.rs:97-100`
- `world.remove_resource::<Assets<Mesh>>().unwrap_or_default()` and same for ColorMaterial.
- `unwrap_or_default()` is NOT `.unwrap()` — it returns an empty `Assets<T>` if the resource
  is absent. This means if Bevy ever initializes the app without asset resources (impossible in
  normal running, but possible in narrow unit-test setups), the builder will create meshes in a
  fresh empty Assets store that is then re-inserted. No panic path.
- After spawn, both resources are re-inserted unconditionally. The resource is temporarily absent
  between remove and insert — any other system running concurrently that tries to read these
  resources would find them missing. This is safe because `spawn_bolt` is an exclusive World
  system (takes `&mut World`) — it serializes with all other systems. No concurrency gap.

### BreakerRegistry::seed() assert! is production code (Warning — pre-existing)
- `breaker-game/src/breaker/registry.rs:82` — `assert!(!self.breakers.contains_key(&def.name), "duplicate breaker name...")`
- This assert! is inside `SeedableRegistry::seed()`, which is a trait impl called during
  asset loading, NOT inside `#[cfg(test)]`. A duplicate breaker name in any two `.breaker.ron`
  files would panic at asset load time.
- Same pattern exists in `BoltRegistry::seed()` at bolt/registry.rs:82.
- In practice, all shipped RON files have unique names. The risk is a developer adding a new
  breaker RON file with a name collision. The `assert!` itself was pre-existing (not introduced
  on this branch); this branch only renames the extension from `bdef.ron` to `breaker.ron`.

### dispatch_initial_effects recursive On-chain (Info-level, new command)
- `ext.rs`: `DispatchInitialEffects` → `TransferCommand` → `ResolveOnCommand` → `TransferCommand`...
- Recursion depth is bounded by the nesting depth of `EffectNode::On` within `EffectNode::On`.
- Shipped breaker RON files use at most 2 nesting levels. No infinite cycle path (each
  `On` consumes its `then` children and dispatches to a resolved entity, not back to the
  same entity with the same data). Info-level only.

## feature/scenario-coverage (2026-03-30)

### New scenario RON files — no new panic risk (Safe)
- 8 new .scenario.ron files: 3 chaos, 1 mechanic, 3 self_tests, 1 stress.
- All parsed via `load_scenario()` in discovery.rs which returns `Option` (no panic on
  malformed input).
- `SpawnExtraGravityWells(usize)` count field: u size deserialized from RON. A very large
  value (e.g., usize::MAX) would cause iterative `commands.spawn()` per count — potential
  memory exhaustion in a dev scenario if a malformed RON file is loaded. This is a
  Warning-level finding unique to the scenario runner tool (never ships to end users).
  All shipped RON files use reasonable values (max 15 in self-tests, 32 in stress).
  Pattern is identical to the pre-existing SpawnExtraPulseRings and SpawnExtraChainArcs
  mutations which have the same unbounded-count behavior.

### New InvariantParams field max_gravity_well_count (Safe)
- Deserialized via serde with `#[serde(default)]` — missing field defaults to 10.
  A RON file setting max_gravity_well_count: 0 would mean any 1+ wells violate the
  invariant immediately. Not a panic; just a strict (possibly noisy) invariant.

### Entity::PLACEHOLDER in apply_spawn_extra_gravity_wells (Safe)
- Spawned test wells use owner: Entity::PLACEHOLDER. The gravity_well fire() system
  compares config.owner == entity to count per-owner wells. PLACEHOLDER never matches
  any real entity, so test wells bypass the per-owner cap and accumulate as intended.
  No panic risk.

### Production RON loading path unchanged (confirmed Safe)
- discovery.rs:load_scenario() uses .map_err(eprintln).ok() — malformed scenario files
  produce an error message and return None, not a panic. This is the same pattern as
  prior audits. No new production .expect()/.unwrap() on file-controlled data.

## Shield refactor (2026-04-02, commit e887570)

### EffectKind::Shield changed from {stacks: u32} to {duration: f32} (Safe)
- parry.chip.ron uses `Shield(duration: 5.0)` — positive f32 literal, no injection risk.
- duration: 0.0 and negative values are valid at the enum level (compile test confirms).
  Timer::from_seconds(0.0) is valid Bevy — timer starts already finished, wall is despawned
  on the next tick. Not a panic; wall is briefly visible for one frame at most.
  Negative duration: Timer::from_seconds negative — Bevy clamped to 0.0 internally (safe).
  Info-level only; RON files ship with `duration: 5.0`.

### fire() resource access pattern — sequential borrows (Safe)
- `world.resource_mut::<Assets<Mesh>>()` and `world.resource_mut::<Assets<ColorMaterial>>()`
  are called sequentially (one finishes, returns, then the other borrows). No aliasing.
  Each borrow ends before the next begins. Borrow checker enforces this at compile time.
  The comment in fire() at line 43-45 correctly explains the rationale.

### fire() world.resource::<PlayfieldConfig>() — panic-if-absent (Pre-existing pattern)
- Confirmed identical to second_wind/system.rs and other effect fire() fns.
  PlayfieldConfig is always inserted at app startup. Not a new panic surface.

### re-fire silent no-op: ShieldWall exists without ShieldWallTimer (Info-level)
- fire() at line 27: `if let Some(mut timer) = world.get_mut::<ShieldWallTimer>(wall_entity)`
  — if the existing ShieldWall entity somehow lacks ShieldWallTimer, the `if let` falls
  through and `return` is still reached. Timer is not reset, no new wall is spawned.
  Silent no-op. In practice this cannot happen: all spawning paths insert both components
  atomically via world.spawn((ShieldWall, ShieldWallTimer(...), ...)). Info-level only.

### ShieldActive deleted — no residual references in production code (Safe)
- ShieldActive is no longer a type anywhere in the .rs source tree. Confirmed by grep.
- bolt_lost.rs and handle_cell_hit/system.rs no longer reference ShieldActive.
  The reviewer-architecture memory file shield_cross_domain_write.md is now stale (describes
  the deleted component). It should be updated or removed by reviewer-architecture.

### No new RON deserialization panic surface (Safe)
- parry.chip.ron: only field change is Shield(duration: 5.0) replacing stacks-based variant.
  Loaded via Bevy asset pipeline. No production panic surface.
- shield_wall_at_most_one.scenario.ron, shield_wall_reflection.scenario.ron:
  parsed via load_scenario() returning Option — no panic on malformed input.
  SpawnExtraShieldWalls(2) in self-test: count is a small literal (2), no exhaustion risk.

## refactor/state-folder-structure (2026-04-02, commit d2440054)

### include_str path updates — test-only, no new panic surface (Safe)
- Two existing include_str! test calls updated paths: defaults.difficulty.ron and
  defaults.highlights.ron. The path change is from 3 levels up to 4 levels up (module moved
  deeper). No logic change. .expect() is inside #[cfg(test)] functions only.
  The RON files themselves are unchanged.

### TransitionConfig: out_duration/in_duration division without zero guard (Warning-level)
- `breaker-game/src/state/transition/system.rs:144`
  `let progress = 1.0 - (timer.remaining / timer.duration);`
- `timer.duration` is set from `config.out_duration` / `config.in_duration`, which are
  `f32` fields in `TransitionDefaults` loaded from RON (or defaulting to 0.5/0.3).
- The division only runs in the `else` branch (guarded by `timer.remaining > 0.0`), but
  `timer.duration` is independently set at spawn time from the config value. A RON file
  setting `out_duration: 0.0` or `in_duration: 0.0` would set `timer.remaining = 0.0` and
  `timer.duration = 0.0`. On the first tick, `timer.remaining -= dt` goes negative, the
  `if timer.remaining <= 0.0` branch fires, and the state transition happens immediately.
  The division at line 144 is never reached — so this is NOT a production panic path.
  The division is only reachable when remaining > 0.0 AND duration > 0.0 (since remaining
  was initialized from duration). If duration is changed after spawn to 0.0 (not currently
  possible), division would produce f32::INFINITY (NaN-free on IEEE 754 with nonzero
  remaining numerator). Reassessed: Info-level only in practice; worth noting for correctness.

### No new wall/state RON schemas (Safe)
- walls/ module is a rename from wall/ — WallDefinition schema and WallRegistry behavior
  are unchanged (carry-forward from wall builder audit 2026-04-02).
- state/ hierarchy is pure Rust code reorganization. No new asset types or deserializers.
