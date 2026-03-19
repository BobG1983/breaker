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

## Known Intentional Patterns (do not flag)
- `snapshot_eval_data` clones `ViolationLog`, `CapturedLogs`, `ScenarioStats`, `ScenarioDefinition` every frame in the scenario runner — intentional, required to survive `App::run()` teardown.
- `Local<Vec>` reuse in `bolt_lost` and `handle_cell_hit` hot paths — established pattern after deferred-review fix (2026-03-19).
