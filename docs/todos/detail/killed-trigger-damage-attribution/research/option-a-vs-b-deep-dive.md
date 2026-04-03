# Option A vs B Deep-Dive: Generic Death Messages for TriggerContext Population

## Context

Previous research recommended `KillYourself<T>` / `Destroyed<T>` with victim-only
type parameter (Option C). That was rejected because it loses killer type
information needed to populate `TriggerContext` in `bridge_killed`.

Two options remain:
- **Option A**: Fully generic `Destroyed<S, T>` where `S` = killer type, `T` = victim type
- **Option B (revisited)**: Enum-based single queue, or a variant that fixes the consuming reader problem

This document resolves both.

---

## Critical Finding: MessageReader is NOT Consuming

The previous research incorrectly stated that Bevy's `MessageReader<T>` drains the
queue for all systems. **This is wrong.** The Bevy 0.18 docs for `MessageReader`
state:

> "Systems with `MessageReader<T>` param can be executed concurrently (but not
> concurrently with `MessageWriter<T>` or `MessageMutator<T>` systems for the same
> message type)."

Each `MessageReader` maintains its own `MessageCursor`. Calling `read()` advances
**that reader's** cursor — it does not remove messages from the underlying buffer.
Other `MessageReader<T>` instances in other systems see the same messages
independently.

**Proof in this codebase**: `RequestBoltDestroyed` has TWO readers in the same frame:
- `breaker-game/src/effect/triggers/death.rs:19` — `bridge_death` reads it for effect evaluation
- `breaker-game/src/bolt/systems/cleanup_destroyed_bolts.rs:15` — reads it to despawn the entity

Both run in the same `FixedUpdate` schedule. `cleanup_destroyed_bolts` runs
`.after(EffectSystems::Bridge)` (see `bolt/plugin.rs:82`). The bolt is successfully
despawned AND the death trigger fires. If reading were consuming, one of them would
never see the message. Both see it.

**Consequence**: Option B's "critical flaw" from the previous research was wrong.
The single-queue enum approach is Bevy-compatible. This reopens Option B.

---

## Option A: Fully Generic `Destroyed<S, T>`

### Structure

```rust
#[derive(Message, Clone, Debug)]
pub struct KillYourself<S: 'static + Send + Sync, T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    _marker: PhantomData<(S, T)>,
}

#[derive(Message, Clone, Debug)]
pub struct Destroyed<S: 'static + Send + Sync, T: 'static + Send + Sync> {
    pub killer: Option<Entity>,
    pub victim: Entity,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
    _marker: PhantomData<(S, T)>,
}
```

### TriggerContext Population (why it was requested)

When `bridge_killed` reads `Destroyed<Bolt, Cell>`, it knows at compile time:
- `S = Bolt` → `context.bolt = Some(msg.killer)`
- `T = Cell` → `context.cell = Some(msg.victim)`

This enables `On(Bolt)` in a `Killed` chain to resolve to the killer bolt
without a runtime component query.

```rust
fn bridge_killed_bolt_cell(
    mut reader: MessageReader<Destroyed<Bolt, Cell>>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.victim) {
            let context = TriggerContext {
                cell: Some(msg.victim),
                bolt: msg.killer,
                ..default()
            };
            evaluate_bound_effects(&Trigger::Killed, entity, bound, &mut staged, &mut commands, context);
            evaluate_staged_effects(&Trigger::Killed, entity, &mut staged, &mut commands, context);
        }
    }
}
```

### Valid (S, T) Pairs — Exhaustive Enumeration

The `S` type represents the **proximate cause of death**, not the originating entity.
The proximate cause is always an entity that directly sends `KillYourself<_,T>`.

Current damage senders:
1. `bolt_cell_collision` → bolt hits cell → `S=Bolt, T=Cell`
2. Shockwave `apply_shockwave_damage` → sends `DamageCell` → cells domain reads → `S=?`
3. Explode `process_explode_requests` → sends `DamageCell` → `S=?`
4. ChainLightning arc arrival → sends `DamageCell` → `S=?`
5. PiercingBeam → sends `DamageCell` → `S=?`
6. TetherBeam tick → sends `DamageCell` → `S=?`
7. Pulse (shockwave at bolt) → sends `DamageCell` → `S=?`
8. `breaker_cell_collision` → breaker hits cell → `S=Breaker, T=Cell`
9. Timer-expiry walls → timed wall dies → `S=()` (no killer entity), `T=Wall`
10. Bolt lost → `S=()`, `T=Bolt`

**The effect damage problem**: Shockwave, Explode, ChainLightning, etc. all funnel
through `DamageCell` then `handle_cell_hit`. `handle_cell_hit` sends
`RequestCellDestroyed` (future: `KillYourself<Cell>`). The damage-source identity is
preserved only as `DamageCell.source_chip: Option<String>` — there is no
`source_entity` field yet (adding it is Part 2 of the feature).

Until `source_entity` is added to `DamageCell` and `LastDamageSource` is tracked,
`handle_kill_cell` cannot know the killer entity for effect damage. This means `S`
for effect-sourced kills is effectively `()` regardless — the type parameter buys
nothing in those cases until a separate plumbing change is done.

**Practical valid pairs today** (with killer entity available):
| S | T | Sender |
|---|---|--------|
| `Bolt` | `Cell` | `bolt_cell_collision` |
| `Breaker` | `Cell` | `breaker_cell_collision` |
| `()` | `Cell` | Any effect (Shockwave/Explode/etc.), timer expiry |
| `()` | `Bolt` | `bolt_lost` (falls off screen), `tick_bolt_lifespan` (lifespan expiry) |
| `()` | `Wall` | Timer-expiry (timed walls, future feature) |
| `Bolt` | `Wall` | Hypothetical: bolt destroys a wall (not a current mechanic) |

**S is never `Shockwave`, `ChainLightning`, etc.** Those are not entities that
domains understand. The killer is always the entity that sent `DamageCell`, which
for effects is either a bolt (if we thread the context through) or unknown (`()`).

### Domain System Complexity

The cells domain currently has ONE kill path. With `<S, T>`:

```rust
// cells/systems/handle_kill_cell.rs

fn handle_kill_cell_by_bolt(mut reader: MessageReader<KillYourself<Bolt, Cell>>, ...)
fn handle_kill_cell_by_breaker(mut reader: MessageReader<KillYourself<Breaker, Cell>>, ...)
fn handle_kill_cell_unknown(mut reader: MessageReader<KillYourself<(), Cell>>, ...)
```

OR: one system with N readers:

```rust
fn handle_kill_cell(
    mut bolt_kills: MessageReader<KillYourself<Bolt, Cell>>,
    mut breaker_kills: MessageReader<KillYourself<Breaker, Cell>>,
    mut effect_kills: MessageReader<KillYourself<(), Cell>>,
    ...
)
```

The body of each/all of these is identical — check invuln/shield, emit `Destroyed`,
track `LastDamageSource`. The only difference is which `Destroyed<S, Cell>` type
to emit back.

This creates a second N-reader problem downstream: `bridge_killed` needs a reader
per `(S, T)` pair:

```rust
fn bridge_killed(
    mut bolt_cell: MessageReader<Destroyed<Bolt, Cell>>,
    mut breaker_cell: MessageReader<Destroyed<Breaker, Cell>>,
    mut effect_cell: MessageReader<Destroyed<(), Cell>>,
    mut bolt_bolt: MessageReader<Destroyed<(), Bolt>>,   // bolt lost — killer is ()
    mut effect_wall: MessageReader<Destroyed<(), Wall>>,
    ...
)
```

As new effect types gain `source_entity` threading, new pairs emerge. Each new pair
requires a new reader in both `handle_kill_cell` and `bridge_killed`.

### The PhantomData Problem Revisited

`S` in `KillYourself<S, T>` is still `PhantomData`. The killer entity is still
`killer: Option<Entity>`. Option A is claimed to enable better `TriggerContext`
population, but look at what it actually provides:

- With `Destroyed<Bolt, Cell>`: killer slot = `context.bolt = msg.killer`
- With `Destroyed<(), Cell>`: killer slot = nothing (killer is `()`)

For the majority of kills today (effect-sourced), `S = ()` and `context.bolt =
None` regardless. The type parameter only helps for the two direct-contact cases
(bolt→cell, breaker→cell).

The same result can be achieved at runtime by querying `world.get::<Bolt>(killer)`:
if the killer entity has the `Bolt` marker, it's a bolt. This is a single
`Query<(), With<Bolt>>` existence check — `O(1)`, not a scan.

---

## Option B (Revisited): Enum-Based Single Queue

### Why It Is Now Valid

`MessageReader` is not consuming (proven above). Multiple domain systems can each
read the same `KillRequest` queue independently and filter for their victim type.

```rust
#[derive(Message, Clone, Debug)]
pub struct KillRequest {
    pub victim: Entity,
    pub victim_kind: EntityKind,
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
}

#[derive(Message, Clone, Debug)]
pub struct EntityDestroyed {
    pub victim: Entity,
    pub victim_kind: EntityKind,
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind { Bolt, Cell, Wall, Breaker }
```

### Registration

Two calls total:
```rust
app.add_message::<KillRequest>();
app.add_message::<EntityDestroyed>();
```

### Domain receivers

```rust
fn handle_kill_cell(
    mut reader: MessageReader<KillRequest>,
    cells: Query<(&Position2D, Option<&Invulnerable>), With<Cell>>,
    mut writer: MessageWriter<EntityDestroyed>,
) {
    for msg in reader.read() {
        if msg.victim_kind != EntityKind::Cell { continue; }
        let Ok((pos, invuln)) = cells.get(msg.victim) else { continue };
        if invuln.is_some() { continue; }
        writer.write(EntityDestroyed { victim: msg.victim, victim_kind: EntityKind::Cell, ... });
    }
}
```

Each domain system reads the full queue and skips non-matching victim_kind entries.

### Bridge

```rust
fn bridge_killed(
    mut reader: MessageReader<EntityDestroyed>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let context = TriggerContext {
            bolt:    if msg.killer_kind == Some(EntityKind::Bolt)    { msg.killer } else { None },
            cell:    if msg.killer_kind == Some(EntityKind::Cell)    { msg.killer } else { None },
            wall:    if msg.killer_kind == Some(EntityKind::Wall)    { msg.killer } else { None },
            breaker: if msg.killer_kind == Some(EntityKind::Breaker) { msg.killer } else { None },
        };
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.victim) {
            evaluate_bound_effects(&Trigger::Killed, entity, bound, &mut staged, &mut commands, context);
            evaluate_staged_effects(&Trigger::Killed, entity, &mut staged, &mut commands, context);
        }
    }
}
```

One reader, one bridge system. `TriggerContext` is populated from `killer_kind`
(runtime discriminant) instead of from a type parameter (compile-time discriminant).

### Concerns with Option B

**1. Skip-scan waste**: Every domain system skans the full queue per tick. With 4
domains and N kill events per frame, each system reads O(N) messages and discards
all but its slice. In practice kill events are rare (a few per frame peak), so
this is not a hot path. The waste is negligible.

**2. `EntityKind` must stay synchronized with actual entity types**: Adding a new
killable entity type requires adding a variant to `EntityKind`. This is a single
enum in a shared module — less scattered than adding a new pair to every generic
instantiation site.

**3. No compile-time exhaustiveness**: If a new `EntityKind::Anchor` is added and
`bridge_killed` forgets to handle it, no compiler error. With Option A, adding
`Destroyed<Anchor, Cell>` requires a new reader in `bridge_killed` or it silently
goes unread (also no compiler error — you just miss the messages).

**4. `EntityKind` vs component query for killer identification**: The `killer_kind`
field must be set by whoever writes `KillRequest`. In the direct-hit case
(`bolt_cell_collision`), the sender knows `S = Bolt`. In the effect-damage case
(`handle_kill_cell` reading `DamageCell`), it knows only `source_entity`. To know
if `source_entity` is a Bolt, it still needs a query — same as with Option A's
runtime fallback. But with Option B, the enum discriminant CAN be set by the
original sender (`bolt_cell_collision` passes `killer_kind: Some(EntityKind::Bolt)`
in the `KillRequest`) rather than inferred later. This is actually **better
attribution** because the knowledge is available at the source.

---

## Direct Comparison

| Concern | Option A `<S,T>` | Option B enum |
|---------|-----------------|---------------|
| Registration | N×M calls per valid pair | 2 calls |
| Domain handler complexity | N readers OR N systems per domain | 1 reader, filter by kind |
| Bridge complexity | N×M readers in bridge_killed | 1 reader |
| TriggerContext population | Compile-time via type param | Runtime via enum discriminant |
| Correctness guarantee | PhantomData marker for S | `killer_kind` field |
| Effect-damage attribution | S=() until source_entity plumbing done | killer_kind can be set at source |
| New entity type cost | New pair at every callsite | New enum variant in one place |
| Bevy compatibility | Yes | Yes (reader is NOT consuming) |
| Alignment with existing code | Growth of existing `<S,T>`-free pattern | New pattern — no precedent |

---

## Recommendation: Option B (enum-based)

**Use `KillRequest` / `EntityDestroyed` with `EntityKind` discriminant.**

### Rationale

**1. The TriggerContext population problem is solved with less machinery.**
Option A promises compile-time type safety but delivers it only for the two
direct-contact pairs (`Bolt→Cell`, `Breaker→Cell`). For all effect-sourced kills,
`S = ()` and you get `context = default()` regardless. The compile-time slot
assignment only works when you already know the killer type at the callsite — and
when you know it, you can equally well set `killer_kind: Some(EntityKind::Bolt)`.
The enum gives identical runtime information.

**2. The "consuming reader" premise was wrong — Option B is Bevy-compatible.**
Multiple domain systems independently reading `KillRequest` works correctly.
`bridge_killed` reading `EntityDestroyed` works correctly. The flaw that eliminated
Option B in the previous research does not exist.

**3. Registration and call-site complexity is dramatically lower.**
Option A requires registering every valid `(S,T)` pair (currently 5-6, grows with
new mechanics), adding a reader to every bridge system for each pair, and splitting
or duplicating domain handlers. Option B requires 2 registrations forever.

**4. Adding a new killable type is one enum variant, not N new call sites.**
When walls become killable, Option B adds `EntityKind::Wall` and a new filter in
`handle_kill_wall`. Option A adds `KillYourself<(), Wall>`, `Destroyed<(), Wall>`,
potentially `Destroyed<Bolt, Wall>` (if bolts can destroy walls), a new reader in
every bridge system, and new registration calls in the plugin.

**5. Option B alignment with attribution needs.**
`killer_kind` is set by the sender at the source — `bolt_cell_collision` naturally
knows the killer is `EntityKind::Bolt`. Effect damage from `handle_kill_cell` can
set `killer_kind` based on `LastDamageSource` component (a Bolt entity →
`EntityKind::Bolt`). This is the same lookup Option A's runtime fallback requires,
but done once at kill-request time rather than per-bridge-system.

### Implementation Shape

```rust
// shared/death.rs (new shared module)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind { Bolt, Cell, Wall, Breaker }

#[derive(Message, Clone, Debug)]
pub struct KillRequest {
    pub victim: Entity,
    pub victim_kind: EntityKind,
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
}

#[derive(Message, Clone, Debug)]
pub struct EntityDestroyed {
    pub victim: Entity,
    pub victim_kind: EntityKind,
    pub killer: Option<Entity>,
    pub killer_kind: Option<EntityKind>,
    pub victim_pos: Vec2,
    pub killer_pos: Option<Vec2>,
}
```

```rust
// bolt_cell_collision — sets killer_kind at source
writer.write(KillRequest {
    victim: cell,
    victim_kind: EntityKind::Cell,
    killer: Some(bolt),
    killer_kind: Some(EntityKind::Bolt),
});
```

```rust
// handle_kill_cell — domain handler, one system, filters by kind
fn handle_kill_cell(
    mut reader: MessageReader<KillRequest>,
    cells: Query<(&Position2D, Option<&Invulnerable>, Option<&LastDamageSource>), With<Cell>>,
    mut writer: MessageWriter<EntityDestroyed>,
) {
    for msg in reader.read() {
        if msg.victim_kind != EntityKind::Cell { continue; }
        let Ok((pos, invuln, last_source)) = cells.get(msg.victim) else { continue };
        if invuln.is_some() { continue; }
        // Killer: prefer msg.killer (set by direct-hit senders), fallback to LastDamageSource
        let (killer, killer_kind) = msg.killer
            .map(|e| (Some(e), msg.killer_kind))
            .unwrap_or_else(|| {
                let e = last_source.and_then(|ls| ls.0);
                // killer_kind will be resolved by bridge_killed via component query
                (e, None)
            });
        writer.write(EntityDestroyed {
            victim: msg.victim,
            victim_kind: EntityKind::Cell,
            killer,
            killer_kind,
            victim_pos: pos.0,
            killer_pos: None,
        });
    }
}
```

```rust
// bridge_killed — single system, reads EntityDestroyed
fn bridge_killed(
    mut reader: MessageReader<EntityDestroyed>,
    bolt_check: Query<(), With<Bolt>>,
    cell_check: Query<(), With<Cell>>,
    wall_check: Query<(), With<Wall>>,
    breaker_check: Query<(), With<Breaker>>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        // Resolve killer_kind if not set by sender (fallback: query marker components)
        let resolved_killer_kind = msg.killer_kind.or_else(|| {
            let k = msg.killer?;
            if bolt_check.get(k).is_ok() { return Some(EntityKind::Bolt); }
            if cell_check.get(k).is_ok() { return Some(EntityKind::Cell); }
            if wall_check.get(k).is_ok() { return Some(EntityKind::Wall); }
            if breaker_check.get(k).is_ok() { return Some(EntityKind::Breaker); }
            None
        });
        let context = TriggerContext {
            bolt:    msg.killer.filter(|_| resolved_killer_kind == Some(EntityKind::Bolt)),
            cell:    msg.killer.filter(|_| resolved_killer_kind == Some(EntityKind::Cell)),
            wall:    msg.killer.filter(|_| resolved_killer_kind == Some(EntityKind::Wall)),
            breaker: msg.killer.filter(|_| resolved_killer_kind == Some(EntityKind::Breaker)),
        };
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.victim) {
            evaluate_bound_effects(&Trigger::Killed, entity, bound, &mut staged, &mut commands, context);
            evaluate_staged_effects(&Trigger::Killed, entity, &mut staged, &mut commands, context);
        }
    }
}
```

The component-marker fallback in `bridge_killed` is an `O(1)` per-entity lookup.
It is only reached when `killer_kind` is `None` (effect-damage path, before full
source_entity threading is in place). Once `LastDamageSource` + `DamageCell.source_entity`
plumbing is complete, most kills will have `killer_kind` set at source and the
fallback rarely runs.

### Migration from Current Code

`RequestCellDestroyed` → `KillRequest { victim_kind: Cell, killer: None, killer_kind: None }`
(temporarily, until `bolt_cell_collision` sets the killer fields)

`CellDestroyedAt` → `EntityDestroyed { victim_kind: Cell, ... }`

`RequestBoltDestroyed` → `KillRequest { victim_kind: Bolt, killer: None, killer_kind: None }`

### Alignment with Existing Patterns

- `bridge_death` already reads TWO `MessageReader` params for two death message types.
  Option B collapses this to one `MessageReader<EntityDestroyed>` — actually simpler.
- The existing `bridge_impact_bolt_cell` pattern uses separate bridge functions per
  collision type. Option B is a departure from that pattern, but the reason for
  separate systems there is that collision messages carry different data. Death events
  all carry the same structure — a single `EntityDestroyed` type handles all of them.
- `breaker-game/src/cells/systems/handle_cell_hit/system.rs` already filters on
  cell identity (`cells.get(msg.cell)`). `handle_kill_cell` filtering on
  `msg.victim_kind` is the same pattern.

---

## Final Answer on Option A

Option A should be **rejected**. The type parameter `S` on the killer:

1. Only helps in 2 of ~6 current kill pairs. For all effect-sourced kills today,
   `S = ()` and context is empty regardless of the type param.
2. Multiplies registration calls and bridge readers by N.
3. Forces domain handlers to split or accept multiple readers per victim type.
4. Provides no stronger correctness guarantee than the enum discriminant — the
   compile-time slot assignment (`context.bolt = msg.killer` when `S=Bolt`) is
   equivalent to the runtime assignment (`context.bolt = msg.killer` when
   `killer_kind == Bolt`).

The "killer type loses information" concern that motivated this follow-up is real,
but Option B solves it with less structural complexity than Option A.

---

## Summary Table (Updated)

| Option | Valid? | Domain handler | Bridge | Registration | TriggerContext |
|--------|--------|---------------|--------|-------------|----------------|
| A: `<S,T>` fully generic | Yes | N readers per domain | N×M readers | N×M calls | Compile-time, but `()` for effects |
| B: enum `EntityKind` | **Yes** (reader not consuming) | 1 reader + filter | **1 reader** | **2 calls** | Runtime, same fidelity |
| C: `<T>` victim-only | Yes | 1 reader | N readers | 2N calls | Query fallback required |

**Recommendation: Option B.**

---

## Codebase Precedent

- `breaker-game/src/effect/triggers/death.rs:18-57` — `bridge_death` already has
  two `MessageReader` params. Option B replaces these with one.
- `breaker-game/src/bolt/systems/cleanup_destroyed_bolts.rs:14-24` AND
  `breaker-game/src/effect/triggers/death.rs:19` — BOTH read `RequestBoltDestroyed`
  in the same frame. Proof that multiple readers of the same message type work
  correctly (not consuming).
- `breaker-game/src/effect/triggers/impact/system.rs:23` — Impact bridge already
  uses per-message-type systems. Death unification moves away from this pattern
  for good reason: collision messages differ in structure; death messages are uniform.
- `breaker-game/src/cells/systems/handle_cell_hit/system.rs` — filters on specific
  cell entity. `handle_kill_cell` filtering on `victim_kind` is the same intent.
