# Protocol: Echo Strike

## Category
`custom-system`

## Game Design
You WANT to Perfect Bump into specific cells to build an echo network, then hit them all simultaneously.

- Perfect Bump followed by cell impact: that cell becomes an echo (max 3 echoes, FIFO).
- On subsequent Perfect Bump followed by cell impact: damages the current target AND all active echoes.
- Echo damage falloff by age: newest ~50%, middle ~25%, oldest ~10% of the impact damage.
- Echoes cleared on node end.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct EchoStrikeConfig {
    /// Maximum number of echo cells tracked (FIFO eviction when full).
    pub max_echoes: u32,
    /// Fraction of impact damage dealt to the newest echo.
    pub newest_fraction: f32,
    /// Fraction of impact damage dealt to the middle echo (when 3 echoes active).
    pub middle_fraction: f32,
    /// Fraction of impact damage dealt to the oldest echo.
    pub oldest_fraction: f32,
}
```

## Components
```rust
/// Tracks the echo network for Echo Strike. One instance per bolt (or global resource if
/// single-bolt only — per-bolt allows interaction with Conductor/multi-bolt).
/// Stored as a VecDeque for FIFO eviction.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct EchoNetwork {
    /// Ordered from oldest (front) to newest (back). Max length = config.max_echoes.
    pub echoes: VecDeque<Entity>,
}

/// Marker: this bolt's next cell impact will register the target as an echo
/// AND deal echo damage to all existing echoes.
#[derive(Component, Debug)]
pub(crate) struct EchoPrimed;
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactCell { cell, bolt }`, `CellDestroyedAt { position }`
**Sends**: `DamageDealt<Cell> { cell, damage, source_chip }` for each echo cell that takes echo damage

## Systems

### `echo_strike_on_bump`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`
- **What it does**: Reads `BumpPerformed` messages. If grade is `Perfect`, marks the bolt with `EchoPrimed` marker. If grade is Early/Late, does nothing (no echo priming on non-perfect bumps).
- **Ordering**: After `BreakerSystems::GradeBump`.

### `echo_strike_on_impact`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`
- **What it does**:
  1. Reads `BoltImpactCell { cell, bolt }` messages.
  2. If the bolt has `EchoPrimed`:
     a. Deal echo damage to all cells currently in the bolt's `EchoNetwork` (damage = impact damage * fraction based on age position).
     b. Add the impacted cell to the `EchoNetwork` (push back). If network is at max capacity, evict the oldest (pop front).
     c. Remove the `EchoPrimed` marker.
  3. If the bolt does NOT have `EchoPrimed`: normal impact, no echo effects.
- **Ordering**: After bolt-cell collision detection. Needs to know the impact damage (read from bolt's damage component or the `DamageDealt<Cell>` message that the collision system already sends).

### `echo_strike_cleanup_destroyed_echoes`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`
- **What it does**: Reads `CellDestroyedAt` messages. Removes any destroyed cell entities from all `EchoNetwork` components. (A destroyed cell can't be an echo target.)
- **Ordering**: After cell destruction.

### `echo_strike_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)` or `OnEnter(NodeState::Transitioning)`
- **What it does**: Removes all `EchoNetwork`, `EchoPrimed` components. Echoes do not persist across nodes.

### Echo Damage Calculation
The fraction assigned to each echo depends on its age position:
- With 1 echo: that echo gets `newest_fraction` (e.g., 0.5).
- With 2 echoes: newest gets `newest_fraction` (0.5), oldest gets `oldest_fraction` (0.1).
- With 3 echoes: newest gets `newest_fraction` (0.5), middle gets `middle_fraction` (0.25), oldest gets `oldest_fraction` (0.1).

Fraction assignment is by position in the deque, not by a fixed slot. When there are 2 echoes, the "middle" slot is unused.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed` message (bump grade + bolt entity).
- **bolt**: Reads `BoltImpactCell` message. Attaches `EchoNetwork` and `EchoPrimed` components to bolt entities.
- **cells**: Sends `DamageDealt<Cell>` messages for echo damage. Reads `CellDestroyedAt` to clean up destroyed echoes.

## Expected Behaviors (for test specs)

1. **Perfect Bump primes the bolt**
   - Given: Echo Strike active, bolt entity exists
   - When: `BumpPerformed { grade: BumpGrade::Perfect, bolt }` is sent
   - Then: Bolt gets `EchoPrimed` marker

2. **Non-perfect bump does NOT prime**
   - Given: Echo Strike active, bolt entity exists
   - When: `BumpPerformed { grade: BumpGrade::Early, bolt }` is sent
   - Then: Bolt does NOT get `EchoPrimed`

3. **Primed bolt registers echo on cell impact**
   - Given: Bolt has `EchoPrimed`, `EchoNetwork` is empty, cell A at (50.0, 100.0)
   - When: `BoltImpactCell { cell: A, bolt }` is sent
   - Then: Cell A is added to `EchoNetwork`. `EchoPrimed` is removed. No echo damage sent (no prior echoes).

4. **Primed bolt deals echo damage to existing echoes**
   - Given: Bolt has `EchoPrimed`, `EchoNetwork` contains [cell A], impact damage = 10.0, `newest_fraction: 0.5`
   - When: `BoltImpactCell { cell: B, bolt }` is sent
   - Then: `DamageDealt<Cell> { cell: A, damage: 5.0 }` sent (10.0 * 0.5). Cell B added to network. `EchoPrimed` removed.

5. **Three echoes with age-based falloff**
   - Given: `EchoNetwork` contains [A (oldest), B (middle), C (newest)], impact damage = 20.0, `newest_fraction: 0.5`, `middle_fraction: 0.25`, `oldest_fraction: 0.1`
   - When: Primed bolt impacts cell D
   - Then: `DamageDealt<Cell> { cell: C, damage: 10.0 }` (newest, 0.5), `DamageDealt<Cell> { cell: B, damage: 5.0 }` (middle, 0.25), `DamageDealt<Cell> { cell: A, damage: 2.0 }` (oldest, 0.1). Cell A evicted (FIFO), D added. Network becomes [B, C, D].

6. **FIFO eviction at max capacity**
   - Given: `max_echoes: 3`, `EchoNetwork` contains [A, B, C]
   - When: Primed bolt impacts cell D
   - Then: A is evicted (oldest). Network becomes [B, C, D].

7. **Destroyed echo cell is removed from network**
   - Given: `EchoNetwork` contains [A, B], cell A is destroyed
   - When: `CellDestroyedAt` for cell A is sent
   - Then: Network becomes [B]. No crash or stale reference.

8. **Echoes cleared on node end**
   - Given: `EchoNetwork` contains [A, B, C]
   - When: Node transitions out of `NodeState::Playing`
   - Then: All `EchoNetwork` and `EchoPrimed` components removed.

9. **Non-primed bolt impact has no echo effect**
   - Given: Bolt without `EchoPrimed`, `EchoNetwork` contains [A, B]
   - When: `BoltImpactCell { cell: C, bolt }` is sent
   - Then: No echo damage. Network unchanged. Cell C is NOT added to network.

## Edge Cases
- **Echo cell destroyed between prime and impact**: If an echo cell dies after being registered but before the next primed impact, `echo_strike_cleanup_destroyed_echoes` removes it. Damage is only sent to surviving echoes.
- **Same cell hit twice**: A cell already in the echo network that is impacted again as the "new" cell gets re-added (moved to newest position in FIFO). It also takes echo damage from its old position before being re-registered.
- **Multi-bolt with Conductor**: Each bolt has its own `EchoNetwork`. Only the conducted bolt benefits from chip effects that affect impact damage, but all bolts independently track echoes.
- **Node end mid-echo**: All echoes cleared. No carry-over.
- **Impact damage of 0**: Echo damage would be 0 * fraction = 0. No `DamageDealt<Cell>` sent for 0-damage echoes.
