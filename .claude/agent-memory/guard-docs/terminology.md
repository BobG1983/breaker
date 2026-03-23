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
- `ConsequenceFired(Consequence)` DELETED in refactor/unify-behaviors (2026-03-21). Replaced by `EffectFired { effect: TriggerChain, bolt: Option<Entity> }` in `behaviors/events.rs`. Do not flag `ConsequenceFired` absence as drift.
- `handle_spawn_bolt` replaced `handle_spawn_bolt_requested`.
- `BreakerSystems::InitParams` tags `init_breaker_params` — alongside `BreakerSystems::Move`.
- `UiSystems::SpawnTimerHud` tags `spawn_timer_hud`.
- OnEnter(Playing) ordering chain documented in `docs/architecture/ordering.md`.
- `GameState::ChipSelect` (NOT `UpgradeSelect`). Message is `ChipSelected { name: String }` — name only, no kind field.
- Inter-node flow: `Playing → TransitionOut → ChipSelect → TransitionIn → Playing` (NodeTransition state removed in Wave 3; replaced by TransitionOut and TransitionIn states with animation).

## Terminology Additions (2026-03-17)
- `Scenario` / `ScenarioDefinition` — automated test run defined in `.scenario.ron`
- `Invariant` / `InvariantKind` — runtime assertion checked each frame during scenario
- `Chaos` / `Scripted` / `Hybrid` — input strategies in the scenario runner
- `Recording` — `debug/recording/` sub-domain, `RecordingConfig`, `--record` dev flag
- All added to `docs/design/terminology.md`

## Terminology Additions (2026-03-21)
- `FrameMutation` — scripted mutation applied at a specific frame during a scenario; used in `frame_mutations` field of `ScenarioDefinition`
- `MutationKind` — enum of mutation operations: `SetBreakerState`, `SetTimerRemaining`, `SpawnExtraEntities`, `MoveBolt`, `TogglePause`
- Added to `docs/design/terminology.md`

## Terminology Additions (2026-03-23, Memorable Moments)
- `HighlightKind` entry expanded from 6 to 15 variants — do not flag 6-variant list as drift
- `HighlightDefaults` — new glossary entry for the RON-loaded asset and its `GameConfig` derive
- `HighlightTriggered` — new glossary entry for the juice/VFX message
- `RunHighlight.value` semantics clarified per-kind (distance for CloseSave, etc.)
- All added to `docs/design/terminology.md`
