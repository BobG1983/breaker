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

## Unified Damage Message

All damage sources send a generic message type per victim type. One system per victim type processes damage and tracks kill attribution.

```rust
/// Generic damage message — one Bevy message queue per victim type T.
/// Sent by: bolt collision, shockwave fire(), chain lightning fire(),
/// explode fire(), any effect that deals damage.
struct DamageDealt<T: Component> {
    pub dealer: Option<Entity>,  // who caused this damage (propagated through chains)
    pub target: Entity,          // who takes the damage
    pub amount: f32,             // damage amount
    _marker: PhantomData<T>,
}
```

Usage: `DamageDealt<Cell>` replaces the old `DamageCell`. `DamageDealt<Bolt>`, `DamageDealt<Wall>`, etc. for other entity types.

### apply_damage system

Processes all `DamageDealt<T>` messages, decrements HP, and sets KilledBy **only on the killing blow** — the hit that crosses HP from positive to zero.

```rust
fn apply_damage<T: Component>(
    mut messages: MessageReader<DamageDealt<T>>,
    mut query: Query<(&mut Hp, &mut KilledBy)>,
) {
    for msg in messages.read() {
        if let Ok((mut hp, mut source)) = query.get_mut(msg.target) {
            let was_alive = hp.current > 0;
            hp.current -= msg.amount;
            if was_alive && hp.current <= 0 {
                // This is the killing blow — set attribution
                source.dealer = msg.dealer;
            }
        }
    }
}
```

### KilledBy component

```rust
/// Set by apply_damage on the killing blow. Read by the death system.
#[derive(Component, Default)]
struct KilledBy {
    pub dealer: Option<Entity>,
}
```

### Damage propagation through effect chains

Effects that deal damage read the current TriggerContext to propagate the dealer:

```rust
// In shockwave fire():
//   TriggerContext has the bolt that caused this shockwave
//   Shockwave sends DamageDealt<Cell> { dealer: context.bolt(), ... }

// In explode fire() (from powder keg Transfer):
//   TriggerContext has the DeathContext of the cell that exploded
//   Explosion sends DamageDealt<Cell> { dealer: death_context.killer, ... }
//   (bolt B killed the cell, so bolt B gets credit for explosion kills)

// In chain lightning fire():
//   TriggerContext has the bolt that caused the chain
//   Each arc sends DamageDealt<Cell> { dealer: context.bolt(), ... }
```

### Replaces

| Before | After |
|---|---|
| `DamageCell` | `DamageDealt<Cell>` (generic per victim type) |
| Direct HP mutation in effect systems | All damage flows through `DamageDealt<T>` → `apply_damage::<T>` |
| No kill attribution | `KilledBy` set on killing blow only |

## Death System

Runs after `apply_damage`. Queries for entities with `Hp.current <= 0`:

```rust
fn detect_deaths(
    query: Query<(Entity, &KilledBy, &Hp), Changed<Hp>>,
    // ...
) {
    for (entity, source, hp) in &query {
        if hp.current <= 0 {
            // Verify killer still exists before sending Killed
            let killer = source.dealer.filter(|&e| world.get_entity(e).is_some());

            // Send KillYourself → domain handler → Destroyed → bridge triggers
            send_kill_yourself(entity, killer);
        }
    }
}
```

## TriggerContext Integration

`TriggerContext` fields map to named trigger participants. Bridges populate fields from `Destroyed<S, T>` messages:
- `Killed(KillTarget)` fires on killer → `context` has DeathContext { victim, killer }
- `Died` fires on victim → same context, different perspective
- `DeathOccurred` fires globally → same context on all entities
- Killer is `Option<Entity>` — if None, `Killed` is not fired (environmental death)

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

## RON Examples (final syntax)

```ron
// Kill reward: "every cell I kill boosts my speed"
Route(Bolt, When(Killed(Cell), Fire(SpeedBoost(1.3))))

// Cell revenge: "when I die, explode"
Route(Cell, When(Died, Fire(Explode(range: 48.0, damage: 10.0))))

// Cascade: "when any cell dies, shockwave from me"
Route(Bolt, When(DeathOccurred(Cell), Fire(Shockwave(
    base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 300.0,
))))

// Powder keg: "when I hit a cell, transfer 'when you die, explode' onto it"
Route(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Transfer(
    When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
))))

// One-shot wall: "when hit by bolt, kill myself"
Route(Wall, When(Impacted(Bolt), Fire(Die)))

// Generic kill reward: "every kill boosts my speed"
Route(Bolt, When(Killed(Any), Fire(SpeedBoost(1.1))))
```

## RON Migration (55 files — validated)

See [ron-migration/](ron-migration/) for all migrated files and [ron-migration/VALIDATION_REPORT.md](ron-migration/VALIDATION_REPORT.md) for the validation results.

```ron
// Before                              // After
On(target: Bolt, then: [...])      →  Route(Bolt, ...)
On(target: Breaker, then: [...])   →  Route(Breaker, ...)
Do(...)                            →  Fire(...)
On(This, Fire(...))                →  Fire(...) (implicit This)
On(target: Cell, ...)              →  On(ImpactTarget::Impactee, ...)
When(trigger: CellDestroyed, ...)  →  When(Killed(Cell), ...) or When(DeathOccurred(Cell), ...)
When(trigger: Death, ...)          →  When(DeathOccurred(Any), ...)
```
