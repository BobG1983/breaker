# Protocol: Echo Strike

## Category
`custom-system`

## Game Design
You WANT to Perfect Bump into specific cells to build an echo network, then hit them all simultaneously.

- Perfect Bump followed by cell impact: that cell becomes an echo (max 3 echoes, FIFO).
- On subsequent Perfect Bump + cell impact: damages the current target AND all active echoes.
- Echo damage falls off by age: newest ~50%, middle ~25%, oldest ~10% of the impact damage.
- Echoes clear on node end.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct EchoStrikeConfig {
    pub max_echoes: u32,            // 3
    pub newest_fraction: f32,       // 0.5
    pub middle_fraction: f32,       // 0.25
    pub oldest_fraction: f32,       // 0.1
}
```

## Components
```rust
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct EchoNetwork {
    /// Ordered oldest → newest; max length = config.max_echoes.
    pub echoes: VecDeque<Entity>,
}

#[derive(Component, Debug)]
pub(crate) struct EchoPrimed;
```

Both `pub(crate)` — owned by EchoStrike.

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltImpactCell { cell, bolt }` (bolt domain), `Destroyed<Cell>` (from `rantzsoft_dmg`).
**Sends**: `DamageDealt<Cell> { cell, damage, source }` in `DeathPipelineSystems::EmitDamage` for each echo cell that takes echo damage. Source tag: `"protocol:echo_strike"`.

## Systems

### `attach_echo_network`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::EchoStrike)`.
- **Query**: `Query<Entity, Added<Bolt>>`.
- **Behavior**: Inserts `EchoNetwork::default()` on each newly spawned bolt.

### `attach_echo_network_on_activate`
- **Schedule**: `FixedUpdate`, reads `ProtocolActivated`.
- **Behavior**: On mid-run Echo Strike selection, inserts `EchoNetwork::default()` on every existing bolt.

### `echo_strike_on_bump`
- **Schedule**: `FixedUpdate`, `.after(BreakerSystems::GradeBump)`.
- **run_if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. On `BumpGrade::Perfect`: inserts `EchoPrimed` on the bolt.

### `echo_strike_on_impact`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::EmitDamage`.
- **run_if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BoltImpactCell`. If the bolt has `EchoPrimed`:
  1. For each existing entry in `EchoNetwork.echoes`, emits `DamageDealt<Cell> { cell: echo_cell, damage: impact_damage * fraction_for_position, source: "protocol:echo_strike" }`. Fraction selected by position: newest, middle (when 3 active), oldest.
  2. Pushes `impact_cell` to the back of `EchoNetwork.echoes`; evicts front if length > `max_echoes`.
  3. Removes `EchoPrimed` from the bolt.
- **Ordering**: In `EmitDamage` set. Runs after bolt-cell collision (which already emits the base `DamageDealt<Cell>` for the impact cell).

### `echo_strike_cleanup_destroyed_echoes`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill)`.
- **Behavior**: Reads `Destroyed<Cell>`. Removes the destroyed cell from every bolt's `EchoNetwork.echoes`.

### `echo_strike_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Removes `EchoNetwork` + `EchoPrimed` from all bolts.

## Pipeline position (dmg crate)

- **Trigger**: `BumpPerformed` (Perfect) → primes bolt. `BoltImpactCell` → emits echo damage.
- **Emits**: `DamageDealt<Cell>` in `DeathPipelineSystems::EmitDamage` — one per echo cell still alive. Each emitted message flows through the full chain (boosts, mutate, vulnerable, apply).
- **Reads**: `Destroyed<Cell>` from `rantzsoft_dmg` for cleanup.
- **No** `DamageBoostStack` / `VulnerableStack` writes.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed`.
- **bolt**: Reads `BoltImpactCell`; reads `Added<Bolt>`.
- **cells / damage crate**: Emits `DamageDealt<Cell>`. Reads `Destroyed<Cell>`.

## Expected Behaviors (for test specs)

1. **Perfect bump primes the bolt** — Perfect `BumpPerformed`: bolt gains `EchoPrimed`.
2. **Non-perfect bump does NOT prime** — Early/Late: no `EchoPrimed`.
3. **Primed bolt registers echo on cell impact (no prior echoes)** — `EchoNetwork.echoes` empty: impact on cell A adds A to network; `EchoPrimed` removed; no echo damage emitted.
4. **Primed bolt deals echo damage to existing echoes** — `EchoNetwork.echoes = [A]`, impact on B, base damage 10, `newest_fraction = 0.5`: emits `DamageDealt<Cell> { cell: A, damage: 5 }`; B added; `EchoPrimed` removed.
5. **Three echoes with age-based falloff** — `echoes = [A, B, C]`, base damage 20, fractions 0.5/0.25/0.1, impact on D: emits damage to C (newest, 10), B (middle, 5), A (oldest, 2); A evicted; D added → `[B, C, D]`.
6. **FIFO eviction at max capacity** — `max_echoes = 3`, `echoes = [A, B, C]` + new echo: A evicted.
7. **Destroyed echo cell removed from network** — `Destroyed<Cell> { A }`: A removed from every bolt's `EchoNetwork`.
8. **Echoes clear on node end** — `OnExit(Playing)`: `EchoNetwork` + `EchoPrimed` removed from all bolts.
9. **Non-primed bolt impact has no echo effect** — bolt without `EchoPrimed`: no echo damage; network unchanged.

## Edge Cases
- **Echo cell destroyed between prime and impact**: cleanup system removes it; damage only goes to surviving echoes.
- **Same cell hit twice while primed**: pre-existing entry takes echo damage from its old position; then the entry is re-added at the newest position (moves to back).
- **Multi-bolt with Conductor**: each bolt has its own `EchoNetwork`. Role swaps don't transfer echoes (effects swap, networks don't — they're scoped to the tracked bolt identity).
- **Impact damage of 0**: no `DamageDealt<Cell>` emitted for 0-damage echoes.
