# Protocol: Conductor

## Category
`custom-system`

## Game Design
You WANT to catch the RIGHT bolt, not just any bolt. Multi-bolt target selection.

- With multiple bolts, only the "Conducted" (primary) bolt gets your chip effects.
- Perfect Bump a bolt: that bolt becomes Conducted (primary).
- Non-primary bolt-loss doesn't count as bolt-lost.
- Edge case: perfect bump bolt A while bolt B (current primary) is about to be lost — A becomes primary, B's loss doesn't trigger bolt-lost.
- Pointless with 1 bolt — changes how multi-bolt plays.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct ConductorConfig {
    /// Grace window (seconds) after a primary swap during which the old primary's
    /// bolt-lost is suppressed. Covers the edge case where the old primary is lost
    /// in the same frame or shortly after the swap (default tunable, e.g. 0.1).
    pub primary_swap_window: f32,
}
```

Populated from `ProtocolTuning::Conductor { primary_swap_window }`.

## Components
```rust
/// Marks the currently Conducted (primary) bolt. Only one bolt should have this
/// at a time. The bolt with this marker receives chip effects; other bolts do not.
#[derive(Component, Debug)]
pub(crate) struct Conducted;

/// Tracks whether this bolt was recently demoted from Conducted status,
/// and suppresses bolt-lost within the swap grace window.
#[derive(Component, Debug)]
pub(crate) struct ConductorSwapGrace {
    /// Remaining time in the grace window (seconds).
    pub remaining: f32,
}
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltLost`, `RequestBoltDestroyed { bolt }`
**Sends**: None (modifies which bolt-lost events are suppressed, does not generate new messages)

## Systems

### `conductor_swap_primary`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: Reads `BumpPerformed` messages. On `BumpGrade::Perfect`: removes `Conducted` from the current primary bolt (if any), inserts `ConductorSwapGrace { remaining: config.primary_swap_window }` on the old primary, inserts `Conducted` on the newly bumped bolt. On non-Perfect bumps: no swap occurs.
- **Ordering**: After breaker `grade_bump`.

### `conductor_tick_swap_grace`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: Decrements `ConductorSwapGrace.remaining` by `delta_secs` for all entities with the component. Removes the component when `remaining <= 0.0`.
- **Ordering**: Before `conductor_suppress_bolt_lost`.

### `conductor_suppress_bolt_lost`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: Intercepts `BoltLost` and `RequestBoltDestroyed` messages. For each bolt-lost event: if the lost bolt does NOT have `Conducted` AND does NOT have `ConductorSwapGrace` — suppress the bolt-lost event (prevent downstream life loss / time penalty). If the lost bolt HAS `Conducted` — allow the bolt-lost event to proceed normally. If the lost bolt has `ConductorSwapGrace` — suppress (it was just swapped away, grace period active).
- **Implementation note**: Suppression mechanism depends on how the bolt-lost pipeline works. Options: (a) consume the `BoltLost` message and do not re-emit, requiring this system to sit between `bolt_lost` and `bridge_bolt_lost`; (b) insert a marker component that `bridge_bolt_lost` checks before triggering life loss. Option (b) is cleaner (no message interception).
- **Ordering**: After bolt `bolt_lost`, before effect `bridge_bolt_lost`.

### `conductor_filter_chip_effects`
- **Schedule**: Wherever chip effect dispatch runs
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: Modifies chip effect dispatch so that only the bolt with `Conducted` receives chip effects. Non-primary bolts still bounce and deal base damage, but do not trigger or receive any chip/effect tree outcomes. Implementation: the effect dispatch system checks for `Conducted` before applying effects to a bolt entity.
- **Implementation note**: This likely requires a query filter addition to the existing effect dispatch path rather than a standalone system. The protocol domain provides the `Conducted` marker; the effect domain reads it.
- **Ordering**: N/A — integrated into existing effect dispatch.

### `conductor_auto_assign_initial`
- **Schedule**: `FixedUpdate`
- **run_if**: `protocol_active(ProtocolKind::Conductor)`, `in_state(NodeState::Playing)`
- **Behavior**: If no bolt currently has `Conducted`, automatically assigns it to the first available bolt. This handles node start (first bolt spawned gets `Conducted`) and the case where the primary bolt is destroyed (next bolt inherits).
- **Ordering**: After bolt spawn systems, after `conductor_suppress_bolt_lost`.

## Cross-Domain Dependencies
- **breaker domain**: Reads `BumpPerformed` message (bump grade + bolt entity).
- **bolt domain**: Reads `BoltLost`, `RequestBoltDestroyed` messages. Writes `Conducted`, `ConductorSwapGrace` components on bolt entities.
- **effect domain**: Effect dispatch path needs to respect `Conducted` marker — only dispatch chip effects to the Conducted bolt. This is a cross-domain read (effect reads bolt's `Conducted` component).
- **chips domain**: Chip effect resolution should only apply to `Conducted` bolt when Conductor protocol is active.

## Expected Behaviors (for test specs)

1. **Perfect bump swaps primary bolt**
   - Given: Bolt A has `Conducted`. Bolt B exists without `Conducted`.
   - When: Perfect bump on bolt B.
   - Then: Bolt B has `Conducted`. Bolt A does not have `Conducted`. Bolt A has `ConductorSwapGrace`.

2. **Non-perfect bump does not swap**
   - Given: Bolt A has `Conducted`. Bolt B exists.
   - When: Early bump on bolt B.
   - Then: Bolt A still has `Conducted`. Bolt B does not have `Conducted`.

3. **Non-primary bolt-lost is suppressed**
   - Given: Bolt A has `Conducted`. Bolt B does not.
   - When: Bolt B falls below playfield (bolt-lost).
   - Then: No life loss / time penalty triggered. Bolt B is destroyed normally (entity cleanup) but the penalty pipeline is not invoked.

4. **Primary bolt-lost triggers normal penalty**
   - Given: Bolt A has `Conducted`.
   - When: Bolt A falls below playfield (bolt-lost).
   - Then: Normal bolt-lost penalties apply (life loss, time penalty, etc.).

5. **Swap grace suppresses old primary bolt-lost**
   - Given: Bolt A has `Conducted`. Bolt B exists.
   - When: Perfect bump on bolt B (A loses `Conducted`, gains `ConductorSwapGrace`). Within grace window, bolt A falls below playfield.
   - Then: Bolt A's bolt-lost is suppressed (no penalty). Bolt B is now primary.

6. **Grace window expires normally**
   - Given: Bolt A has `ConductorSwapGrace { remaining: 0.1 }`.
   - When: 0.1 seconds elapse.
   - Then: `ConductorSwapGrace` removed from bolt A. If bolt A is subsequently lost, normal non-primary suppression applies (still suppressed because A is not `Conducted`).

7. **Only Conducted bolt receives chip effects**
   - Given: Bolt A has `Conducted` with chip effects (e.g., Piercing). Bolt B does not have `Conducted`.
   - When: Bolt B impacts a cell.
   - Then: No chip effects fire (no piercing, no chain lightning, etc.). Base damage only.

8. **Auto-assign on node start**
   - Given: Conductor protocol active. New node starts. First bolt spawned.
   - When: `conductor_auto_assign_initial` runs.
   - Then: The spawned bolt receives `Conducted`.

9. **Auto-assign when primary is destroyed**
   - Given: Bolt A has `Conducted`, bolt B exists. Bolt A is destroyed (primary lost).
   - When: `conductor_auto_assign_initial` runs next frame.
   - Then: Bolt B receives `Conducted`.

## Edge Cases
- Single bolt: `Conducted` is always on the only bolt. Protocol has no meaningful effect (by design — "pointless with 1 bolt").
- All bolts lost simultaneously: normal bolt-lost triggers for the `Conducted` bolt; others suppressed. Game over logic proceeds from the single penalty.
- Perfect bump on already-Conducted bolt: no-op (bolt already has `Conducted`, no swap needed, no grace window inserted).
- Fission interaction: when a bolt splits (Fission protocol), the new bolt does NOT inherit `Conducted`. Player must Perfect Bump it to make it primary.
- Effect-in-flight: if bolt B fires chip effects mid-flight and then a swap makes bolt A primary, bolt B's already-fired effects are not recalled. The filter only applies at dispatch time.
