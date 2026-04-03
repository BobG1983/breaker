# Idiom Research: Generic Message Patterns for Entity Death

## Context

The project wants to replace four bespoke death-messaging types
(`RequestCellDestroyed`, `CellDestroyedAt`, `RequestBoltDestroyed`, and the
orphaned `Trigger::CellDestroyed`) with a unified generic pattern. The proposed
design has two generic structs `KillYourself<S, T>` and `Destroyed<S, T>` where
`S` and `T` are domain marker types (`Bolt`, `Cell`, `Wall`, `Breaker`).

The constraints the pattern must satisfy:

1. Domain systems only see kill-requests for their own victim type
   (`KillYourself<_, Wall>` for the wall domain, etc.)
2. The effect trigger bridge (`bridge_death`) must react to every `Destroyed`
   message regardless of which concrete `<S, T>` pair was produced.
3. Messages must satisfy Bevy 0.18's `Message + Clone` bound, registered per
   concrete type via `app.add_message::<T>()`.
4. Registration boilerplate must not scale with the full N×M cartesian product
   of all killer/victim combinations.

### What Bevy 0.18's message system is

`Message` in this codebase is a derive macro (`#[derive(Message, Clone, Debug)]`).
`MessageReader<T>` and `MessageWriter<T>` are ordinary system parameters. Each
distinct `T` is a completely separate FIFO queue — there is no shared queue or
runtime type erasure. Generic struct instantiations (`KillYourself<Bolt, Cell>`
vs `KillYourself<Wall, Cell>`) are treated as unrelated types by Bevy.

---

## The Four Options Evaluated

### Option A — Fully generic `KillYourself<S, T>` / `Destroyed<S, T>`

```rust
#[derive(Message, Clone, Debug)]
struct KillYourself<S: 'static + Send + Sync, T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    _marker: PhantomData<(S, T)>,
}

#[derive(Message, Clone, Debug)]
struct Destroyed<S: 'static + Send + Sync, T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    pub killer_pos: Option<Vec2>,
    pub victim_pos: Vec2,
    _marker: PhantomData<(S, T)>,
}
```

**Registration**: `app.add_message::<KillYourself<Bolt, Cell>>()`  
Each valid `(S, T)` pair is a separate registration call. With 4 types and
roughly 6 valid combinations in practice (`Bolt→Cell`, `Bolt→Wall`,
`Wall→Bolt`, `()`→`Wall`, `Bolt→Bolt`, `Bolt→Breaker`) that is 6 registration
calls — acceptable.

**Domain receive**: The wall domain writes one reader:

```rust
fn handle_kill_wall_by_bolt(
    mut reader: MessageReader<KillYourself<Bolt, Wall>>,
    ...
```

To accept *any* killer the wall domain needs one reader per killer type:

```rust
mut bolt_kills: MessageReader<KillYourself<Bolt, Wall>>,
mut bolt_kills2: MessageReader<KillYourself<(), Wall>>,   // timer-expiry
```

Or it needs a separate concrete wrapper (see Option C below).

**The "read ALL Destroyed" problem** is the critical blocker. The effect trigger
system currently has:

```rust
fn bridge_death(
    mut cell_reader: MessageReader<RequestCellDestroyed>,
    mut bolt_reader: MessageReader<RequestBoltDestroyed>,
    ...
```

With fully generic types it becomes:

```rust
fn bridge_death(
    mut bolt_cell: MessageReader<Destroyed<Bolt, Cell>>,
    mut bolt_wall: MessageReader<Destroyed<Bolt, Wall>>,
    mut wall_bolt: MessageReader<Destroyed<Wall, Bolt>>,
    mut unit_wall: MessageReader<Destroyed<(), Wall>>,
    ...
```

This already mirrors the existing pattern — `bridge_death` already has two
readers. Adding two more as new entity types gain death support is not a
regression; it is equivalent growth. The parameter count is bounded by the
number of valid `(S, T)` pairs in the game, which is small and known at compile
time.

**Conclusion**: Option A works, but the `PhantomData<(S, T)>` on the killer
type `S` is purely phantom — `S` carries zero data. This is the key observation
that motivates Option C.

---

### Option B — Enum-based `KillRequest` / `EntityKind` discriminant

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind { Bolt, Cell, Wall, Breaker }

#[derive(Message, Clone, Debug)]
pub struct KillRequest {
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
    pub victim: Entity,
    pub victim_kind: EntityKind,
}

#[derive(Message, Clone, Debug)]
pub struct EntityDestroyed {
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
    pub victim: Entity,
    pub victim_kind: EntityKind,
    pub killer_pos: Option<Vec2>,
    pub victim_pos: Vec2,
}
```

**Registration**: Two calls total — `app.add_message::<KillRequest>()` and
`app.add_message::<EntityDestroyed>()`. Minimal.

**Domain receive**: All domains receive the same `KillRequest` queue and filter:

```rust
fn handle_kill_wall(
    mut reader: MessageReader<KillRequest>,
    ...
) {
    for msg in reader.read() {
        if msg.victim_kind != EntityKind::Wall { continue; }
        ...
    }
}
```

**The "read ALL Destroyed" problem**: One `MessageReader<EntityDestroyed>` in
`bridge_death`. Trivially solved.

**Critical flaw**: `MessageReader<T>` is a FIFO consumer. When the wall domain
and the bolt domain both `read()` from the same `KillRequest` queue,
**whichever system runs first drains the queue** and the second receives
nothing. This is not theoretical — it is the documented behavior of Bevy's
message system. The wall domain would filter and discard bolt-victim messages,
but those bolt-victim messages are now gone from the queue before the bolt
domain can read them.

This flaw is fundamental and cannot be fixed within the message abstraction
without duplicating the queue (which defeats the purpose) or serializing all
domains through a single dispatcher system (which reintroduces coupling).

**Conclusion**: Option B is eliminated by Bevy's message consumption model.

---

### Option C — Victim-only generic `KillYourself<T>` / `Destroyed<T>`

```rust
#[derive(Message, Clone, Debug)]
pub struct KillYourself<T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    _marker: PhantomData<T>,
}

#[derive(Message, Clone, Debug)]
pub struct Destroyed<T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    pub killer_pos: Option<Vec2>,
    pub victim_pos: Vec2,
    _marker: PhantomData<T>,
}
```

**Registration**: One call per victim type — 4 total for the 4 entity domains:

```rust
app.add_message::<KillYourself<Cell>>();
app.add_message::<KillYourself<Bolt>>();
app.add_message::<KillYourself<Wall>>();
app.add_message::<KillYourself<Breaker>>();
// Same for Destroyed<T>
```

**Domain receive**: Each domain receives exactly one message type, accepting any
killer:

```rust
fn handle_kill_wall(
    mut reader: MessageReader<KillYourself<Wall>>,
    walls: Query<(&Position2D, Option<&Invulnerable>), With<Wall>>,
    mut writer: MessageWriter<Destroyed<Wall>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let Ok((pos, invuln)) = walls.get(msg.victim) else { continue };
        if invuln.is_some() { continue; }
        writer.write(Destroyed { victim: msg.victim, victim_pos: pos.0, ..default() });
        commands.entity(msg.victim).despawn();
    }
}
```

One system, one reader, one writer. No killer-type filtering required in the
domain system.

**The "read ALL Destroyed" problem**: The effect trigger bridge needs N separate
readers — one per victim type:

```rust
fn bridge_death(
    mut cell_destroyed: MessageReader<Destroyed<Cell>>,
    mut bolt_destroyed: MessageReader<Destroyed<Bolt>>,
    mut wall_destroyed: MessageReader<Destroyed<Wall>>,
    mut breaker_destroyed: MessageReader<Destroyed<Breaker>>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
```

This is exactly what the codebase already does. `bridge_death` today has two
readers (`RequestCellDestroyed` + `RequestBoltDestroyed`). Option C grows that
to 4 readers as all entity types gain death support — a clean, linear expansion
that matches the existing shape of the code.

**The `DeathTarget` discriminant**: When firing `Trigger::Death(DeathTarget)`,
the bridge knows which reader fired:

```rust
    for msg in cell_destroyed.read() {
        let context = TriggerContext { cell: Some(msg.victim), ..default() };
        fire_global(&Trigger::Death(DeathTarget::Cell), context, &mut query, &mut commands);
    }
    for msg in bolt_destroyed.read() {
        let context = TriggerContext { bolt: Some(msg.victim), ..default() };
        fire_global(&Trigger::Death(DeathTarget::Bolt), context, &mut query, &mut commands);
    }
```

`DeathTarget::Any` can be handled as the final fallback in `evaluate_bound_effects`
(if `Death(Any)` matches all `Death(_)` variants).

**Conclusion**: Option C is strictly better than Option A. The killer marker `S`
in `KillYourself<S, T>` carries no data — it is phantom information that can be
inferred from the victim type's domain handler or from `msg.killer: Option<Entity>`.
Removing `S` halves the number of message registrations and eliminates the
multi-reader problem in domain systems.

---

### Option D — Trait-based with concrete structs per domain

```rust
trait DeathRequest { fn victim(&self) -> Entity; }

struct KillCell { pub killer: Option<Entity>, pub victim: Entity }
struct KillBolt { pub killer: Option<Entity>, pub victim: Entity }
struct KillWall { pub killer: Option<Entity>, pub victim: Entity }
```

This is what the codebase already does with `RequestCellDestroyed` and
`RequestBoltDestroyed`. It has no shared abstraction, so adding a third entity
type means adding a third concrete struct with no code reuse.

A trait adds nothing here because Bevy's message system is monomorphized — you
cannot have `MessageReader<dyn DeathRequest>`. The trait would be purely
nominal and would never be called polymorphically.

**Conclusion**: Option D is the status quo. It is being replaced precisely
because it scales linearly with new entity types and has no shared structure.

---

## Recommendation

**Use Option C: `KillYourself<T>` and `Destroyed<T>` generic only on victim
type.**

```rust
/// In a new `shared/messages.rs` or `shared/death.rs`

/// Request for a domain to destroy one of its entities.
///
/// `T` is the victim marker type (e.g., `Cell`, `Bolt`, `Wall`, `Breaker`).
/// `killer` is the entity that caused the death — `None` for self-inflicted
/// (timer expiry, `Die` effect with no causer).
#[derive(Message, Clone, Debug)]
pub struct KillYourself<T: 'static + Send + Sync + Clone> {
    /// Entity that caused this death, if any.
    pub killer: Option<Entity>,
    /// Entity to be destroyed.
    pub victim: Entity,
    _marker: PhantomData<T>,
}

/// Broadcast that an entity was destroyed.
///
/// `T` is the victim marker type. Consumed by effect triggers and any
/// system that needs to react to entity death.
#[derive(Message, Clone, Debug)]
pub struct Destroyed<T: 'static + Send + Sync + Clone> {
    /// Entity that caused this death, if any.
    pub killer: Option<Entity>,
    /// Entity that was destroyed.
    pub victim: Entity,
    /// World position of the killer at destruction time, if any.
    pub killer_pos: Option<Vec2>,
    /// World position of the victim at destruction time.
    pub victim_pos: Vec2,
    _marker: PhantomData<T>,
}

impl<T: 'static + Send + Sync + Clone> KillYourself<T> {
    pub fn new(victim: Entity, killer: Option<Entity>) -> Self {
        Self { killer, victim, _marker: PhantomData }
    }
}

impl<T: 'static + Send + Sync + Clone> Destroyed<T> {
    pub fn new(
        victim: Entity,
        victim_pos: Vec2,
        killer: Option<Entity>,
        killer_pos: Option<Vec2>,
    ) -> Self {
        Self { killer, victim, killer_pos, victim_pos, _marker: PhantomData }
    }
}
```

---

## Rationale

### Why victim-only generic, not fully generic on killer too

The `S` (killer) type in `KillYourself<S, T>` is a `PhantomData` marker — the
killer entity's identity is already carried by `killer: Option<Entity>`. The
marker type `S` adds no information that the receiving domain system can act on:
the wall domain's `handle_kill_wall` does not behave differently based on
whether a `Bolt` or a `Shockwave` sent the kill request. It checks invulnerability
and dispatches regardless.

Removing `S` eliminates:
- N×M message registrations in the plugin (N victims × M killers)
- Multiple readers per domain system ("wall domain reads `KillYourself<Bolt, Wall>`
  AND `KillYourself<(), Wall>`")
- The need to define marker types for non-entity killers like `Shockwave`,
  `TimerExpiry`, or future `EffectKind` variants

The killer's identity is preserved in full fidelity as `Option<Entity>` — any
system that needs to know what killed an entity (e.g., the future Killed trigger
for damage attribution) can query the killer entity directly.

### Why not enum-based (Option B)

Bevy's `MessageReader<T>` is a consuming reader. Multiple systems reading from
the same queue type will only receive messages that haven't already been
consumed. With a single `KillRequest` queue, whichever domain system runs first
drains all messages including ones destined for other domains. This is
not a performance concern — it is a correctness failure. Option B is
architecturally incompatible with Bevy's message model.

### How it fits existing codebase patterns

The current `bridge_death` system in
`breaker-game/src/effect/triggers/death.rs:18` already reads two separate
`MessageReader` parameters (`RequestCellDestroyed` + `RequestBoltDestroyed`) and
dispatches the same `Trigger::Death` with different `TriggerContext` per reader.
Option C extends this existing shape to 4 readers as entity types are added —
the code structure does not change, only the number of reader parameters grows.

The existing domain handler pattern (`cleanup_cell` in
`breaker-game/src/cells/systems/cleanup_cell.rs`) already reads one message
type, emits one downstream message, and despawns — this is exactly what
`handle_kill_<T>` will look like.

### Performance

Zero additional allocation. `PhantomData<T>` is zero-sized. The message
structs are identical in memory layout regardless of the type parameter. Bevy's
monomorphization produces separate type registrations but identical machine code
for each instantiation.

The N separate `MessageReader` parameters in `bridge_death` add N separate queue
polls per frame, but all queues are empty on most frames (death events are rare).
This is not a hot path.

---

## Handling `was_required_to_clear`

The current `RequestCellDestroyed.was_required_to_clear` field is cell-domain
data that `bridge_death` and `cleanup_cell` need. It cannot move into the
generic `Destroyed<Cell>` struct without making `Destroyed` non-generic or
adding a second type parameter for domain-specific payloads.

**Recommended approach**: retain the field as a cell-domain-specific lookup
rather than a message payload:

- `RequiredToClear` as a component on cell entities (already present or easily
  added)
- `cleanup_cell` reads it from the entity before despawn and emits it via
  the existing `CellDestroyedAt` message (which drives node completion tracking
  in `track_node_completion`)
- `KillYourself<Cell>` / `Destroyed<Cell>` stay generic — they carry
  `victim: Entity`, and downstream systems query the entity while it still
  exists

Alternatively, `CellDestroyedAt` can remain as a cell-domain-only downstream
broadcast from `handle_kill_cell`. This preserves the two-phase destruction
pattern (generic kill request → domain cleanup → domain-specific notification)
and does not require any current consumers of `CellDestroyedAt` to change.

The TODO note says `was_required_to_clear` "can move to a cell-domain component
or be carried in a cell-specific wrapper around the generic message." The
component approach is cleaner — it keeps the generic messages free of domain
cargo.

---

## Alternatives Considered

- **Option A (fully generic `<S, T>`)**: Eliminated because the `S` killer marker
  is phantom information already available as `Option<Entity>`. Keeping it
  doubles the number of message registrations and requires domain systems to
  accept multiple readers per kill type. No benefit over Option C.

- **Option B (enum discriminant, single queue)**: Eliminated because Bevy's
  `MessageReader` is consuming. Domain systems competing for the same queue would
  silently drop messages.

- **Option D (concrete types per domain)**: Eliminated as the status quo being
  replaced. Adding a third entity type (Wall) requires a third concrete struct with
  no shared structure and a third ad-hoc registration.

- **Trait objects (`Box<dyn DeathMessage>`)**: Incompatible with Bevy's message
  system, which requires `T: 'static + Send + Sync`. The type erasure would need
  to happen above the message layer, reintroducing the single-queue consumption
  problem.

---

## Codebase Precedent

- `breaker-game/src/effect/triggers/death.rs:18` — `bridge_death` already uses
  two `MessageReader` parameters for two death source types. Option C grows this
  to 4 readers — same shape, linear growth.

- `breaker-game/src/effect/triggers/impact/system.rs:23` — multiple named
  bridge functions (`bridge_impact_bolt_cell`, `bridge_impact_bolt_wall`, etc.),
  one per collision message type. This is the established pattern for "N sources
  → same trigger action": separate systems per message type, not a unified
  dispatcher. Option C follows this pattern.

- `breaker-game/src/cells/systems/cleanup_cell.rs:11` — current cell kill
  handler: reads one message, emits one downstream message, despawns entity.
  `handle_kill_cell` in the new design is structurally identical.

- `breaker-game/src/shared/components.rs` — zero-sized marker components
  (`CleanupOnNodeExit`, `CleanupOnRunEnd`) used as type-level discriminants.
  `Bolt`, `Cell`, `Wall`, `Breaker` are existing marker components that serve
  as the `T` parameter — no new marker types needed.

- `docs/architecture/standards.md` — "No over-engineering: No abstractions,
  generics, or indirection until there's a concrete second use case." The second
  use case exists here: wall death (new) joins cell death and bolt death. Three
  identical death patterns with no shared structure is the over-engineering
  being avoided.

---

## Registration Boilerplate Estimate

Option C requires 8 registration calls total (4 victim types × 2 message types):

```rust
// In the plugin that owns shared death messaging:
app.add_message::<KillYourself<Cell>>();
app.add_message::<KillYourself<Bolt>>();
app.add_message::<KillYourself<Wall>>();
app.add_message::<KillYourself<Breaker>>();
app.add_message::<Destroyed<Cell>>();
app.add_message::<Destroyed<Bolt>>();
app.add_message::<Destroyed<Wall>>();
app.add_message::<Destroyed<Breaker>>();
```

Compare to status quo: 4 registrations for 2 concrete types each with bespoke
names. Net increase: 4 calls, which also replace 4 separate struct definitions.

---

## Summary Table

| Option | Domain receiver | Bridge readers | Registration | Shared abstraction | Bevy-compatible |
|--------|----------------|---------------|-------------|-------------------|-----------------|
| A: `<S,T>` | multiple readers per domain | N×M readers | N×M calls | yes | yes |
| B: enum | one shared queue | 1 reader | 2 calls | yes | **NO** |
| C: `<T>` victim-only | 1 reader per domain | N readers | 2N calls | yes | yes |
| D: concrete | 1 reader per domain | N readers | N pairs | no | yes |

**Option C is the recommendation.** It is compatible with Bevy's consumption
model, keeps domain systems simple (one reader, one writer), produces the same
`bridge_death` shape the codebase already uses, and avoids the phantom killer
type that Option A carries without benefit.
