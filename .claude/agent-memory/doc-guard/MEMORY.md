# doc-guard Memory

## Intentionally Forward-Looking Docs (Do Not Flag as Drift)
- `docs/architecture/state.md` — lists `RunSetup`, `UpgradeSelect`, `MetaProgression` states that exist in code but screens are not yet built. These are forward-looking and correct by design.
- `docs/plan/phase-2/phase-2c-archetype-system.md` — checklist still has `[ ]` items but some are partially implemented (ArchetypeDefinition, Aegis, behaviors). File is in `plan/phase-2/` (not done/) so it is intentionally in-progress.

## Known Gaps (Accepted for Now)
- `docs/architecture/plugins.md` folder listing marks `run/` and `ui/` as `(stub — Phase 2+)` — these domains are now active. Should be cleaned up.
- `docs/architecture/plugins.md` marks `upgrades/` as `(stub — Phase 3+)` — Phase 3 is now Dev Infrastructure; upgrades are Phase 8. Phase number is wrong.

## Phase Completion Log
- 2026-03-16: Phase 2a (Level Loading) confirmed complete — file at `docs/plan/done/phase-2/phase-2a-level-loading.md`
- 2026-03-16: Phase 2b (Run Structure & Node Timer) confirmed complete — file at `docs/plan/done/phase-2/phase-2b-run-and-timer.md`
- PLAN.md links for 2a/2b are broken (still point to old `plan/phase-2/` path) — fix pending

## Terminology Decisions
- `BumpPerformed` consumers in breaker domain: system names are `spawn_bump_grade_text` and `perfect_bump_dash_cancel` (not `bump_feedback` which is only a module name)
- Bolt lost feedback: system is `spawn_bolt_lost_text` in module `bolt/systems/bolt_lost_feedback.rs`

## Recurring Drift Patterns
- Stub labels in `plugins.md` folder listing go stale as phases complete — check on each doc-guard pass
- PLAN.md links break when subphase files are moved to `done/` folder — check all paths are valid
