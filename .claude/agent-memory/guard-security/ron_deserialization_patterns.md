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
