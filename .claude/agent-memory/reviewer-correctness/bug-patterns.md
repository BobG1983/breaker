---
name: Bug Patterns
description: Known bugs (fixed), recurring bug categories, and ECS pitfalls found
type: reference
---

## Known Bug Patterns (Fixed)
- **Double-tap consume uses 0.0 not NEG_INFINITY**: FIXED — `read_input_actions` uses `f64::NEG_INFINITY`.
- **check_valid_breaker_state missing Settling→Dashing transition**: FIXED.
- **check_valid_breaker_state Local tracks last entity only**: Documented; not re-flagged.

## Recurring Bug Categories
- **Stale screen resources**: RunSetupSelection, PauseMenuSelection, etc. persist between visits. Safe because `insert_resource` overwrites on re-entry.
- **Stale selection index**: All reset to 0 by spawn systems on OnEnter. Safe.
- **seed_upgrade_registry Local<bool>**: Persists for app lifetime. Correct — Loading only runs once.

## ECS Pitfalls Found
- `apply_bump_velocity` collects messages into Vec before querying — correct pattern for borrow conflicts.
- `ChipSelected` message has no consumer yet. Fire-and-forget, no ECS error.
- `spawn_chip_select` takes `Res<ChipRegistry>` (not Option) — guaranteed safe because Loading completes first.
