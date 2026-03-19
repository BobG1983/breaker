# Performance Agent Memory

## Calibration
- Entity scale: fixed grid (cells per layout), 1 bolt, 1 breaker. Max ~50-200 entities total.
- Scenario count: 29 RON files (as of 2026-03-19); 2 are stress scenarios.
- Phase 2 is active. Phase 3 (upgrades/full content) will increase entity counts.

## Session History
See [ephemeral/](ephemeral/) — not committed.

Ephemeral reviews on file:
- `ephemeral/review-2026-03-19-local-vec.md` — Local<Vec> hot-path allocation refactor (Clean)
- `ephemeral/review-2026-03-19-clamp-applydeferred.md` — PlayfieldConfig Res + ApplyDeferred in OnEnter (Clean)
- `ephemeral/review-2026-03-19-stress-runner.md` — Stress runner subprocess management (1 Minor, otherwise Clean)
- `ephemeral/review-2026-03-19-spawn-batched-refactor.md` — spawn_batched extraction in execution.rs (Clean; prior Minor resolved)
- `ephemeral/review-2026-03-19-clamp-bolt-to-playfield.md` — clamp_bolt_to_playfield safety system (Clean)
- `ephemeral/review-2026-03-19-full-tree.md` — Full-tree validation pass (Clean; 2 Minors noted)
- `ephemeral/review-2026-03-19-chips-runseed.md` — chips/apply_chip_effect + run_setup seed systems (1 Moderate, 2 Minor)

## Known Intentional Patterns (do not flag)
- `snapshot_eval_data` clones `ViolationLog`, `CapturedLogs`, `ScenarioStats`, `ScenarioDefinition` every frame in the scenario runner — intentional, required to survive `App::run()` teardown.
- `Local<Vec>` reuse in `bolt_lost` and `handle_cell_hit` hot paths — established pattern after deferred-review fix (2026-03-19).
- `BumpVisual` add/remove per bump on single Breaker entity — 1 entity, low frequency, negligible archetype cost.
- `Option<&BumpPerfectMultiplier>` / `Option<&BumpWeakMultiplier>` in BumpTimingQuery / BumpGradingQuery — set at archetype init time, not per-frame churn. Intentional archetype-optional design.
- All FixedUpdate physics/breaker/cell systems have `run_if(in_state(PlayingState::Active))` guards — confirmed correct.
- Hot-reload propagation uses `resource_changed::<T>` guards — does not run every frame.
- Debug overlay systems use early-return on `overlays.is_active(...)` — cheap when disabled, no query work skipped needed.
- Optional component queries on Bolt (5 optionals) and Breaker (4 optionals) in `apply_chip_effect` — 1 entity each, added gradually during ChipSelect, not per-frame churn. Acceptable at current scale.
- `update_seed_display` allocates per frame (format! + clone) — guarded by `run_if(in_state(GameState::RunSetup))` and processes 1 entity; acceptable for UI screen.
