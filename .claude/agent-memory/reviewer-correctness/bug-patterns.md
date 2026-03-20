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
- `ChipSelected` message is consumed by `apply_chip_effect` (ChipsPlugin). Consumer added in feature/phase4b1-chip-effects.
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

## Phase 4 Wave 2 Confirmed Bugs (2026-03-19, feature/phase4-wave2-session4) — OPEN

- **handle_node_cleared uses registry.len() instead of NodeSequence.assignments.len()**: `handle_node_cleared.rs:30` — game prematurely wins when NodeSequence has more nodes than layout registry count. Fix: read NodeSequence resource, use `assignments.len().saturating_sub(1)`.
- **spawn_cells_from_grid ignores CellBehavior**: `spawn_cells_from_layout.rs:57-83` — never reads `def.behavior.locked` or `def.behavior.regen_rate`. Lock/regen cell types defined in RON never get `Locked`/`LockAdjacents`/`CellRegen` components. Fix: read behavior fields and insert components after spawn.
- **NodeSequence hp_mult never applied to cells**: `spawn_cells_from_layout.rs:62` — uses `def.hp` directly, ignores NodeAssignment.hp_mult. All nodes same cell HP regardless of tier. Fix: pass hp_mult from current NodeSequence entry.
- **NodeSequence timer_mult never applied to timer**: `init_node_timer.rs:11` — uses `layout.timer_secs` directly, ignores NodeAssignment.timer_mult. All nodes same timer. Fix: multiply timer_secs * timer_mult from current NodeSequence entry.

## Phase 4b.2 Bolt Persistence Bugs (2026-03-19, feature/phase4b2-effect-consumption) — FIXED
- **reset_bolt spawns bolt at stale breaker x**: FIXED — `reset_bolt` now has `.after(BreakerSystems::Reset)` in `bolt/plugin.rs`. Confirmed in code.
- **bridge_bump_whiff can miss BumpWhiffed in same frame**: FIXED — `bridge_bump_whiff` now has `.after(BreakerSystems::GradeBump)` in `behaviors/plugin.rs`. Confirmed in code.
