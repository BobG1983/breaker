---
name: Generic death message pattern — enum-based single queue (revised)
description: KillRequest/EntityDestroyed with EntityKind enum; fully-generic <S,T> and victim-only <T> both rejected; MessageReader is NOT consuming
type: project
---

For the unified death messaging system:

**Use enum-based single queue**: `KillRequest` / `EntityDestroyed` with `EntityKind`
discriminant. One message type per phase, not a generic struct.

**MessageReader is NOT consuming.** Each `MessageReader<T>` has an independent
cursor. Multiple systems reading the same message type in the same frame each see all
messages. PROOF: `RequestBoltDestroyed` is read independently by both `bridge_death`
(effect triggers) and `cleanup_destroyed_bolts` (bolt domain) in the same
`FixedUpdate` — both work correctly. The previous research was wrong to eliminate
enum-based queues on this basis.

**Registration**: 2 calls total — `add_message::<KillRequest>()` +
`add_message::<EntityDestroyed>()`.

**Domain handlers**: Read `KillRequest`, filter on `msg.victim_kind`, emit
`EntityDestroyed`. One system per domain, one reader.

**Bridge**: `bridge_killed` reads one `MessageReader<EntityDestroyed>`. Populates
`TriggerContext` from `msg.killer_kind` (set by sender when known, fallback to
component-marker query when `None`).

**Why NOT victim-only `<T>`**: `bridge_killed` needs to know which TriggerContext
slot the killer goes in. With `Destroyed<T>`, killer is `Option<Entity>` with no
type information — requires component query on killer entity at bridge time. The
enum gives same result with the discriminant set at source (better attribution).

**Why NOT fully generic `<S,T>`**: S is still PhantomData for effect-damage kills
(S=() until source_entity plumbing is complete), requires N×M registrations and
N×M bridge readers, forces domain handlers to split. No benefit over enum for
current mechanics.

**Killer resolution fallback** (for effect-damage path before source_entity threading):
```rust
// In bridge_killed — O(1) per entity, rarely reached when killer_kind is set
let resolved_kind = msg.killer_kind.or_else(|| {
    let k = msg.killer?;
    if bolt_check.get(k).is_ok() { return Some(EntityKind::Bolt); }
    // ... etc
    None
});
```

**Precedent files**: 
- `effect/triggers/death.rs:18` — existing 2-reader bridge_death; Option B collapses to 1
- `bolt/systems/cleanup_destroyed_bolts.rs:14` AND `effect/triggers/death.rs:19` —
  BOTH read `RequestBoltDestroyed` independently in same frame (proof of non-consuming)

**How to apply**: When implementing the kill/die pattern, use `KillRequest` +
`EntityDestroyed` with `EntityKind`. Do NOT parameterize on killer type. Do NOT
assume MessageReader is consuming — multiple systems can read the same message queue.
