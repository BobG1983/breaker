---
name: Known State
description: Intentionally forward-looking docs, known gaps, scenario runner architecture, recurring drift patterns
type: reference
---

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `MetaProgression` state that exists in code but screen is not yet built
- `docs/plan/done/phase-2/phase-2e-chrono-and-prism.md` — checklist has 3 open items (Chrono/Prism bolt-loss visual indicators, all three playable). Known incomplete accepted as Phase 2 closed.

## Known Gaps (Accepted for Now)
- Phase 2e checklist: Bolt-loss visual indicators for Chrono and Prism not yet built — left unchecked

## Scenario Runner Architecture (do not re-flag)
- `breaker-runner-scenarios/` is a workspace peer — documented in `plugins.md` workspace layout
- `ScenarioLayoutOverride` resource lives in `breaker-game/src/shared/` — allows scenario runner to bypass run setup
- `debug/recording/` sub-domain exists in `breaker-game/src/debug/` — captures live inputs
- `validate_pass` logic: if `expected_violations: Some(...)` the scenario is a self-test — violations must match exactly
- Log capture filter covers both `breaker` and `breaker_scenario_runner` targets
- CI: `.github/workflows/ci.yml` has `test` (3 platforms) and `scenarios` (Linux headless) jobs

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete
- PLAN.md links break when subphase files are moved to `done/` folder
- Phase plan files in `plan/phase-N/` should be updated with redirects when moved to `done/`
