---
name: Terminology Decisions
description: Vocabulary decisions, preferred terms, and additions made during reviews
type: reference
---

## Terminology Decisions
- `BumpPerformed` consumers in breaker domain: system names are `spawn_bump_grade_text` and `perfect_bump_dash_cancel` (not `bump_feedback` which is only a module name)
- Bolt lost feedback: system is `spawn_bolt_lost_text` in module `bolt/systems/bolt_lost_feedback.rs`
- Archetypes (Aegis, Chrono, Prism) are purely data-driven via RON files — no `AegisPlugin`, `ChronoPlugin`, or `PrismPlugin` exist. Do not flag as a gap.
- `behaviors/` is a top-level domain (not nested under `breaker/`). Plugin is `BehaviorsPlugin`. System set is `BehaviorSystems::Bridge`.
- `ConsequenceFired(Consequence)` is a Bevy observer event (not a Message) — lives in `behaviors/definition.rs`.
- `handle_spawn_bolt` replaced `handle_spawn_bolt_requested`.
- `BreakerSystems::InitParams` tags `init_breaker_params` — alongside `BreakerSystems::Move`.
- `UiSystems::SpawnTimerHud` tags `spawn_timer_hud`.
- OnEnter(Playing) ordering chain documented in `docs/architecture/ordering.md`.
- `GameState::ChipSelect` (NOT `UpgradeSelect`). Message is `ChipSelected { name, kind }`.
- Inter-node flow: `Playing → ChipSelect → NodeTransition → Playing`.

## Terminology Additions (2026-03-17)
- `Scenario` / `ScenarioDefinition` — automated test run defined in `.scenario.ron`
- `Invariant` / `InvariantKind` — runtime assertion checked each frame during scenario
- `Chaos` / `Scripted` / `Hybrid` — input strategies in the scenario runner
- `Recording` — `debug/recording/` sub-domain, `RecordingConfig`, `--record` dev flag
- All added to `docs/TERMINOLOGY.md`
