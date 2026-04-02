---
name: Breaker builder typestate migration
description: Breaker entities use typestate builder (Breaker::builder()) with 7 dimensions including Role; fully wired in production
type: project
---

Breaker entity construction migrated to typestate builder pattern (`Breaker::builder()` in `breaker/builder/core.rs`).

**Key facts:**
- 7 typestate dimensions: D (Dimensions), Mv (Movement), Da (Dashing), Sp (Spread), Bm (Bump), V (Visual), R (Role)
- Role dimension: `Primary` (CleanupOnRunEnd) vs `Extra` (CleanupOnNodeExit) -- matches bolt's Role pattern
- Visual dimension: `Rendered` (Mesh2d + MeshMaterial2d) vs `Headless` (omits them) -- matches bolt's Visual pattern
- Convenience method: `.definition(&BreakerDefinition)` transitions D+Mv+Da+Sp+Bm simultaneously
- Builder imports from effect domain: `EffectCommandsExt`, `RootEffect`, `LivesCount` -- all read/dispatch, not boundary violations
- Inner file uses `core.rs` (not `builder.rs`) to avoid `clippy::module_inception`
- 4 terminal `impl` blocks: Rendered+Primary, Rendered+Extra, Headless+Primary, Headless+Extra

**Current state (as of 2026-04-02, verified in feature/breaker-builder-pattern):**
- Builder is implemented and tested (11+ test files in builder/tests/)
- Old 4-system chain eliminated; `spawn_or_reuse_breaker` wires the builder
- `BreakerConfig`, `BreakerDefaults`, `BreakerStatOverrides` all eliminated
- `BreakerDefinition` is the single source (36 fields, all `#[serde(default)]` except `name`)
- `BreakerRegistry` implements `SeedableRegistry`
- Architecture docs fully updated (builders/breaker.md, builders/pattern.md, data.md)
- `LivesCount` lives in `effect/effects/life_lost.rs` -- breaker builder imports it cross-domain (read access, not a violation)

**Minor findings (non-blocking):**
- `BreakerDashData` and `BreakerResetData` in queries.rs are `pub` instead of `pub(crate)` -- no external consumers exist
- `GameDrawLayer::Breaker` is in build_core (all builds including headless) while bolt omits `GameDrawLayer::Bolt` from headless -- cosmetic inconsistency
- `sync_bolt_scale` in FixedUpdate vs `sync_breaker_scale` in Update -- schedule placement inconsistency for equivalent visual sync systems

**How to apply:** When reviewing breaker-related changes, verify builder usage for new breaker-spawning code. `.definition()` replaces `.config()`. 7 typestate dims: D, Mv, Da, Sp, Bm, V, R. `.primary()` and `.extra()` select the Role dimension.
