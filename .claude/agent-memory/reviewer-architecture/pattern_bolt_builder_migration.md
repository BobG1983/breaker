---
name: Bolt builder typestate migration
description: Bolt entities now use typestate builder (Bolt::builder()); init_bolt_params and prepare_bolt_velocity eliminated; velocity clamping inline at each mutation site
type: project
---

Bolt entity construction migrated to typestate builder pattern (`Bolt::builder()` in `bolt/builder.rs`).

**Key changes:**
- `init_bolt_params` system ELIMINATED — builder inserts all components at spawn time
- `prepare_bolt_velocity` system ELIMINATED — velocity clamping now inline via `apply_velocity_formula()` at each mutation site (collision, lost, launch, reset)
- `BoltSystems::InitParams` and `BoltSystems::PrepareVelocity` set variants REMOVED
- Effect domain fire functions (spawn_bolts, spawn_phantom, chain_bolt, tether_beam, mirror_protocol) use `Bolt::builder()` for entity construction
- `spawn_bolt` is now an exclusive system (`world: &mut World`) — uses `resource_mut::<Messages<T>>()` instead of `MessageWriter`
- Builder visibility: `pub(crate) mod builder` — accessible within `breaker-game` crate only

**Why:** Eliminates frame-delay where bolt entities existed without physics components. Centralizes component insertion for future BoltConfig elimination and BoltDefinition registry migration.

**How to apply:** When reviewing bolt-related changes, verify builder usage for new bolt-spawning code. When reviewing ordering constraints, note that velocity clamping is inline — there is no single PrepareVelocity ordering anchor anymore.

**Open items:**
- `bolt/builder.rs` at ~2700 lines needs file splitting (confirmed by vetted_dependencies.md 2026-03-31; listed as HIGH in reviewer-file-length/phase4_findings.md)
- Architecture docs (ordering.md, plugins.md, bolt-definitions.md) partially updated per guard-docs/known-state.md 2026-03-31 entry; `ordering.md` and `plugins.md` still may reference deleted systems — verify
- Gravity well / attraction effects write Velocity2D without same-frame clamping (one-frame delay before collision systems clamp)
- `BoltSystems::PrepareVelocity` ordering anchor used by gravity_well/attraction register() is stale; verify current ordering in plugin
