# Unified Death Pipeline (merged from todo #7)

Absorbed from the original "Killed trigger, Die effect, unified death messaging" todo. Full design lives here; original detail file at `detail/killed-trigger-damage-attribution.md` is the historical reference.

## Messages

```rust
struct KillYourself<S: Component, T: Component> {
    pub victim: Entity,
    pub killer: Option<Entity>,
}

struct Destroyed<S: Component, T: Component> {
    pub victim: Entity,
    pub killer: Option<Entity>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
}
```

Generic on killer type (S) and victim type (T). Each (S, T) pair is a separate Bevy message queue.

## Death Chain

```
Fire(Die) evaluates on entity
  → sends KillYourself<S, T> { victim: self, killer: from TriggerContext }

Domain kill handler receives KillYourself<S, T>
  → checks invuln/shield (can ignore — no Destroyed sent)
  → sends Destroyed<S, T> { victim, killer, positions }
  → does NOT despawn yet

bridge_destroyed<S, T> receives Destroyed<S, T>
  → fires Killed(KillTarget) on KILLER entity only
  → fires Died on VICTIM entity only
  → fires DeathOccurred(DeathTarget) GLOBALLY on all entities with BoundEffects
  → entity survives through trigger evaluation + death animation
  → despawn after via PendingDespawn
```

## Triggers from Death Events

| Trigger | Fires on | Participants |
|---|---|---|
| `Killed(KillTarget)` | Killer only | `::Killer`, `::Victim` |
| `Died` | Victim only | `::Victim`, `::Killer` |
| `DeathOccurred(DeathTarget)` | All entities globally | `::Entity`, `::Killer` |

## DeathAttribution Trait

```rust
trait DeathAttribution: Send + Sync + 'static {
    fn kill_target() -> KillTarget;
    fn death_target() -> DeathTarget;
}

impl DeathAttribution for (Bolt, Cell) {
    fn kill_target() -> KillTarget { KillTarget::Cell }
    fn death_target() -> DeathTarget { DeathTarget::Cell }
}
```

## Valid (S, T) Pairs

| S (killer) | T (victim) | Source |
|---|---|---|
| `Bolt` | `Cell` | Direct hit + bolt-sourced effects |
| `Breaker` | `Cell` | Breaker-cell collision |
| `Cell` | `Cell` | Chain reaction |
| `Bolt` | `Wall` | One-shot wall hit |
| `()` | `Bolt` | Bolt lost, lifespan expiry |
| `()` | `Wall` | Timer expiry |

## Generic Damage: DamageDealt<T>

Replaces `DamageCell`:

```rust
struct DamageDealt<T: Component> {
    pub target: Entity,
    pub damage: f32,
    pub source_chip: Option<String>,
    pub source_entity: Option<Entity>,
}
```

## TriggerContext Integration

`TriggerContext` fields map to named trigger participants. Bridges populate fields from `Destroyed<S, T>` messages:
- `Killed(KillTarget)` fires on killer → `context.bolt/breaker/cell = killer`, victim fields populated
- `Died` fires on victim → same context, different perspective
- `DeathOccurred` fires globally → same context on all entities

## Types Eliminated

| Before | After |
|---|---|
| `DamageCell` | `DamageDealt<Cell>` |
| `RequestCellDestroyed` | `KillYourself<S, Cell>` |
| `CellDestroyedAt` | `Destroyed<S, Cell>` |
| `RequestBoltDestroyed` | `KillYourself<(), Bolt>` |
| `Trigger::CellDestroyed` | `DeathOccurred(Cell)` |
| `Trigger::Death` | `DeathOccurred(DeathTarget)` |
| `bridge_cell_destroyed` | `bridge_destroyed::<S, Cell>` |
| `bridge_death` | N × `bridge_destroyed::<S, T>` |
| `bridge_died` | collapsed into `bridge_destroyed` |

## RON Examples (new vocabulary)

```ron
// Kill reward: "every cell I kill boosts my speed"
When(Killed(Cell), On(This, Fire(SpeedBoost(1.3))))

// Cell revenge: "when I die, explode"
When(Died, On(This, Fire(Explode(range: 48.0, damage: 10.0))))

// Cascade: "when any cell dies, shockwave from me"
When(DeathOccurred(Cell), On(This, Fire(Shockwave(
    base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 300.0,
))))

// Powder keg: "when I hit a cell, stamp 'when you die, explode' on it"
When(Impacted(Cell), On(Impacted::Target, Transfer(
    When(Died, On(This, Fire(Explode(range: 48.0, damage: 10.0))))
)))

// One-shot wall: "when hit by bolt, kill myself"
When(Impacted(Bolt), On(This, Fire(Die)))

// Generic kill reward: "every kill boosts my speed"
When(Killed(Any), On(This, Fire(SpeedBoost(1.1))))
```

## RON Migration (~17 files)

```ron
// Before                              // After
When(trigger: CellDestroyed, ...)  →  When(DeathOccurred(Cell), ...)
When(trigger: Death, ...)          →  When(DeathOccurred(Any), ...)
Do(...)                            →  Fire(...)
On(target: Bolt, then: [...])      →  When/On with new vocabulary
```
