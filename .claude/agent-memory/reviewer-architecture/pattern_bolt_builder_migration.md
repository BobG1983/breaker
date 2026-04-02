---
name: Bolt builder typestate migration
description: Bolt entities now use typestate builder (Bolt::builder()) with 6 dimensions (P,S,A,M,R,V); init_bolt_params and prepare_bolt_velocity eliminated; velocity clamping inline at each mutation site
type: project
---

Bolt entity construction migrated to typestate builder pattern (`Bolt::builder()` in `bolt/builder/core.rs`).

**Key changes:**
- 6 typestate dimensions: P (Position), S (Speed), A (Angle), M (Motion: Serving/HasVelocity), R (Role: Primary/Extra), V (Visual: Rendered/Headless)
- `init_bolt_params` system ELIMINATED — builder inserts all components at spawn time
- `prepare_bolt_velocity` system ELIMINATED — velocity clamping now inline via `apply_velocity_formula()` at each mutation site
- Effect domain fire functions use `Bolt::builder()` for entity construction
- `spawn_bolt` is an exclusive system (`world: &mut World`)
- Builder visibility: `pub(crate) mod builder` — accessible within `breaker-game` crate only
- Visual dimension added (matching breaker's pattern): Rendered includes Mesh2d + MeshMaterial2d + GameDrawLayer::Bolt; Headless omits all three
- `BoltRadius` is now a type alias for `BaseRadius` from `shared/size.rs`
- `BaseRadius`, `MinRadius`, `MaxRadius` live in `shared/size.rs`
- `apply_velocity_formula()` in `bolt/queries.rs` (file-placement concern: it's a function, not a query type)

**Current state (as of 2026-04-02, verified in feature/breaker-builder-pattern):**
- Builder tests split into 8 test files in bolt/builder/tests/
- Builder is fully wired; all bolt spawning uses Bolt::builder()
- `BoltConfig` eliminated

**How to apply:** When reviewing bolt-related changes, verify builder usage for new bolt-spawning code. Velocity clamping is inline — no single PrepareVelocity ordering anchor.
