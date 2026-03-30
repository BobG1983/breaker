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
- `effective_damage = BASE_BOLT_DAMAGE * EffectiveDamageMultiplier`
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
