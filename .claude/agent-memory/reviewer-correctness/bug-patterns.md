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

## Phase 4 Wave 1 Confirmed Bugs (2026-03-19)
- `stack_u32` (apply_chip_effect.rs:204): `*current / per_stack` panics with integer division by zero when `per_stack=0`. No guard. Current RON files are safe (non-zero values), but future chips with `Piercing(0)` or `ChainHit(0)` would panic at stack-2 selection time.
  **FIXED in refactor/phase4-wave1-cleanup**: `stack_u32` now has `if per_stack == 0 { return; }` guard at line 42 of effects/mod.rs.

## Phase 4 Wave 1 Cleanup Confirmed Bugs (2026-03-19)
- `spawn_chip_select` (spawn_chip_select.rs:23): `registry.values().take(MAX_CARDS)` iterates HashMap in non-deterministic order. Before this PR, registry was a Vec (deterministic insertion order). Now chip offers vary arbitrarily between runs even with the same seed, breaking run reproducibility.
  **FIXED in feature/phase4b2-effect-consumption**: Now uses `registry.ordered_values().take(MAX_CARDS)` — deterministic insertion-order iteration via `Vec<String>` parallel to the HashMap.

## Phase 4b.2 Effect Consumption Confirmed Bugs (2026-03-19)
- **Amp chip effects silently discarded**: All five Amp observers (handle_piercing, handle_damage_boost, handle_bolt_speed_boost, handle_chain_hit, handle_bolt_size_boost) in `chips/effects/` query `With<Bolt>`. But bolt entities have `CleanupOnNodeExit` and are despawned on `OnExit(GameState::Playing)` — which fires when `Playing → ChipSelect`. So during `ChipSelect`, no bolt entities exist. The observers iterate zero entities, silently discarding all Amp effects. Augment effects (Breaker-targeted) work correctly because `Breaker` has `CleanupOnRunEnd` and persists.
  - Reproducer: Select "Piercing Shot" amp in chip select; proceed to next node; no Piercing component on new bolt.

## Phase 4b.2 Bolt Persistence Bugs (2026-03-19, feature/phase4b2-effect-consumption)
- **reset_bolt spawns bolt at stale breaker x**: `reset_bolt` reads `(breaker_x, breaker_y)` from the breaker's current Transform to position the bolt. But `reset_bolt` runs in `OnEnter(Playing)` with ordering `.after(BoltSystems::InitParams)`, while `reset_breaker` runs `.after(BreakerSystems::InitParams)` in the same schedule. No ordering between these two Reset sets. If `reset_bolt` fires before `reset_breaker`, the bolt is placed at the breaker's pre-reset x (wherever it was at end of previous node), not the centered x. Old `spawn_bolt` always used x=0.0 hardcoded, so this is a behavioral regression. Fix: add `.after(BreakerSystems::Reset)` to `reset_bolt` wiring.
- **bridge_bump_whiff can miss BumpWhiffed in same frame**: `bridge_bump_whiff` in `behaviors/plugin.rs` runs `.after(PhysicsSystems::BreakerCollision)`. `grade_bump` in `breaker/plugin.rs` WRITES `BumpWhiffed` and also runs `.after(PhysicsSystems::BreakerCollision)`. No ordering constraint between the two systems. If `bridge_bump_whiff` runs before `grade_bump` in the same FixedUpdate tick, it reads zero messages and fires no consequences for that whiff. Fix: add `.after(grade_bump)` to `bridge_bump_whiff` wiring. Existing `bridge_bump` has the same relationship — it also runs `.after(PhysicsSystems::BreakerCollision)` and reads `BumpPerformed` written by `grade_bump`; but `bridge_bump` reads `BumpPerformed` while `update_bump` (not `grade_bump`) can also write it retroactively, so the impact is different.
