# Trigger Systems

Each trigger type has its own module in `effect/triggers/`. Each module contains a `register()` function and a bridge system.

## Bridge System Pattern

```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, bridge_bolt_lost.after(BoltSystems::BoltLost).run_if(...));
}

fn bridge_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 { return; }

    // BoltLost is global: evaluate every entity with chains
    for (entity, chains, mut armed) in &mut query {
        // Walk StagedEffects (consumed on match), then BoundEffects (permanent)
        // Do → commands.fire_effect(entity, effect)
        // non-Do children of matching When → push to StagedEffects
        // On → resolve target, commands.fire_effect / commands.transfer_effect
    }
}
```

Trigger systems are **normal Bevy systems** — no exclusive world access, no parallelism blocking.

## What Each Trigger System Knows

- **Its scope**: global (iterate all entities with BoundEffects) or targeted (specific entities from message). See [design/triggers/](../../design/triggers/index.md) for the full scope table.
- **Its message**: what game message it bridges (BoltImpactCell, BumpPerformed, BoltLost, etc.)
- **Its On resolution**: how to resolve Target values to entities from message data. Each trigger has different context — Impact has both collision participants, BoltLost has none, etc.

## Global vs Targeted

**Global trigger systems** iterate ALL entities with BoundEffects:
```rust
for (entity, chains, mut armed) in &mut query {
    // evaluate this entity's chains against the trigger
}
```

**Targeted trigger systems** evaluate specific entities only:
```rust
if let Ok((entity, chains, armed)) = query.get_mut(bolt_entity) {
    // evaluate bolt's chains for Impacted(Cell)
}
if let Ok((entity, chains, armed)) = query.get_mut(cell_entity) {
    // evaluate cell's chains for Impacted(Bolt)
}
```

## Example: Impact and Impacted bridging BoltImpactCell

A single collision message (`BoltImpactCell { bolt, cell }`) is handled by separate systems in `impact.rs` (global) and `impacted.rs` (targeted). Each collision type gets its own system — one per message type per module.

### impact.rs — One system per collision type, global triggers

```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, (
        bridge_impact_bolt_cell.after(BoltSystems::CellCollision),
        bridge_impact_bolt_wall.after(BoltSystems::CellCollision),
        bridge_impact_bolt_breaker.after(BoltSystems::BreakerCollision),
        // future: bridge_impact_breaker_cell, bridge_impact_breaker_wall, bridge_impact_cell_wall
    ).run_if(in_state(PlayingState::Active)));
}

/// BoltImpactCell → Impact(Cell) global + Impact(Bolt) global
fn bridge_impact_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Impact(Cell): sweep all entities
        for (entity, chains, mut armed) in &mut query {
            // walk chains matching Trigger::Impact(ImpactTarget::Cell)
            // On resolution: Target::Bolt → msg.bolt, Target::Cell → msg.cell
        }

        // Impact(Bolt): sweep all entities
        for (entity, chains, mut armed) in &mut query {
            // walk chains matching Trigger::Impact(ImpactTarget::Bolt)
            // On resolution: Target::Bolt → msg.bolt, Target::Cell → msg.cell
        }
    }
}

/// BoltImpactWall → Impact(Wall) global + Impact(Bolt) global
fn bridge_impact_bolt_wall(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Impact(Wall): sweep all entities
        // Impact(Bolt): sweep all entities
        // On resolution: Target::Bolt → msg.bolt, Target::Wall → msg.wall
    }
}

// bridge_impact_bolt_breaker, bridge_impact_breaker_cell, etc. — same pattern
```

### impacted.rs — One system per collision type, targeted triggers

```rust
pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, (
        bridge_impacted_bolt_cell.after(BoltSystems::CellCollision),
        bridge_impacted_bolt_wall.after(BoltSystems::CellCollision),
        bridge_impacted_bolt_breaker.after(BoltSystems::BreakerCollision),
        // future: bridge_impacted_breaker_cell, bridge_impacted_breaker_wall, bridge_impacted_cell_wall
    ).run_if(in_state(PlayingState::Active)));
}

/// BoltImpactCell → Impacted(Cell) on bolt + Impacted(Bolt) on cell
fn bridge_impacted_bolt_cell(
    mut reader: MessageReader<BoltImpactCell>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Impacted(Cell) on the bolt: "you were in an impact with a cell"
        if let Ok((entity, chains, mut armed)) = query.get_mut(msg.bolt) {
            // walk chains matching Trigger::Impacted(ImpactTarget::Cell)
            // On resolution: Target::Bolt → msg.bolt, Target::Cell → msg.cell
        }

        // Impacted(Bolt) on the cell: "you were in an impact with a bolt"
        if let Ok((entity, chains, mut armed)) = query.get_mut(msg.cell) {
            // walk chains matching Trigger::Impacted(ImpactTarget::Bolt)
            // On resolution: Target::Bolt → msg.bolt, Target::Cell → msg.cell
        }
    }
}

/// BoltImpactWall → Impacted(Wall) on bolt + Impacted(Bolt) on wall
fn bridge_impacted_bolt_wall(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Impacted(Wall) on bolt, Impacted(Bolt) on wall
        // On resolution: Target::Bolt → msg.bolt, Target::Wall → msg.wall
    }
}

// bridge_impacted_bolt_breaker, bridge_impacted_breaker_cell, etc. — same pattern
```

### The full picture for one BoltImpactCell message

The bolt domain detects the collision and sends `BoltImpactCell { bolt, cell }`. Four systems pick it up (two in impact.rs, two... actually just one each):

1. **bridge_impact_bolt_cell** fires two global triggers:
   - `Impact(Cell)` — sweeps all entities, evaluates chains matching `Impact(Cell)`
   - `Impact(Bolt)` — sweeps all entities, evaluates chains matching `Impact(Bolt)`

2. **bridge_impacted_bolt_cell** fires two targeted triggers:
   - `Impacted(Cell)` — evaluates only `msg.bolt`'s chains
   - `Impacted(Bolt)` — evaluates only `msg.cell`'s chains

Both systems have the same On resolution context: `Target::Bolt → msg.bolt`, `Target::Cell → msg.cell`. If a chain on the bolt says `On(target: Cell, then: [...])`, the system resolves Cell to `msg.cell` and transfers the children there.
