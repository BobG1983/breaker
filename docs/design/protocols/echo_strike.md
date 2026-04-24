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

### `echo_strike_emit_siblings`
- **Schedule**: `FixedUpdate`, in `DmgSystems::PostApply`.
- **run_if**: `protocol_active(ProtocolKind::EchoStrike)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads current-frame `DamageDealt<Cell>` messages. For each
  post-apply primary:
  1. Skip if `source == "protocol:echo_strike"` (loop protection).
  2. Skip if `msg.amount <= 0.0` — invulnerable filter zeroed the primary;
     enforces the unified "invulnerable source → no ripple" rule shared
     with Diffusion and Tether.
  3. Resolve the candidate bolt via `msg.dealer.or(msg.attributed_to)`. This
     lets echoes fire on BOTH primary bump-damage (`dealer = Some(bolt)`) AND
     ripple damage from Diffusion / Tether (`dealer = None`,
     `attributed_to = Some(bolt)`).
  4. Skip if the bolt is not queryable or not `EchoPrimed`. A
     `processed_this_frame` guard prevents same-tick pierce double-fires.
  5. For each existing entry in `EchoNetwork.echoes`, emit
     `DamageDealt<Cell> { target: echo, amount: msg.amount * fraction,
     source: Some(SourceId::from("protocol:echo_strike")), dealer: None,
     attributed_to: msg.attributed_to.or(msg.dealer), .. }`. Fractions
     follow the deque-size rule (newest, middle for 3+, oldest). Siblings
     traverse the full damage pipeline on the next `FixedUpdate` tick
     (1-frame delay).
  6. Mutate `EchoNetwork`: dedup then `push_back(msg.target)`; `pop_front`
     if length exceeds `max_echoes`.
  7. Remove `EchoPrimed` from the bolt.

### `echo_strike_cleanup_destroyed_echoes`
- **Schedule**: `FixedUpdate`, `.after(DmgSystems::ApplyKill)`.
- **Behavior**: Reads `Destroyed<Cell>`. Removes the destroyed cell from every bolt's `EchoNetwork.echoes`.

### `echo_strike_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Removes `EchoNetwork` + `EchoPrimed` from all bolts.

## Pipeline position (dmg crate)

- **Trigger**: `BumpPerformed` (Perfect) → primes bolt via `echo_strike_on_bump`.
- **Ripple emitter** in `DmgSystems::PostApply`. Reads the post-apply
  `DamageDealt<Cell>` messages and emits one echo sibling per entry in the
  bolt's `EchoNetwork.echoes`. Siblings traverse the FULL pipeline on the
  next `FixedUpdate` tick (1-frame delay).
- **Loop protection** via `source == "protocol:echo_strike"` — siblings
  carry that source, so subsequent PostApply passes skip them.
- **Invulnerable skip** via `msg.amount <= 0.0`.
- **Kill attribution** travels via `attributed_to = msg.attributed_to.or(msg.dealer)`.
- **Reads**: `Destroyed<Cell>` from `rantzsoft_dmg` for cleanup.
- **No** `DamageBoostStack` / `VulnerableStack` writes.
- The prior `ECHO_STRIKE_SENTINEL` constant is deleted — the source string
  is now inline `SourceId::from("protocol:echo_strike")`.

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
