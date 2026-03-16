# doc-guard Memory

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `RunSetup`, `UpgradeSelect`, `MetaProgression` states that exist in code but screens are not yet built. These are forward-looking and correct by design.
- `docs/plan/phase-2/phase-2c-archetype-system.md` — checklist still has `[ ]` items but some are partially implemented (ArchetypeDefinition, Aegis, behaviors). File is in `plan/phase-2/` (not done/) so it is intentionally in-progress.

## Known Gaps (Accepted for Now)
- Phase 2e checklist: Bolt-loss visual indicators for Chrono and Prism not yet confirmed built — left unchecked in plan file pending code confirmation.

## Phase Completion Log
- 2026-03-16: Phase 2a (Level Loading) confirmed complete — file at `docs/plan/done/phase-2/phase-2a-level-loading.md`
- 2026-03-16: Phase 2b (Run Structure & Node Timer) confirmed complete — file at `docs/plan/done/phase-2/phase-2b-run-and-timer.md`
- 2026-03-16: Phase 2e (Visual Polish & Additional Archetypes) core implementation confirmed complete — interpolate domain, Chrono/Prism archetypes, RON files. Visual indicators and playability items still open.
- PLAN.md: 2e moved to Done section; 2a/2b links resolve correctly to `plan/done/phase-2/`

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

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete — check on each doc-guard pass
- PLAN.md links break when subphase files are moved to `done/` folder — check all paths are valid
