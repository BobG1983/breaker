# Die Effect + Unified Death Messaging

## Summary
Generic entity death pattern: `Die` effect → `KillYourself<S, T>` message → domain decides → `Destroyed<S, T>` message. Replaces `RequestCellDestroyed`, `CellDestroyedAt`, `RequestBoltDestroyed`, `Trigger::CellDestroyed`, and `Trigger::Death` with a single unified system.

## Context
Emerged from wall builder design. Currently cell death and bolt death have separate bespoke message types and trigger systems. `CellDestroyed` is really just `Death(Cell)`. This pattern unifies all entity death into one generic flow, eliminating per-domain death message types and collapsing two trigger systems into one.

## Design

### Messages (generic on victim only, 2 types — replaces 4+)

```rust
/// Request to kill an entity. Domain systems decide whether to honor it.
/// T = victim entity type marker (Cell, Bolt, Wall, Breaker).
#[derive(Message, Clone)]
struct KillYourself<T: Component> {
    pub victim: Entity,
    pub killer: Option<Entity>,  // None = self-inflicted (timer, Die effect)
    _marker: PhantomData<T>,
}

/// Broadcast that an entity was destroyed. Effect system listens for triggers.
#[derive(Message, Clone)]
struct Destroyed<T: Component> {
    pub victim: Entity,
    pub killer: Option<Entity>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
    _marker: PhantomData<T>,
}
```

**Why victim-only, no killer type param `<S, T>`:** (see research/generic-message-patterns.md)
- Bevy `MessageReader<T>` is consuming — domains must have separate queues per victim type
- Killer type as `S` would be `PhantomData` — `killer: Option<Entity>` carries the same info
- Dropping `S` halves registration calls (8 total instead of 16+)
- `bridge_death` already uses multiple `MessageReader` params — extending linearly is clean

Concrete instantiations:
- `KillYourself<Cell>` — "kill this cell" (killer entity tracked via `killer` field)
- `KillYourself<Wall>` — "kill this wall" (from Die effect, timer expiry, etc.)
- `Destroyed<Cell>` — "cell was destroyed at position"
- `Destroyed<Wall>` — "wall was destroyed at position"

### Death Chain

```
EffectKind::Die fires on target entity
  → effect system resolves target type (Wall/Bolt/Cell/Breaker)
  → sends KillYourself<S, T>(killer, victim)
  
Domain system receives KillYourself<S, T>
  → checks: should this entity die? (invuln? shield? armor?)
  → if NO: ignore, done
  → if YES:
    → play death animation / VFX / sound
    → despawn entity
    → send Destroyed<S, T> { killer, victim, killer_pos, victim_pos }

Effect trigger system receives Destroyed<S, T>
  → fires Trigger::Death(DeathTarget) globally on all entities with BoundEffects
  → TriggerContext carries the dying entity for On(Cell)/On(Wall)/etc. resolution
```

### Trigger Unification

**Before:**
- `Trigger::Death` — "anything died" (global)
- `Trigger::CellDestroyed` — "a cell died" (global, cell-specific)
- `bridge_death` system — reads `RequestCellDestroyed` + `RequestBoltDestroyed`
- `bridge_cell_destroyed` system — reads `CellDestroyedAt`

**After:**
- `Trigger::Death(DeathTarget)` — single trigger with discriminant
- `DeathTarget::Cell`, `DeathTarget::Bolt`, `DeathTarget::Wall`, `DeathTarget::Breaker`, `DeathTarget::Any`
- One `bridge_death` system reads all `Destroyed<S, T>` messages

**RON migration:**
```ron
// Before
When(trigger: CellDestroyed, then: [...])
When(trigger: Death, then: [...])

// After
When(trigger: Death(Cell), then: [...])
When(trigger: Death(Any), then: [...])
```

### Types Eliminated
- `RequestCellDestroyed` → `KillYourself<Cell>`
- `CellDestroyedAt` → `Destroyed<Cell>`
- `RequestBoltDestroyed` → `KillYourself<Bolt>`
- `Trigger::CellDestroyed` → `Trigger::Death(Cell)`
- `bridge_cell_destroyed` system → collapsed into `bridge_death`

### Systems Eliminated
- `effect/triggers/cell_destroyed.rs` — entire file, collapsed into `bridge_death`

### Domain Kill Handlers
Each domain implements a system that receives `KillYourself` and decides:

```rust
// Wall domain
fn handle_kill_wall(
    mut kill_requests: MessageReader<KillYourself<Wall>>,
    walls: Query<(&Position2D, Option<&Invulnerable>), With<Wall>>,
    mut destroyed_writer: MessageWriter<Destroyed<Wall>>,
) {
    for request in kill_requests.read() {
        let Ok((pos, invuln)) = walls.get(request.victim) else { continue };
        if invuln.is_some() { continue; }  // ignore — invulnerable
        destroyed_writer.write(Destroyed {
            victim: request.victim,
            killer: request.killer,
            victim_pos: pos.0,
            killer_pos: None, // looked up if needed
            _marker: PhantomData,
        });
        // Do NOT despawn yet — entity must survive for Died trigger + death animation
        // Despawn happens after animation completes or at end of frame if no animation
    }
}
```

Cell, bolt, breaker domains follow the same pattern.

### Invulnerability Extension Point
Domains that ignore `KillYourself` simply don't send `Destroyed`. No downstream effects fire. A shielded bolt, armored cell, or invulnerable wall — same pattern, different component check.

## RON Files to Migrate (~15)
- `cascade.chip.ron` (3 tiers) — `CellDestroyed` → `Death(Cell)`
- `chain_reaction.chip.ron` — `CellDestroyed` → `Death(Cell)`
- `splinter.chip.ron` (3 tiers) — `CellDestroyed` → `Death(Cell)`
- `feedback_loop.chip.ron` — `CellDestroyed` → `Death(Cell)`
- `entropy_engine.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `voltchain.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `gravity_well.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `supernova.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `split_decision.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `chain_reaction.evolution.ron` — `CellDestroyed` → `Death(Cell)`
- `breaker.example.ron` — update trigger list in comments

## Scope
- **In**: `EffectKind::Die`, `KillYourself<S, T>` generic message, `Destroyed<S, T>` generic message, `Trigger::Death(DeathTarget)` with discriminant, `DeathTarget` enum, domain kill-handler systems (wall, cell, bolt, breaker), `bridge_death` unification, RON migration, eliminate `RequestCellDestroyed`/`CellDestroyedAt`/`RequestBoltDestroyed`/`bridge_cell_destroyed`
- **Out**: Invulnerability components (future), death animations/VFX (Phase 5)

## Dependencies
- Depends on: Wall builder pattern (introduces first wall death use case)
- Replaces: Existing cell death messaging (`RequestCellDestroyed`, `CellDestroyedAt`)
- Replaces: `Trigger::CellDestroyed` and `cell_destroyed.rs` trigger system
- Relates to: Killed trigger + damage source attribution (todo #7) — `KillYourself<S, T>` carries killer info

## Notes
- The generic `<S, T>` parameterization means Rust's type system enforces that you can't accidentally send a `Destroyed<Bolt, Cell>` from the wall domain. Each domain only sends messages with its own victim type.
- `killer: Option<Entity>` handles self-inflicted death (timer expiry, `Die` effect with no causer) by being `None`.
- The `was_required_to_clear` field on current `RequestCellDestroyed` / `CellDestroyedAt` is cell-domain-specific. It can move to a cell-domain component or be carried in a cell-specific wrapper around the generic message.
- This pattern directly supports the Killed trigger todo (#7) — damage attribution is just "who sent the `KillYourself`?"

## Status
`[NEEDS DETAIL]` — despawn timing needs design (entity must survive for `Died` trigger evaluation + death animation before actual despawn). `was_required_to_clear` migrates to a cell-domain component, read by `handle_kill_cell` — straightforward. Research resolved: victim-only generic `KillYourself<T>` / `Destroyed<T>`, see research/generic-message-patterns.md.
