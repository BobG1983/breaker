---
name: Current lint state
description: Full workspace clippy and test result as of 2026-03-30 on develop — all crates PASS, all tests PASS
type: project
---

Last run: 2026-03-30 (develop branch, post-merge of feature/source-chip-shield-absorption)

## Format: PASS (no files changed)

## game crate (breaker-game, dclippy): PASS (0 errors, 0 warnings)
## rantzsoft_spatial2d (spatial2dclippy): PASS (0 errors, 0 warnings)
## rantzsoft_physics2d (physics2dclippy): PASS (0 errors, 0 warnings)
## rantzsoft_defaults (defaultsclippy): PASS (0 errors, 0 warnings)
## breaker-scenario-runner (dsclippy): PASS (0 errors, 0 warnings)

## Tests
- dtest: PASS (1990 tests)
- spatial2dtest: PASS (97 tests)
- physics2dtest: PASS (117 tests)
- defaultstest: PASS (52 tests)
- dstest: PASS (424 + 5 tests)

Previous build error E0425 in gravity_well.rs is resolved.
Game crate grew by 1 test since last run (1989 → 1990).
