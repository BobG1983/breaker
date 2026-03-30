---
name: Phase 3 file length findings (archived)
description: Files flagged after feature/stat-effects (Phase 3); all have been split by c9964b7 refactor (2026-03-30) — archived for reference
type: project
---

All files flagged in Phase 3 have been split by refactor commit c9964b7 (2026-03-30).

## Previously HIGH priority (now split)

- `breaker-game/src/chips/offering.rs` — now `chips/offering/` dir (system.rs + tests.rs)
- `breaker-game/src/effect/triggers/impacted.rs` — now `impacted/` dir (system.rs + tests.rs)
- `breaker-game/src/effect/triggers/impact.rs` — now `impact/` dir (system.rs + tests.rs)

## Previously MEDIUM priority (now split)

- `breaker-game/src/bolt/systems/bolt_wall_collision.rs` — now `bolt_wall_collision/` dir
- `breaker-game/src/effect/core/types.rs` — now `core/types/` dir (definitions.rs + tests.rs)

## Effect files confirmed clean (unchanged)

All `effect/effects/*.rs` files (attraction, chain_bolt, damage_boost, gravity_well, piercing,
quick_stop, ramping_damage, shield, shockwave, size_boost, speed_boost, etc.) remain 22–273 lines.

See `phase4_findings.md` for the current open findings.
