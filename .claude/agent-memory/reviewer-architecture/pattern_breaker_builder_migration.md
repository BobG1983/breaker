---
name: Breaker builder typestate migration
description: Breaker entities use typestate builder (Breaker::builder()) with 7 dimensions including Role; old spawn chain still wired in production
type: project
---

Breaker entity construction migrated to typestate builder pattern (`Breaker::builder()` in `breaker/builder/core.rs`).

**Key facts:**
- 7 typestate dimensions: D (Dimensions), Mv (Movement), Da (Dashing), Sp (Spread), Bm (Bump), V (Visual), R (Role)
- Role dimension: `Primary` (CleanupOnRunEnd) vs `Extra` (CleanupOnNodeExit) -- matches bolt's Role pattern
- Convenience method: `.definition(&BreakerDefinition)` transitions D+Mv+Da+Sp+Bm simultaneously
- Builder imports from effect domain: `EffectCommandsExt`, `RootEffect`, `LivesCount` -- all read/dispatch, not boundary violations
- Inner file uses `core.rs` (not `builder.rs`) to avoid `clippy::module_inception`

**Current state (as of 2026-04-01):**
- Builder is implemented and tested (10 test files, ~60 behaviors)
- Old spawn chain (`spawn_breaker` -> `init_breaker_params` -> `init_breaker` -> `dispatch_breaker_effects`) STILL WIRED in `plugin.rs`
- Wiring swap has not happened yet -- builder coexists with old systems
- `BreakerStatOverrides` fully removed from code; negative RON test verifies old format fails

**Visibility fix required:**
- `breaker/mod.rs` line 3: must be `pub(crate) mod builder;` not `pub mod builder;` (convention from pattern.md)
- `breaker/mod.rs` line 15: must be `pub(crate) use` not `pub use` to match

**Architecture doc drift:**
- `builders/breaker.md` lists 6 dims (missing R), refers to `.config(&BreakerConfig)` not `.definition(&BreakerDefinition)`
- `builders/pattern.md` shows 6-field struct (missing `role: R`)
- `data.md` references old component names (`BreakerWidth` -> `BaseWidth`) and `.bdef.ron` -> `.breaker.ron`
- `bolt-definitions.md` still references `BreakerStatOverrides`

**How to apply:** When reviewing breaker-related changes, verify builder usage for new breaker-spawning code. When the wiring swap happens, verify `spawn_breaker`/`init_breaker_params`/`init_breaker` are removed and ordering constraints updated.
