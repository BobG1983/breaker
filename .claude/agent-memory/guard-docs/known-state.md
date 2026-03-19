---
name: Known State
description: Intentionally forward-looking docs, known gaps, scenario runner architecture, drift patterns
type: reference
---

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `MetaProgression` state that exists in code but screen is not yet built
- `docs/plan/done/phase-2/phase-2e-chrono-and-prism.md` — checklist has 3 open items (Chrono/Prism bolt-loss visual indicators, all three playable). Known incomplete accepted as Phase 2 closed.

## Known Gaps (Accepted for Now)
- Phase 2e checklist: Bolt-loss visual indicators for Chrono and Prism not yet built — left unchecked

## Scenario Runner Architecture (do not re-flag)
- `breaker-scenario-runner/` is a workspace peer — documented in `plugins.md` workspace layout
- `ScenarioLayoutOverride` resource lives in `breaker-game/src/run/node/resources.rs` (not shared/) — allows scenario runner to bypass run setup
- `debug/recording/` sub-domain exists in `breaker-game/src/debug/` — captures live inputs
- `validate_pass` logic: if `expected_violations: Some(...)` the scenario is a self-test — violations must match exactly
- Log capture filter covers both `breaker` and `breaker_scenario_runner` targets
- CI: `.github/workflows/ci.yml` has `test` (3 platforms) and `scenarios` (Linux headless) jobs

## Spawn Coordination Architecture (do not re-flag)
- `SpawnNodeComplete` is a real active message sent by `check_spawn_complete` in `run/node/` — consumed by scenario runner for baseline entity count sampling
- Spawn signals: `BreakerSpawned` (breaker), `BoltSpawned` (bolt), `CellsSpawned` (run/node), `WallsSpawned` (wall) — all consumed by `check_spawn_complete`
- `check_spawn_complete` uses a `Local<SpawnChecklist>` bitfield — resets after firing to allow multi-node runs
- All 5 of these messages are now documented in `docs/architecture/messages.md`

## NodeSystems Set (do not re-flag)
- `NodeSystems` enum lives in `run/node/sets.rs` with variants: Spawn, TrackCompletion, TickTimer, ApplyTimePenalty, InitTimer
- Used cross-domain: `run/plugin.rs` orders `handle_node_cleared` and `handle_timer_expired` against it
- Now documented in `docs/architecture/ordering.md` and `docs/architecture/plugins.md`

## BreakerSystems::Reset (do not re-flag)
- `BreakerSystems::Reset` tags `reset_breaker` in `breaker/plugin.rs` OnEnter(Playing)
- Intra-domain only — no cross-domain consumers currently
- Added to ordering.md defined sets table with note "intra-domain only"

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete
- New system sets added to code without corresponding update to ordering.md defined sets table
- Spawn-coordination messages easily missed since they're internal infrastructure, not gameplay messages
- Intra-domain ordering chains in ordering.md can drift when constraints are restructured
- `PLAN.md` links break when subphase files are moved to `done/` folder
