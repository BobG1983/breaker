# Protocol: Debt Collector

## Category
`custom-system`

## Game Design
You WANT to deliberately do Early/Late bumps to build a damage multiplier, then cash out with a Perfect bump.

- Early or Late bump: stack += 0.5 debt.
- Perfect bump: next cell impact deals `base_damage * (1 + stack)`. Stack resets to 0.
- Stack does NOT persist across nodes.
- Stack IS lost on bolt-lost (punishment for losing the bolt you were building on).
- Scales multiplicatively with damage chips.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DebtCollectorConfig {
    pub stack_per_bump: f32,    // 0.5
}
```

## Components
```rust
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct DebtStack(pub f32);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct DebtCashOut(pub f32);
```

Both are `pub(crate)` — DebtCollector owns them; no other domain reads or writes them.

## Messages
**Reads**: `BumpPerformed { grade, bolt }` (breaker domain), `BoltLost { bolt }` (bolt domain).
**Sends**: None. Cash-out damage lands via `DamageBoostStack::add_one_shot` (Pattern B) on the bolt — the next `DamageDealt<Cell>` aggregation picks it up in `DmgSystems::ApplyDamageBoosts`.

## Systems

### `attach_debt_stack`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::DebtCollector)`.
- **Query**: `Query<Entity, Added<Bolt>>`.
- **Behavior**: For each newly spawned bolt, inserts `DebtStack::default()`. Canonical `Added<T>` attach pattern — fires exactly once per bolt at spawn.

### `attach_debt_stack_on_activate`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::DebtCollector)`.
- **Behavior**: Reads `ProtocolActivated { kind }` messages. If kind is `DebtCollector`: inserts `DebtStack::default()` on every existing `Bolt` entity. Covers mid-run activation (DebtCollector selected from chip-select during a node) where pre-existing bolts won't re-trigger `Added<Bolt>`.

### `debt_collector_on_bump`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. Early/Late: `DebtStack.0 += config.stack_per_bump`. Perfect: inserts `DebtCashOut(stack.0)` on the bolt, resets `DebtStack.0 = 0.0`.
- **Ordering**: `.after(BreakerSystems::GradeBump)`.

### `debt_collector_cash_out_on_bump`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`.
- **Behavior**: When a bolt has `DebtCashOut(n)`: calls `DamageBoostStack::add_one_shot(1.0 + n)` on the bolt (the "1.0 +" makes the base hit deal normal damage AND the stack multiplies it by `1 + n`). Removes `DebtCashOut`.
- **Ordering**: `.before(DmgSystems::ApplyDamageBoosts)` — one-shot must be present for the next aggregation.

### `debt_collector_on_bolt_lost`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BoltLost`. Resets the lost bolt's `DebtStack` to 0 and removes any `DebtCashOut` marker.

### `debt_collector_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)`.
- **Behavior**: Removes `DebtStack` + `DebtCashOut` from all bolts. Stack does not persist across nodes.

## Pipeline position (dmg crate)

- **Trigger**: `BumpPerformed` (Early/Late builds; Perfect cashes out).
- **Writes**: `DamageBoostStack::add_one_shot(1.0 + stack)` on the cashing-out bolt (Pattern B from `rantzsoft_dmg`).
- **Consumed in**: `DmgSystems::ApplyDamageBoosts` on the bolt's next `DamageDealt<Cell>` aggregation.
- **No direct damage emission** from DebtCollector.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed`.
- **bolt**: Reads `BoltLost`. Reads `Added<Bolt>`. Writes `DamageBoostStack::add_one_shot`.
- **damage crate (`rantzsoft_dmg`)**: Aggregates the one-shot at the bolt's next cell-impact damage emission.

## Expected Behaviors (for test specs)

1. **Early/Late bump adds to stack** — `DebtStack(0.0)`, `stack_per_bump = 0.5`, Early bump: `DebtStack(0.5)`.
2. **Multiple Early/Late bumps accumulate** — `DebtStack(1.0)` + Late bump: `DebtStack(1.5)`.
3. **Perfect bump creates cash-out and resets stack** — `DebtStack(1.5)`, Perfect: `DebtCashOut(1.5)` inserted, `DebtStack(0.0)`.
4. **Cash-out one-shot applied on next cell impact** — `DebtCashOut(1.5)`, bolt base damage 10: `add_one_shot(2.5)` pushed; on next impact, `DamageDealt<Cell>` aggregates to `10 * 2.5 = 25` (via `ApplyDamageBoosts`).
5. **Cash-out is single-shot** — `DamageBoostStack::one_shots` entry cleared on first consumption; second cell impact deals only base damage.
6. **Bolt-lost resets stack** — `BoltLost { bolt }`: `DebtStack(0.0)`, `DebtCashOut` removed.
7. **Stack does not persist across nodes** — `OnExit(Playing)`: all `DebtStack` + `DebtCashOut` removed.
8. **Zero-stack Perfect bump cashes out at 1.0x** — `DebtStack(0.0)`, Perfect: `add_one_shot(1.0)` → next hit deals base damage (no bonus).
9. **New bolt spawned mid-node gets DebtStack** — Fission spawns bolt; `Added<Bolt>` fires; `DebtStack::default()` inserted.
10. **Pre-existing bolt gets DebtStack on mid-run activation** — bolts exist, then DebtCollector is selected: `ProtocolActivated { kind: DebtCollector }` emitted; `attach_debt_stack_on_activate` inserts `DebtStack` on every live bolt.

## Edge Cases
- **Bolt-lost during cash-out flight**: `DebtCashOut` present but bolt lost before impact: boost disappears with the bolt.
- **Multiple bolts**: each tracks its own `DebtStack`. Perfect bump on bolt A doesn't affect bolt B.
- **Node end mid-stack**: all stacks cleared.
- **Interaction with Fission**: split bolts inherit `DebtStack::default()` (fresh 0.0); parent retains its stack.
