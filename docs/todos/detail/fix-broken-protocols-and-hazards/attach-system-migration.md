# Migrate attach-systems to `Added<T>` filter

Assumes: #2 (`mutators/protocols/`, `mutators/hazards/`). Affects DebtCollector (protocol), Renewal (hazard), Volatility (hazard) — any other mechanic using the same pattern gets migrated too; grep during impl.

## What's broken

Three attach-systems query `Query<Entity, (With<T>, Without<AttachedComponent>)>` every FixedUpdate and insert the missing component on any match. Two problems: the pattern is wrong (use `Added<T>`), and the naming is inconsistent:

| Current name | Attaches component | Naming issue |
|--------------|-------------------|--------------|
| `attach_stack` (DebtCollector) | `DebtStack` | missing mechanic prefix |
| `renewal_attach_timers` | `RenewalTimer` (singular) | system name is plural, verb in middle |
| `attach_volatility_timers` | `VolatilityTimer` (singular) | system name is plural |

Canonical naming convention (this remediation pins it): `attach_<component_snake_case>` — attach prefix, component name in snake_case, singular if the component is singular. Gives:

- `attach_debt_stack` (attaches `DebtStack`)
- `attach_renewal_timer` (attaches `RenewalTimer`)
- `attach_volatility_timer` (attaches `VolatilityTimer`)

No `_on_spawn` suffix — the `Added<T>` in the query signature communicates the trigger; the suffix is noise.

Pattern issue: Bevy provides `Added<T>` which matches entities added on the current tick exactly once. Replacing `Without<X>` with `Added<T>`:

- Each new entity triggers the attach exactly once, the frame it spawns.
- On ticks where nothing spawned, the query has zero results — no archetype scan overhead.
- No idempotency needed — the system sees each entity once.

`Added<T>` is the canonical Bevy idiom for "react to new entities of this kind."

## Fix

### Pattern

Each attach system changes from:

```rust
pub(crate) fn attach_x(
    mut commands: Commands,
    cells: Query<Entity, (With<Cell>, Without<X>)>,
    config: Res<XConfig>,
) {
    for entity in &cells {
        commands.entity(entity).insert(X::from_config(&config));
    }
}
```

to:

```rust
pub(crate) fn attach_x(
    mut commands: Commands,
    new_cells: Query<Entity, Added<Cell>>,
    config: Res<XConfig>,
) {
    for entity in &new_cells {
        commands.entity(entity).insert(X::from_config(&config));
    }
}
```

Same function name; query changes from `Without<X>` to `Added<Cell>`. The run-if gating (`hazard_active(X)` or equivalent) stays as-is.

### Per-consumer migrations

**DebtCollector** (`mutators/protocols/debt_collector/system.rs`):

```rust
pub(crate) fn attach_debt_stack(
    mut commands: Commands,
    new_bolts: Query<Entity, Added<Bolt>>,
    _config: Res<DebtCollectorConfig>,
) {
    for entity in &new_bolts {
        commands.entity(entity).insert(DebtStack::default());
    }
}
```

Schedule: `FixedUpdate`, `run_if = protocol_active(DebtCollector)`.

**Renewal** (`mutators/hazards/renewal/system.rs`):

```rust
pub(crate) fn attach_renewal_timer(
    mut commands: Commands,
    new_cells: Query<Entity, Added<Cell>>,
    config: Res<RenewalConfig>,
) {
    for entity in &new_cells {
        commands.entity(entity).insert(RenewalTimer::from_config(&config));
    }
}
```

Schedule: `FixedUpdate`, `run_if = hazard_active(Renewal)`.

**Volatility** (`mutators/hazards/volatility/system.rs`):

```rust
pub(crate) fn attach_volatility_timer(
    mut commands: Commands,
    new_cells: Query<(Entity, &Hp), Added<Cell>>,
    config: Res<VolatilityConfig>,
) {
    for (entity, hp) in &new_cells {
        let lifted_max = (hp.starting * 2.0).max(hp.max.unwrap_or(hp.starting));
        commands.entity(entity).insert((
            VolatilityTimer::from_config(&config),
            Hp { current: hp.current, starting: hp.starting, max: Some(lifted_max) },
        ));
    }
}
```

Schedule: `FixedUpdate`, `run_if = hazard_active(Volatility)`.

Note: Volatility's `Hp.max` lift used to query every cell every tick and ensure `max` was the max-of-(existing, starting * 2). Under `Added<Cell>`, the lift runs once at spawn — and since no other system should be increasing `Hp.max` between spawn-time and Volatility's attach, the max-of merge can simplify to a direct set. If any other hazard also modulates `Hp.max`, the `max-of` merge still applies (pre-existing `Hp.max` from the spawn includes any RON-level override; the lift picks the larger value).

### Gating consideration — activation mid-run

`Added<T>` only fires on entities added THIS tick. If a protocol/hazard activates with existing entities in the world, those entities do NOT retrigger `Added<T>`.

Per the project memory `project_hazards_activation.md`: hazards don't activate mid-node. When a hazard activates at node boundary, the node's cells spawn AFTER the hazard becomes active — `Added<Cell>` catches them. No migration problem for hazards.

**Protocols CAN activate mid-run** (chip selection). When DebtCollector is selected, existing bolts (primary + any live extras) won't re-trigger `Added<Bolt>`. Fix: DebtCollector's activation path also stamps `DebtStack` onto already-live bolts:

```rust
pub(crate) fn attach_debt_stack_on_activate(
    mut commands: Commands,
    mut reader: MessageReader<ProtocolActivated>,
    existing_bolts: Query<Entity, With<Bolt>>,
) {
    for msg in reader.read() {
        if msg.kind != ProtocolKind::DebtCollector { continue }
        for entity in &existing_bolts {
            commands.entity(entity).insert(DebtStack::default());
        }
    }
}
```

Naming: `attach_debt_stack_on_activate` keeps the `attach_<component>` root and appends `_on_activate` because the trigger is distinct (activation message, not entity spawn). The base `attach_debt_stack` handles all future spawns; this one-shot handles the activation moment.

Renewal and Volatility don't need the catch-up path (hazards don't activate mid-node).

### Delete

- The old `attach_stack` / `renewal_attach_timers` / `attach_volatility_timers` every-tick systems with `Without<X>` filters.
- Any `commands.entity(entity).insert_if_new(...)` calls that exist because the old system ran idempotently.

### Test update

Existing attach tests that simulated "tick → entity gets component" need rewriting:

- `Added<T>` only fires on spawn. Tests that inserted a cell via `commands.spawn((Cell, ...))` and then ticked to check for attach still work — spawn + next tick sees `Added<Cell>`.
- Tests that mutated existing cells (removed and re-added the component) no longer work — `Added<T>` doesn't fire on component insert, only on entity spawn. Rewrite such tests to spawn fresh cells.

## Tests to author

`mutators/protocols/debt_collector/tests/attach_on_spawn.rs`:

1. **`new_bolt_gets_debt_stack`** — activate DebtCollector; spawn a bolt; tick; assert `DebtStack` present.
2. **`bolt_existing_before_activation_gets_debt_stack`** — spawn a bolt BEFORE DebtCollector activates; emit `ProtocolActivated { kind: DebtCollector }`; tick; assert the pre-existing bolt NOW has `DebtStack`.
3. **`post_activation_spawn_path_gates_on_protocol_active`** — with DebtCollector INACTIVE, spawn a bolt; tick; assert NO `DebtStack` (because the attach-on-spawn system has `run_if = protocol_active(DebtCollector)`).

`mutators/hazards/renewal/tests/attach_on_spawn.rs`:

4. **`new_cell_gets_renewal_timer`** — activate Renewal (at node boundary); spawn a cell; tick; assert `RenewalTimer` present.
5. **`cell_spawned_before_hazard_active_does_not_get_timer`** — spawn cell, then activate Renewal mid-tick (shouldn't happen in practice but test the contract). Assert no retroactive attach.

`mutators/hazards/volatility/tests/attach_on_spawn.rs`:

6. **`new_cell_gets_volatility_timer_and_hp_max_lift`** — activate Volatility; spawn cell with `Hp { starting: 2.0, max: None, .. }`; tick; assert `VolatilityTimer` present; assert `Hp.max == Some(4.0)` (lifted to `starting * 2`).

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/debt_collector/system.rs` | REPLACE `attach_stack` (every-tick `Without<DebtStack>`) with `attach_debt_stack` (`Added<Bolt>`) + `attach_debt_stack_on_activate` (`MessageReader<ProtocolActivated>`) |
| `mutators/protocols/debt_collector/register.rs` | Register both new systems; drop the old one |
| `mutators/hazards/renewal/system.rs` | RENAME `renewal_attach_timers` → `attach_renewal_timer` (singular, matches component); query from `Without<RenewalTimer>` → `Added<Cell>` |
| `mutators/hazards/volatility/system.rs` | RENAME `attach_volatility_timers` → `attach_volatility_timer` (singular); query from `Without<VolatilityTimer>` → `Added<Cell>`; `Hp.max` lift happens inline at spawn |
| `mutators/protocols/debt_collector/tests/attach.rs` | NEW — tests 1-3 |
| `mutators/hazards/renewal/tests/attach.rs` | NEW — tests 4-5 |
| `mutators/hazards/volatility/tests/attach.rs` | NEW — test 6 |
| Existing attach tests in each consumer | REWRITE to use fresh-spawn pattern (the `Added<T>` contract) |

## Out of scope

- Documentation updates (`Added<T>` is standard Bevy; no architectural write-up needed; no new canonical-pattern doc).
- Any other mechanic using the every-tick-`Without<X>` pattern that isn't in the three listed here — grep during impl and fold in if found.
- Replacing `Added<T>` with `OnAdd` observers — equivalent correctness, different ergonomics; stick with `Added<T>` query filters to match the existing codebase's style unless observers are already the dominant pattern in the affected modules.
