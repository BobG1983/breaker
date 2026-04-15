---
name: Effect System Refactor Coverage Map
description: Coverage state for feature/effect-system-refactor branch — nested trigger arming, Shape D On(Participant), SpawnStampRegistry watchers, unified death pipeline, lock/invulnerable, entropy refactor
type: project
---

## Branch: feature/effect-system-refactor

Key commits:
- `04574d8b` feat: nested trigger arming, Shape D participant-targeted reversal, spawn-stamp watchers, entropy refactor

## Mechanic Coverage by Change Area

| Mechanic | Unit Tests | Scenario Coverage | Status |
|----------|-----------|-------------------|--------|
| Nested When(X, When/Once/Until) arming | when.rs — 14 behaviors | surge_overclock, once_damage_single_fire, overclock_until_speed, damage_boost_until_reversal, whiplash_whiff_chaos, supernova_chain_stress, tether_chain_bolt_stress | ADEQUATE — all three gate-inner patterns (When, Once, Until) exercised |
| When(X, When) → staged (not recursed) | when.rs unit tests | surge_overclock: When(PerfectBumped, When(Impacted(Cell), ...)); supernova_chain_stress: 3-deep | ADEQUATE |
| When(X, Once) → staged | when.rs unit tests | once_damage_single_fire: When(Impacted(Cell), Once([...])) | ADEQUATE |
| When(X, Until) → staged | when.rs unit tests | overclock_until_speed: When(PerfectBumped, Until(TimeExpires(1.0), ...)); damage_boost_until_reversal | ADEQUATE |
| Shape D: On(Participant, Terminal) | on.rs unit tests | NONE in any scenario | MISSING |
| Shape D: ArmedFiredParticipants drain | armed_fired_participants.rs unit tests | NONE — no scenario uses armed source strings | MISSING |
| SpawnStampRegistry watcher path | stamp_spawned_bolts.rs unit tests | NONE — all scenarios use Stamp() initial_effects, not Spawn() chip root nodes | MISSING — STRUCTURAL RUNNER GAP |
| Entropy refactor | entropy_engine_stress | entropy_engine_stress: SpawnBolts + Shockwave pool | ADEQUATE |
| Cell death unified pipeline | died_trigger, cell_death_speed_burst | Indirect: Death trigger fires = cells died and pipeline ran | WEAK — no invariant validates Destroyed<Cell> position or Invulnerable skip |
| Bolt lifespan death | spawn_bolts_stress, phantom_bolt_stress, entropy_engine_stress | Lifespan cleanup exercised under load | ADEQUATE for NoEntityLeaks; no dedicated Destroyed<Bolt> message validation |
| Lock/Invulnerable coupling | locked/components.rs unit tests (hooks) | NONE — no layout with LockCell cells in any scenario | MISSING |
| Breaker death (handle_breaker_death) | handle_breaker_death.rs (9 unit tests) | aegis_lives_exhaustion: run ends; dead_mans_hand_bolt_loss: repeated bolt loss | WEAK — "run ends" only; Destroyed<Breaker> effect bridge never validated |
| Wall death (handle_kill<Wall>) | death_pipeline unit tests | NONE — also no KillYourself<Wall> producer in game | EXPECTED GAP (dead code path) |

## Key Findings

### Structural Runner Gap: SpawnStampRegistry

The scenario `initial_effects` field only supports `Stamp(EntityKind, Tree)` and `Spawn(EntityKind, Tree)`.
The SpawnStampRegistry is populated exclusively when a chip dispatched at runtime has a `Spawn(EntityKind, Tree)` root node — NOT by `initial_effects: [Spawn(...)]` in the scenario RON.

The watcher systems (`stamp_spawned_bolts`, `stamp_spawned_cells`, `stamp_spawned_walls`, `stamp_spawned_breakers`) all use `Added<T>` queries and read from `SpawnStampRegistry`. There is no way to populate `SpawnStampRegistry` via scenario RON syntax. This path is only reachable by selecting a chip whose implementation uses `Spawn` tree nodes — not directly injectable.

**New invariant needed:** None will detect a missing watcher stamp — the bug would be silent (spawned bolts simply don't have effects).

### Shape D / On(Participant) Gap

The `Tree::On(ParticipantTarget, Terminal)` variant is entirely new on this branch. No scenario RON uses it. The `On(...)` variant is not exposed in scenario RON syntax.

**Question for writer-scenarios:** Can `On(Participant, Terminal)` be expressed in scenario `initial_effects`? If yes, add a scenario. If no, this is a runner capability gap.

### Lock/Invulnerable Gap

No layout in any scenario contains `LockCell`/`Locked` cells. The `Invulnerable` marker + `apply_damage` skipping is unit-tested in death_pipeline but never exercised under chaos. Risk: damage routing to Invulnerable entities would silently absorb without error under any existing invariant.

## Invariant Gaps Specific to This Branch

- No invariant validates `Destroyed<Cell>` message carries correct position — only NoEntityLeaks/NoNaN guard the pipeline
- No invariant validates that `Invulnerable` cells survive regardless of damage input
- No invariant validates that `ArmedFiredParticipants` drains completely after disarm reversal
- No invariant validates that SpawnStampRegistry-registered effects appear on spawned entities

## Previously-Open Gaps Now Closed (vs. prior coverage_effect_system.md)

- `Died` trigger: now covered by `died_trigger.scenario.ron`
- `NoBump` trigger: now covered by `no_bump_trigger.scenario.ron`
- `Impacted(Breaker)`: now covered by `impacted_breaker_trigger.scenario.ron`
- `Impact(Bolt)`: now covered by `impact_bolt_trigger.scenario.ron`

**How to apply:** Flag Shape D `On(Participant)` and Lock/Invulnerable as HIGH gaps on this branch. SpawnStampRegistry is a structural runner gap — may require a new initial_effects syntax variant or chip injection support. Nested trigger arming is adequately covered — do not flag surge_overclock etc. as gaps.
