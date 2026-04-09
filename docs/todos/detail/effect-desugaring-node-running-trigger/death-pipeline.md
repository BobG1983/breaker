# Unified Death Pipeline (merged from todo #7)

Absorbed from the original "Killed trigger, Die effect, unified death messaging" todo. Full design lives here; original detail file at `detail/killed-trigger-damage-attribution.md` is the historical reference.

## GameEntity Trait

```rust
/// Marker trait for entity types that participate in the death pipeline.
/// Impl'd on every domain entity type that can be killed or kill.
trait GameEntity: Component {}

impl GameEntity for Bolt {}
impl GameEntity for Cell {}
impl GameEntity for Wall {}
impl GameEntity for Breaker {}
```

## Messages

```rust
struct KillYourself<T: GameEntity> {
    pub victim: Entity,
    pub killer: Option<Entity>,
}

struct Destroyed<T: GameEntity> {
    pub victim: Entity,
    pub killer: Option<Entity>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
}
```

Generic on victim type (T). Each T is a separate Bevy message queue. Killer type is classified at runtime from `KilledBy.dealer` entity's components, not via a type parameter.

## Death Chain

```
Fire(Die) evaluates on entity
  → sends KillYourself<T> { victim: self, killer: from TriggerContext }

Domain kill handler receives KillYourself<T>
  → checks invuln/shield (can ignore — no Destroyed sent)
  → sends Destroyed<T> { victim, killer, positions }
  → does NOT despawn yet

bridge_destroyed<T> receives Destroyed<T>
  → fires Killed(KillTarget) on KILLER entity only
  → fires Died on VICTIM entity only
  → fires DeathOccurred(DeathTarget) GLOBALLY on all entities with BoundEffects
  → entity survives through trigger evaluation + death animation
  → despawn after via DespawnEntity message (processed in PostFixedUpdate)
```

## Triggers from Death Events

| Trigger | Fires on | Participants |
|---|---|---|
| `Killed(KillTarget)` | Killer only | `::Killer`, `::Victim` |
| `Died` | Victim only | `::Victim`, `::Killer` |
| `DeathOccurred(DeathTarget)` | All entities globally | `::Entity`, `::Killer` |

## DeathAttribution

Death attribution works via runtime killer classification. When `detect_*_deaths` reads `KilledBy.dealer`, it inspects the dealer entity's components to determine what kind of entity performed the kill (Bolt, Breaker, Cell, etc.). This replaces the old compile-time `(S, T)` generic pairs — the victim type is known from the specialized detect system, and the killer type is resolved at runtime.

```rust
/// Classify the killer entity at runtime by inspecting its components.
fn classify_killer(entity: Entity, world: &World) -> Option<KillTarget> {
    if world.get::<Bolt>(entity).is_some() { return Some(KillTarget::Bolt); }
    if world.get::<Breaker>(entity).is_some() { return Some(KillTarget::Breaker); }
    if world.get::<Cell>(entity).is_some() { return Some(KillTarget::Cell); }
    None // environmental death
}
```

## Valid Killer/Victim Pairs

| Killer (runtime) | T (victim) | Source |
|---|---|---|
| Bolt | `Cell` | Direct hit + bolt-sourced effects |
| Breaker | `Cell` | Breaker-cell collision |
| Cell | `Cell` | Chain reaction |
| Bolt | `Wall` | One-shot wall hit |
| None (environmental) | `Bolt` | Bolt lost, lifespan expiry |
| None (environmental) | `Wall` | Timer expiry |

## Unified Damage Message

All damage sources send a generic message type per victim type. One system per victim type processes damage and tracks kill attribution.

```rust
/// Generic damage message — one Bevy message queue per victim type T.
/// Sent by: bolt collision, shockwave fire(), chain lightning fire(),
/// explode fire(), any effect that deals damage.
struct DamageDealt<T: GameEntity> {
    pub dealer: Option<Entity>,      // who caused this damage (propagated through chains)
    pub target: Entity,              // who takes the damage
    pub amount: f32,                 // damage amount
    pub source_chip: Option<String>, // which chip originated this damage (for UI/stats)
    _marker: PhantomData<T>,
}
```

Usage: `DamageDealt<Cell>` replaces the old `DamageCell`. `DamageDealt<Bolt>`, `DamageDealt<Wall>`, etc. for other entity types.

### apply_damage system

Processes all `DamageDealt<T>` messages, decrements HP, and sets KilledBy **only on the killing blow** — the hit that crosses HP from positive to zero.

**Locked cells**: `apply_damage::<Cell>` skips entities with the `Locked` component — locked cells cannot take damage. This system must be ordered AFTER `check_lock_release` so that cells unlocked this frame can still receive damage.

```rust
fn apply_damage<T: GameEntity>(
    mut messages: MessageReader<DamageDealt<T>>,
    mut query: Query<(&mut Hp, &mut KilledBy), Without<Locked>>,
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

## Death Detection Systems

Per-domain specialized systems run after `apply_damage`. Each queries its domain's health component + marker to detect newly dead entities. Killer type is classified at runtime from `KilledBy.dealer`.

```rust
// In cells/ domain
fn detect_cell_deaths(
    query: Query<(Entity, &KilledBy, &Hp), (With<Cell>, Changed<Hp>)>,
    mut kill_messages: MessageWriter<KillYourself<Cell>>,
    // ...
) {
    for (entity, source, hp) in &query {
        if hp.current <= 0 {
            kill_messages.send(KillYourself {
                victim: entity,
                killer: source.dealer,
            });
        }
    }
}

// In bolt/ domain
fn detect_bolt_deaths(
    query: Query<(Entity, &KilledBy, &Hp), (With<Bolt>, Changed<Hp>)>,
    mut kill_messages: MessageWriter<KillYourself<Bolt>>,
    // ...
) { /* same pattern */ }

// In wall/ domain — if walls have HP
fn detect_wall_deaths(
    query: Query<(Entity, &KilledBy, &Hp), (With<Wall>, Changed<Hp>)>,
    mut kill_messages: MessageWriter<KillYourself<Wall>>,
    // ...
) { /* same pattern */ }
```

## DespawnEntity Message

```rust
/// Sent after death animations and trigger evaluation complete.
/// Lives in shared/messages.rs.
struct DespawnEntity {
    pub entity: Entity,
}

/// Processes all pending despawn requests. Runs in PostFixedUpdate.
fn process_despawn_requests(
    mut messages: MessageReader<DespawnEntity>,
    mut commands: Commands,
) {
    for msg in messages.read() {
        commands.entity(msg.entity).despawn();
    }
}
```

## Node Completion Tracking

`track_node_completion` queries `Has<RequiredToClear>` from the still-alive entity. The entity survives until `DespawnEntity` processes in PostFixedUpdate, so the `RequiredToClear` component is still accessible when `Destroyed<Cell>` is handled in FixedUpdate. No extra field on `Destroyed<T>` is needed.

## TriggerContext Integration

`TriggerContext` fields map to named trigger participants. Bridges populate fields from `Destroyed<T>` messages:
- `Killed(KillTarget)` fires on killer → `context` has DeathContext { victim, killer }
- `Died` fires on victim → same context, different perspective
- `DeathOccurred` fires globally → same context on all entities
- Killer is `Option<Entity>` — if None, `Killed` is not fired (environmental death)

## Types Eliminated

| Before | After |
|---|---|
| `DamageCell` | `DamageDealt<Cell>` |
| `RequestCellDestroyed` | `KillYourself<Cell>` |
| `CellDestroyedAt` | `Destroyed<Cell>` |
| `RequestBoltDestroyed` | `KillYourself<Bolt>` |
| `Trigger::CellDestroyed` | `DeathOccurred(Cell)` |
| `Trigger::Death` | `DeathOccurred(DeathTarget)` |
| `bridge_cell_destroyed` | `bridge_destroyed::<Cell>` |
| `bridge_death` | N × `bridge_destroyed::<T>` |
| `bridge_died` | collapsed into `bridge_destroyed` |
| `PendingDespawn` | `DespawnEntity` message + `process_despawn_requests` |

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
