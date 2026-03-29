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
