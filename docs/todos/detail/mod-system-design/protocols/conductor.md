# Protocol: Conductor

## Category
`custom-system`

## Game Design
Perfect-bump an extra bolt to inherit the primary bolt's chip effects. Dropping any bolt still costs a life as normal.

- With multiple bolts, only the "primary" bolt (`PrimaryBolt` marker) carries chip effects (`BoundEffects` / `StagedEffects` components).
- Perfect Bump an `ExtraBolt`: that bolt becomes the new primary; the old primary becomes an extra bolt. Their `BoundEffects` / `StagedEffects` components are swapped in the same transaction.
- No bolt-lost suppression — losing any bolt (primary or extra) costs a life as before.
- Pointless with 1 bolt — changes how multi-bolt plays.

## Config Resource
```rust
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConductorConfig;
```

Unit struct — used only as a presence marker for the
`Option<Res<ConductorConfig>>` harness-safety gate in
`conductor_swap_on_perfect_bump`. Inserted by `activate` on the
`ProtocolTuning::Conductor` unit variant.

## Components
Uses existing bolt-domain components — no Conductor-owned components:

- `PrimaryBolt` — marks the currently primary bolt (exactly one at any time). Lives in `bolt/components.rs`.
- `ExtraBolt` — marks non-primary bolts. Lives in `bolt/components.rs`.
- `BoundEffects` — the chip effect tree bound to a bolt. Lives in `effect_v3/`.
- `StagedEffects` — staged effects awaiting bolt dispatch. Lives in `effect_v3/`.

## Messages
**Reads**: `BumpPerformed { grade, bolt }`
**Sends**: None

## Systems

### `conductor_swap_on_perfect_bump`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. On `BumpGrade::Perfect` where the bumped bolt has `ExtraBolt` (and is not already `PrimaryBolt`):
  1. Removes `PrimaryBolt` from the current primary; inserts `ExtraBolt` on it.
  2. Removes `ExtraBolt` from the bumped bolt; inserts `PrimaryBolt` on it.
  3. Swaps `BoundEffects` and `StagedEffects` between the two entities via `swap_optional_component`.
  4. Sets `swap_done = true` — at most one swap per tick (subsequent messages in the same tick are drained without acting, so buffered messages do not replay next tick).
- Non-Perfect bumps: skipped.
- Bumps on non-`ExtraBolt` entities: skipped (only extra bolts can be promoted).
- Absent `ConductorConfig`: drains the reader and returns (harness-safety pattern).
- **Ordering**: `.after(BreakerSystems::GradeBump)`, `.before(EffectV3Systems::Bridge)` — mandatory upper bound so the swap happens before bump bridges walk the effect tree on the newly promoted bolt.

## Implementation Notes
- `swap_optional_component<C>` is a file-private helper that removes `C` from both entities and re-inserts the other's value. Both `BoundEffects` and `StagedEffects` implement `Clone`.
- No cleanup system — Conductor has no owned per-node state. `PrimaryBolt` / `ExtraBolt` / `BoundEffects` / `StagedEffects` lifecycle is managed by their respective domains.
- `register` does NOT call `init_resource`. `ConductorConfig` is installed by `activate`; there is no per-node tracking resource.

## Cross-Domain Dependencies
- **breaker domain**: Reads `BumpPerformed` message (bump grade + bolt entity).
- **bolt domain**: Reads `PrimaryBolt`, `ExtraBolt` components on bolt entities; writes them via `Commands`.
- **effect_v3 domain**: Reads `BoundEffects`, `StagedEffects` components on bolt entities; writes them via `Commands`. The swap happens before `EffectV3Systems::Bridge` so the bridge sees the post-swap state.

## Expected Behaviors (for test specs)

1. **Perfect bump on extra bolt swaps primary**
   - Given: Bolt A has `PrimaryBolt` + `BoundEffects(X)`. Bolt B has `ExtraBolt` + no effects.
   - When: Perfect bump on bolt B.
   - Then: Bolt B has `PrimaryBolt` + `BoundEffects(X)`. Bolt A has `ExtraBolt` + no effects.

2. **Non-perfect bump does not swap**
   - Given: Bolt A has `PrimaryBolt`. Bolt B has `ExtraBolt`.
   - When: Early bump on bolt B.
   - Then: Bolt A still has `PrimaryBolt`. Bolt B still has `ExtraBolt`. No component changes.

3. **Perfect bump on non-ExtraBolt is a no-op**
   - Given: Bolt A has `PrimaryBolt`. No other bolts.
   - When: Perfect bump on bolt A.
   - Then: No swap occurs (bolt A is not an `ExtraBolt`).

4. **At most one swap per tick**
   - Given: Bolt A has `PrimaryBolt`. Bolt B has `ExtraBolt`.
   - When: Two Perfect BumpPerformed messages for bolt B arrive in the same tick.
   - Then: Exactly one swap occurs. The second message is drained without acting.

5. **BoundEffects and StagedEffects both swap**
   - Given: Bolt A has `PrimaryBolt`, `BoundEffects(X)`, `StagedEffects(Y)`. Bolt B has `ExtraBolt`.
   - When: Perfect bump on bolt B.
   - Then: Bolt B has `BoundEffects(X)`, `StagedEffects(Y)`. Bolt A has neither.

6. **Swap when one side has no effects**
   - Given: Bolt A has `PrimaryBolt` + no `BoundEffects`. Bolt B has `ExtraBolt` + `BoundEffects(Z)`.
   - When: Perfect bump on bolt B.
   - Then: Bolt B has `PrimaryBolt` + no `BoundEffects`. Bolt A has `ExtraBolt` + `BoundEffects(Z)`.

7. **Absent ConductorConfig drains reader**
   - Given: Protocol inactive (no `ConductorConfig` resource). BumpPerformed message queued.
   - When: `conductor_swap_on_perfect_bump` runs.
   - Then: Reader is drained. No swap. No panic.

## Edge Cases
- Single bolt: no `ExtraBolt` entities exist, so every bump is skipped. Protocol has no effect (by design — "pointless with 1 bolt").
- Fission interaction: when a bolt splits (Fission protocol), the new bolt spawns with `ExtraBolt` and no effects. Player must Perfect Bump it to promote it.
- Effect-in-flight: effects already dispatched from a bolt before the swap are not recalled. The swap only affects the components at swap time, not in-flight effect state.
