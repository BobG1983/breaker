# Protocol: Conductor

## Category
`custom-system`

## Game Design
Perfect-bump an extra bolt to inherit the primary bolt's chip effects. Dropping any bolt still costs a life.

- With multiple bolts, only the "primary" bolt carries chip effects (`BoundEffects` / `StagedEffects`).
- Perfect Bump on an `ExtraBolt`: roles swap — that bolt becomes primary; the old primary becomes extra. `BoundEffects` + `StagedEffects` swap with the role change.
- No bolt-lost suppression — losing any bolt costs a life as normal.
- Pointless with 1 bolt — meaningful only in multi-bolt play.

## Config Resource
```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConductorConfig;
```

Unit struct — presence marker only. Inserted on `ProtocolTuning::Conductor` activation.

## Components
None owned by Conductor. Uses existing `PrimaryBolt` / `ExtraBolt` markers + `BoundEffects` / `StagedEffects` on bolt entities.

## Messages
**Reads**: `BumpPerformed { grade, bolt }` (breaker domain).
**Sends**: `SwapBoltRoles { old_primary: Entity, new_primary: Entity }` (new bolt-domain message introduced in TODO #9). Bolt domain's consumer performs the marker + effect swap in one transactional call — Conductor does NOT write `PrimaryBolt` / `ExtraBolt` markers directly from the protocol domain.

## Systems

### `conductor_request_swap_on_perfect_bump`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::Conductor)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. On `BumpGrade::Perfect` where the bumped bolt carries `ExtraBolt`: queries the current `PrimaryBolt` entity, emits `SwapBoltRoles { old_primary, new_primary: bumped_bolt }`. Drains all other messages on the tick (`swap_done` guard) to prevent multiple swaps per tick.
- **Ordering**: `.after(BreakerSystems::GradeBump)`. The bolt-domain consumer for `SwapBoltRoles` must run `.before(EffectV3Systems::Bridge)` so downstream bridges see the post-swap state.

Bolt-domain consumer (owned by bolt domain, not Conductor):

### `apply_swap_bolt_roles` (bolt domain)
- Removes `PrimaryBolt` from `old_primary`; inserts `ExtraBolt`.
- Removes `ExtraBolt` from `new_primary`; inserts `PrimaryBolt`.
- Swaps `BoundEffects` and `StagedEffects` between the two entities.

No PunchScale VFX insertion anywhere — the role-swap dispatch is data-only. VFX is deferred to Phase 5 graphics.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Conductor does not participate in any `DeathPipelineSystems` set.
- **Trigger**: `BumpPerformed` with Perfect grade on an `ExtraBolt`.
- **Writes**: `SwapBoltRoles` message (consumed in bolt domain).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed`.
- **bolt**: Owns `SwapBoltRoles` message + consumer. Writes `PrimaryBolt` / `ExtraBolt` markers.
- **effect_v3**: Consumer swaps `BoundEffects` / `StagedEffects`. The bridge sees post-swap state.

## Expected Behaviors (for test specs)

1. **Perfect bump on extra bolt swaps primary** — Bolt A is `PrimaryBolt` + `BoundEffects(X)`, Bolt B is `ExtraBolt` + no effects, Perfect bump on B: after swap, B is `PrimaryBolt` + `BoundEffects(X)`, A is `ExtraBolt` + no effects.
2. **Non-perfect bump does not swap** — Early bump on B: no `SwapBoltRoles` emitted; markers unchanged.
3. **Perfect bump on non-ExtraBolt is a no-op** — Perfect bump on A (already primary): no `SwapBoltRoles` emitted.
4. **At most one swap per tick** — two Perfect `BumpPerformed` for B in same tick: exactly one `SwapBoltRoles` emitted; second drained.
5. **Both `BoundEffects` and `StagedEffects` swap** — A has both, B has neither: after swap, B has both, A has neither.
6. **No PunchScale VFX insertion** — confirming no visual-effect component is inserted on the newly promoted bolt (VFX is Phase 5).
7. **Absent config drains reader** — `ConductorConfig` missing: the system's `run_if` gate prevents it from running at all.

## Edge Cases
- Single bolt: no `ExtraBolt` entities, so every bump is skipped. Protocol has no effect (by design — "pointless with 1 bolt").
- Fission interaction: new bolts from Fission spawn with `ExtraBolt` + no effects. Player must Perfect Bump to promote.
- Effect-in-flight: already-dispatched effects from a bolt before the swap are not recalled. Swap affects components at swap time.
