# doc-guard Memory

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `MetaProgression` state that exists in code but screen is not yet built. Forward-looking and correct by design.
- `docs/plan/done/phase-2/phase-2e-chrono-and-prism.md` — checklist has 3 open items (Chrono/Prism bolt-loss visual indicators, all three playable). These are known incomplete items accepted as Phase 2 closed.

## Known Gaps (Accepted for Now)
- Phase 2e checklist: Bolt-loss visual indicators for Chrono and Prism not yet built — left unchecked in done file as honest record.

## Phase Completion Log
- 2026-03-16: Phase 2a (Level Loading) confirmed complete — file at `docs/plan/done/phase-2/phase-2a-level-loading.md`
- 2026-03-16: Phase 2b (Run Structure & Node Timer) confirmed complete — file at `docs/plan/done/phase-2/phase-2b-run-and-timer.md`
- 2026-03-16: Phase 2c (Archetype System & Aegis) confirmed complete — file moved to `docs/plan/done/phase-2/phase-2c-archetype-system.md`
- 2026-03-16: Phase 2d (Screens) confirmed complete — file moved to `docs/plan/done/phase-2/phase-2d-screens-and-ui.md`
- 2026-03-16: Phase 2e (Visual Polish & Additional Archetypes) core complete — file moved to `docs/plan/done/phase-2/phase-2e-chrono-and-prism.md`
- 2026-03-16: Full Phase 2 marked Done in PLAN.md; Phase 3 was Current
- 2026-03-17: Phase 3a/3b/3c/3d/3e all confirmed complete — all moved to `docs/plan/done/phase-3/`; Phase 4 is now Current

## Terminology Decisions
- `BumpPerformed` consumers in breaker domain: system names are `spawn_bump_grade_text` and `perfect_bump_dash_cancel` (not `bump_feedback` which is only a module name)
- Bolt lost feedback: system is `spawn_bolt_lost_text` in module `bolt/systems/bolt_lost_feedback.rs`
- Archetypes (Aegis, Chrono, Prism) are purely data-driven via RON files — no `AegisPlugin`, `ChronoPlugin`, or `PrismPlugin` exist. Do not flag this as a gap.
- `behaviors/` is a top-level domain (not nested under `breaker/`). Plugin is `BehaviorsPlugin`. System set is `BehaviorSystems::Bridge` (in `behaviors/sets.rs`).
- `ConsequenceFired(Consequence)` is a Bevy observer event (not a Message) — lives in `behaviors/definition.rs`. It replaces the old per-consequence events (`LoseLifeRequested`, `TimePenaltyRequested`, `SpawnBoltRequested`).
- `handle_spawn_bolt` (in `behaviors/consequences/spawn_bolt.rs`) replaced `handle_spawn_bolt_requested`.
- `BreakerSystems::InitParams` (in `breaker/sets.rs`) tags `init_breaker_params` — added alongside `BreakerSystems::Move`.
- `UiSystems::SpawnTimerHud` (in `ui/sets.rs`) tags `spawn_timer_hud` — new set, new file.
- OnEnter(Playing) ordering chain: `apply_archetype_config_overrides` → `BreakerSystems::InitParams` → `init_archetype` → `UiSystems::SpawnTimerHud` → `spawn_lives_display`. Documented in `docs/architecture/ordering.md` under "OnEnter(GameState::Playing)" subsection.
- `GameState::ChipSelect` (NOT `UpgradeSelect`) — the between-nodes chip selection state. Message is `ChipSelected { name, kind }` (NOT `UpgradeSelected`). All docs now use ChipSelect/ChipSelected.
- Inter-node flow: `Playing → ChipSelect → NodeTransition → Playing`. Documented in `docs/architecture/state.md` NodeTransition section.

## Terminology Additions (2026-03-17)
- `Scenario` / `ScenarioDefinition` — automated test run defined in `.scenario.ron`
- `Invariant` / `InvariantKind` — runtime assertion checked each frame during scenario
- `Chaos` / `Scripted` / `Hybrid` — input strategies in the scenario runner
- `Recording` — `debug/recording/` sub-domain, `RecordingConfig`, `--record` dev flag
- All added to `docs/TERMINOLOGY.md`

## Scenario Runner Architecture (do not re-flag)
- `breaker-scenario-runner/` is a workspace peer — documented in `plugins.md` workspace layout
- `ScenarioLayoutOverride` resource lives in `breaker-game/src/shared/` — allows scenario runner to bypass run setup
- `debug/recording/` sub-domain exists in `breaker-game/src/debug/` — captures live inputs to `.scripted.ron` files
- `validate_pass` logic in `runner.rs`: if `expected_violations: Some(...)` the scenario is a self-test — violations must match exactly
- Log capture filter covers both `breaker` and `breaker_scenario_runner` targets
- CI: `.github/workflows/ci.yml` has `test` (3 platforms) and `scenarios` (Linux headless) jobs

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete — check on each doc-guard pass
- PLAN.md links break when subphase files are moved to `done/` folder — check all paths are valid
- Phase plan files in `plan/phase-N/` should be updated with redirects when moved to `done/`

## Session History
See [ephemeral/](ephemeral/) — not committed.
