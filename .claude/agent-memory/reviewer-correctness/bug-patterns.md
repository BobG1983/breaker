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
- `apply_bump_velocity` DELETED (2026-03-21) — velocity scaling now via TriggerChain::SpeedBoost leaf → handle_speed_boost observer. The Vec-collection pattern for borrow conflicts was used here; apply it in any future systems with the same shape.
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

## Phase 4 Wave 2 OPEN Bugs — ALL FIXED (2026-03-19 second session)
All four bugs recorded as OPEN in Phase 4 Wave 2 are now confirmed FIXED in current codebase:
- `handle_node_cleared` now uses `NodeSequence.assignments.len()` ✓
- `spawn_cells_from_layout` now reads `def.behavior.locked` and `def.behavior.regen_rate` ✓
- `spawn_cells_from_layout` now uses `resolve_hp_mult()` for `hp_mult` ✓
- `init_node_timer` now reads `timer_mult` from `NodeSequence` ✓

## Full-tree Review Confirmed Bug (2026-03-19, second session) — FIXED (2026-03-19 third session)
- **spawn_run_end_screen shows wrong loss text for Aegis**: FIXED — `RunOutcome::Lost` split into `TimerExpired` and `LivesDepleted`. `handle_timer_expired` sets `TimerExpired`, `handle_run_lost` sets `LivesDepleted`. Screen match arm maps each to correct text. Confirmed clean.

## Overclock Engine Bugs (2026-03-20, fix/stress-count-and-dead-code)

- **ActiveChains (was ActiveOverclocks) never cleared between runs**: `chips/effects/overclock.rs` — `handle_overclock` pushes to `ActiveChains.0` on chip select. `reset_run_state` (OnExit MainMenu) clears ChipInventory but not ActiveChains. Overclock chains from run N persist and fire in run N+1. Fix: clear `ActiveChains.0` in a system on `OnEnter(GameState::Playing)` or `OnExit(GameState::MainMenu)`.

- **Retroactive bump path silences None last_hit_bolt**: `breaker/systems/bump.rs:115` — `update_bump` uses `bump.last_hit_bolt.unwrap_or(Entity::PLACEHOLDER)`. The `None` case is not reachable through current code, but the invariant `post_hit_timer > 0 ↔ last_hit_bolt is Some` is not structural. Should use `expect()` or restructure the timer/entity as a single `Option<(f32, Entity)>`. Medium confidence — not currently reachable, but silently wrong if it becomes reachable.

## Recurring Bug Category (new)
- **Resource Vec not cleared on run reset**: pattern seen in ActiveChains. When a Vec resource is populated during gameplay, ensure `reset_run_state` or an OnEnter(Playing) system clears it. Check all Vec resources when adding new ones.

## Overclock Trigger Chain Bugs (2026-03-20, feature/overclock-trigger-chain)

- **Global-triggered Shockwave no-ops silently**: FIXED (2026-03-20, feature/overclock-trigger-chain). `EffectFired.bolt` (was `OverclockEffectFired.bolt`) changed from `Entity` (using PLACEHOLDER for global triggers) to `Option<Entity>` (using `None`). `handle_shockwave` explicitly `let Some(bolt_entity) = trigger.event().bolt else { return; }` — the no-op for global triggers is now intentional and documented. Test `shockwave_no_op_with_none_bolt` covers this. Design decision: global-trigger shockwaves require a bolt entity for position; no position → no area damage. See design-principles.md for the design note.

- **bridge_cell_destroyed fires once for N destroyed cells**: Uses `reader.read().count() == 0` to detect any messages then evaluates chains once regardless of count. If N cells are destroyed in one frame, `OnCellDestroyed(Shockwave)` fires exactly once. The comment says "once per message" but implementation is "once if any messages". If design intent is one-shockwave-per-destroyed-cell, this is a bug.
  - Confidence: medium (design intent unclear)

- **inter-frame cascade for OnCellDestroyed(Shockwave)**: Shockwave writes DamageCell messages → handle_cell_hit processes them and writes CellDestroyed → on the next frame, bridge_cell_destroyed sees those CellDestroyed messages and fires the shockwave again → shockwave writes more DamageCell → etc. Bounded (terminates when no in-range cells remain), but produces multiple shockwave rounds per original trigger. May be surprising.
  - Confidence: medium (may be intentional cascade mechanic)
  - Confidence: medium (may be intentional)

## refactor/unify-behaviors Confirmed Bugs (2026-03-21) — RESOLVED BY UNIFICATION

NOTE: The following bugs were opened when new TriggerChain variants were added as type-only in refactor/unify-behaviors Step 1. The full unification (bolt/behaviors/ merged into behaviors/) resolved the structural issues by wiring evaluate.rs (now TriggerKind in behaviors/evaluate.rs with EarlyBump, LateBump, BumpWhiff variants) and adding handlers in behaviors/effects/. Verify against current code before re-flagging.

- **OnEarlyBump, OnLateBump, OnBumpWhiff trigger variants**: `TriggerKind` (was `OverclockTriggerKind`) in `behaviors/evaluate.rs` now includes EarlyBump, LateBump, BumpWhiff variants. The evaluate() function's or-pattern now covers all 10 trigger kinds. VERIFY: check behaviors/evaluate.rs — if TriggerKind has EarlyBump/LateBump/BumpWhiff variants and evaluate() handles them, this is fixed.

- **No bridge for BumpWhiffed**: `bridge_bump` (behaviors/bridges.rs) handles BumpGrade mapping to TriggerKind. VERIFY: check if bridge_bump_whiff system exists in behaviors/bridges.rs.

- **LoseLife, TimePenalty, SpawnBolt, SpeedBoost (was BoltSpeedBoost) leaves — RESOLVED**: The unification confirmed `handle_life_lost`, `handle_time_penalty`, `handle_spawn_bolt`, and `handle_speed_boost` observers all exist in `behaviors/effects/` and observe `EffectFired`. All four leaf types are fully wired. BoltSpeedBoost renamed to SpeedBoost { target: SpeedBoostTarget, multiplier: f32 } in refactor/unify-behaviors.

- **Recurring pattern**: Adding TriggerChain variants requires THREE coordinated updates: (1) enum + depth()/is_leaf(), (2) TriggerKind + evaluate(), (3) bridge system + effect handler. This three-part requirement is now well-documented in the codebase.

## Overclock Engine Bugs (2026-03-20, fix/stress-count-and-dead-code)

- **ActiveOverclocks (now ActiveChains) never cleared between runs**: `chips/effects/overclock.rs:15` — `handle_overclock` pushes to `ActiveChains.0` on chip select. `reset_run_state` (OnExit MainMenu) clears ChipInventory but not ActiveChains. Overclock chains from run N persist and fire in run N+1. Fix: clear `ActiveChains.0` in a system on `OnEnter(GameState::Playing)` or `OnExit(GameState::MainMenu)`. NOTE: type renamed from ActiveOverclocks to ActiveChains in refactor/unify-behaviors.

## SpeedBoost Generalization Bugs (2026-03-21, refactor/unify-behaviors or follow-on branch)

- **init_archetype wipes overclock chains on every node entry**: `behaviors/init.rs:108` — `*active = ActiveChains(chains)` unconditionally replaces the resource on every `OnEnter(GameState::Playing)`. `handle_overclock` pushes overclock chip chains to `ActiveChains` during ChipSelect state. State flow: Playing→TransitionOut→ChipSelect (handle_overclock adds chain)→TransitionIn→Playing (init_archetype resets). Overclock chains selected between nodes are silently discarded on the next node entry. Fix: `init_archetype` should EXTEND `active.0` with the archetype chains rather than replacing the entire resource. Or separate "archetype chains" from "overclock chains" so only archetype chains are reset. Confidence: HIGH.

## BumpForceBoost Dead Code (confirmed 2026-03-21)
- `BumpForceBoost` component is stamped by `handle_bump_force_boost` (chips/effects/bump_force_boost.rs) but no system reads it to affect bump behavior. The chip effect observer correctly stacks the value on the breaker, but the value is never consumed. This is a pre-existing gap, not introduced by the SpeedBoost refactor. The PR description notes it as "intentional — left for future use."

## Wave 3 Chip Select / Transition Bugs (2026-03-22) — OPEN

- **spawn_chip_select overwrites ChipOffers with unweighted registry-order list**: `screen/chip_select/systems/spawn_chip_select.rs:23,50` — Despite the `(generate_chip_offerings, ApplyDeferred, spawn_chip_select).chain()` wiring in ChipSelectPlugin, `spawn_chip_select` reads `registry.ordered_values().take(MAX_CARDS)` directly and then calls `commands.insert_resource(ChipOffers(offers))`, overwriting the weighted-random `ChipOffers` inserted by `generate_chip_offerings`. The entire offering algorithm is bypassed. Players always see the same first-N chips from registry insertion order.

- **advance_node on OnEnter(TransitionIn) immediately short-circuits the TransitionIn animation**: `run/plugin.rs:44` — `advance_node` is registered on `OnEnter(GameState::TransitionIn)` and unconditionally sets `NextState(Playing)`. Since OnEnter schedules run before Update, this means `TransitionIn` lasts exactly one frame. The `animate_transition` system (Update) never gets a chance to run its timer to completion and transition to Playing — `advance_node` always wins. Fix: either move `advance_node` to `OnEnter(GameState::Playing)` (so it fires when Playing is entered), or remove the `NextState(Playing)` call from `advance_node` and let `animate_transition` drive the `TransitionIn → Playing` transition.
