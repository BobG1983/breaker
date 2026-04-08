# Protocol: Debt Collector

## Category
`custom-system`

## Game Design
You WANT to deliberately do Early/Late bumps to build a damage multiplier, then cash out with a Perfect bump.

- Early or Late bump: stack += 0.5x multiplier
- Perfect bump: next cell impact deals `normal damage * (1 + stack)`. Stack resets to 0.
- Stack does NOT persist across nodes.
- Stack IS lost on bolt-lost (punishment for losing the bolt you were building on).
- Scales multiplicatively with damage chips.

## Config Resource
```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DebtCollectorConfig {
    /// Multiplier added to the stack per Early or Late bump.
    pub stack_per_bump: f32,
}
```

## Components
```rust
/// Per-bolt debt multiplier stack. Attached to each bolt entity when Debt Collector is active.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct DebtStack(pub f32);
```

## Messages
**Reads**: `BumpPerformed { grade, bolt }`, `BoltLost`, `BoltImpactCell { cell, bolt }`
**Sends**: `DamageDealt<Cell> { cell, damage, source_chip }` (with bonus damage on cash-out)

## Systems

### `debt_collector_on_bump`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`
- **What it does**: Reads `BumpPerformed` messages. For Early/Late grades, adds `config.stack_per_bump` to the bolt's `DebtStack`. For Perfect grade, marks the bolt as "cashing out" (sets a `DebtCashOut` marker component with the raw accumulated stack value) and resets the stack to 0.
- **Ordering**: After `BreakerSystems::GradeBump` (bump grade must be determined first).

### `debt_collector_on_impact`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`
- **What it does**: Reads `BoltImpactCell` messages. If the impacting bolt has a `DebtCashOut` marker, sends a bonus `DamageDealt<Cell>` message with `damage = normal_damage * stack_value`. The normal hit already deals `normal_damage`, so total damage = `normal_damage * (1 + stack)`. Removes the `DebtCashOut` marker after applying (one-shot: only the NEXT cell impact gets the bonus, not all impacts until next bump).
- **Ordering**: After bolt-cell collision detection (so `BoltImpactCell` has been sent).

### `debt_collector_on_bolt_lost`
- **Schedule**: `FixedUpdate`
- **Run if**: `protocol_active(ProtocolKind::DebtCollector)` + `in_state(NodeState::Playing)`
- **What it does**: Reads `BoltLost` messages. Resets the lost bolt's `DebtStack` to 0 and removes any `DebtCashOut` marker. (Punishment: you lose your accumulated stack when a bolt is lost.)
- **Ordering**: After bolt-lost detection.

### `debt_collector_attach_stack`
- **Schedule**: `FixedUpdate` or `OnEnter(NodeState::Playing)`
- **Run if**: `protocol_active(ProtocolKind::DebtCollector)`
- **What it does**: Attaches `DebtStack::default()` to any bolt entity that doesn't already have one. Ensures new bolts (spawned mid-node via Fission or other mechanics) get tracked.
- **Ordering**: After bolt spawning systems.

### `debt_collector_cleanup_node`
- **Schedule**: `OnExit(NodeState::Playing)` or `OnEnter(NodeState::Transitioning)`
- **What it does**: Removes all `DebtStack` and `DebtCashOut` components. Stack does not persist across nodes.

## Cross-Domain Dependencies
- **breaker**: Reads `BumpPerformed` message (bump grade + bolt entity).
- **bolt**: Reads `BoltLost` message. Reads `BoltImpactCell` message. Attaches components to bolt entities.
- **cells**: Sends `DamageDealt<Cell>` message for bonus damage on cash-out.
- **effect**: The bonus damage scales multiplicatively with damage chips. The cash-out damage is sent as a separate `DamageDealt<Cell>` message which the damage pipeline processes normally (chip damage multipliers already applied to the base hit; Debt Collector's bonus is additive on top of that base, then scaled by the stack).

## Expected Behaviors (for test specs)

1. **Early/Late bump adds to stack**
   - Given: Bolt with `DebtStack(0.0)`, `DebtCollectorConfig { stack_per_bump: 0.5 }`
   - When: `BumpPerformed { grade: BumpGrade::Early, bolt }` is sent
   - Then: Bolt's `DebtStack` becomes `0.5`

2. **Multiple Early/Late bumps accumulate**
   - Given: Bolt with `DebtStack(1.0)`, `config.stack_per_bump = 0.5`
   - When: `BumpPerformed { grade: BumpGrade::Late, bolt }` is sent
   - Then: Bolt's `DebtStack` becomes `1.5`

3. **Perfect bump marks cash-out and resets stack**
   - Given: Bolt with `DebtStack(1.5)`
   - When: `BumpPerformed { grade: BumpGrade::Perfect, bolt }` is sent
   - Then: Bolt gets `DebtCashOut(2.5)` marker (1.0 + 1.5 stack). `DebtStack` resets to `0.0`.

4. **Cash-out applies bonus damage on next cell impact**
   - Given: Bolt with `DebtCashOut(2.5)`, normal base damage = 10.0
   - When: `BoltImpactCell { cell, bolt }` is sent
   - Then: Bonus `DamageDealt<Cell> { cell, damage: 15.0 }` is sent (10.0 * 1.5 stack). Normal hit adds 10.0, total = 25.0. `DebtCashOut` is removed.

5. **Cash-out is one-shot (only next impact)**
   - Given: Bolt had `DebtCashOut` removed after first impact
   - When: Second `BoltImpactCell { cell, bolt }` is sent
   - Then: No bonus damage. Normal damage only.

6. **Bolt-lost resets stack**
   - Given: Bolt with `DebtStack(2.0)` and `DebtCashOut(3.0)`
   - When: `BoltLost` is sent for that bolt
   - Then: `DebtStack` resets to `0.0`. `DebtCashOut` is removed.

7. **Stack does not persist across nodes**
   - Given: Bolt with `DebtStack(1.5)` at end of node
   - When: Node ends (transition out of `NodeState::Playing`)
   - Then: All `DebtStack` and `DebtCashOut` components are removed.

8. **Perfect bump with zero stack still works**
   - Given: Bolt with `DebtStack(0.0)`
   - When: `BumpPerformed { grade: BumpGrade::Perfect, bolt }` is sent
   - Then: `DebtCashOut(0.0)` is set (0.0 stack). No bonus damage on impact — normal hit only.

9. **New bolts spawned mid-node get DebtStack**
   - Given: Debt Collector is active, a new bolt is spawned (e.g., via Fission)
   - When: Bolt entity appears without `DebtStack`
   - Then: `DebtStack(0.0)` is attached automatically.

## Edge Cases
- **Bolt-lost during cash-out flight**: If a bolt has `DebtCashOut` but is lost before impacting a cell, the cash-out is wasted (bolt-lost clears both stack and cash-out).
- **Multiple bolts**: Each bolt tracks its own independent `DebtStack`. Perfect bump on bolt A doesn't affect bolt B's stack.
- **Node end mid-stack**: All stacks cleared. No carry-over.
- **Zero-stack Perfect bump**: Results in `DebtCashOut(0.0)` — no bonus damage, just the normal hit. The math works cleanly (bonus = base × 0.0 = 0).
- **Interaction with Fission**: A split bolt inherits no stack (gets fresh `DebtStack(0.0)`). The parent bolt keeps its stack.
