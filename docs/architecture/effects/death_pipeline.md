# Death Pipeline

Unified damage → death → trigger chain. All damage flows through typed messages, kill attribution propagates through effect chains, and death triggers fire on the correct entities.

## The Chain

```
DamageDealt<T> → apply_damage<T> (sets KilledBy on killing blow)
  → detect_deaths (N specialized per-domain systems, not one generic system)
  → KillYourself<T> { victim, killer: Option<Entity> }
  → domain handler (invuln check, animation)
  → Destroyed<T> { victim, killer, positions }
  → bridge_destroyed<T: GameEntity> (fires Died, Killed, DeathOccurred)
  → DespawnEntity message (deferred to PostFixedUpdate)
```

## Messages

```rust
/// Generic damage message — one Bevy message queue per victim type T.
struct DamageDealt<T: GameEntity> {
    dealer: Option<Entity>,     // who caused this damage (propagated through chains)
    target: Entity,             // who takes the damage
    amount: f32,                // damage amount
}

/// Domain kill request — generic on victim type T (S generic removed; killer is Option<Entity>).
struct KillYourself<T: GameEntity> {
    victim: Entity,
    killer: Option<Entity>,
}

/// Death notification — sent after domain handler validates (S generic removed; killer is Option<Entity>).
struct Destroyed<T: GameEntity> {
    victim: Entity,
    killer: Option<Entity>,
    victim_pos: Vec2,
    killer_pos: Option<Vec2>,
}

/// Deferred despawn request — processed in PostFixedUpdate.
struct DespawnEntity { entity: Entity }
```

## apply_damage System

Processes `DamageDealt<T>` messages. Decrements HP. Sets `KilledBy` **only on the killing blow** — the hit that crosses HP from positive to zero.

```rust
#[derive(Component, Default)]
struct KilledBy { dealer: Option<Entity> }
```

Multi-source same frame: message processing order determines the killing blow. Deterministic (system ordering + message queue order).

## Domain Handlers

Each T has a domain handler that sits between `KillYourself<T>` and `Destroyed<T>`:

| T (victim) | Handler | Special behavior |
|---|---|---|
| `Cell` | cells domain | Check invulnerability (guarded cells with active guardians); chain reaction — bolt attribution propagates via killer |
| `Wall` | wall domain | One-shot walls, timer expiry |
| `Bolt` | bolt domain | Environmental death (killer: None) — Killed trigger skipped |

All handlers follow: receive KillYourself → validate → send Destroyed → send DespawnEntity message. Entity MUST survive through `bridge_destroyed` trigger evaluation — despawn happens in PostFixedUpdate.

**Note:** detect_deaths are N specialized per-domain systems (one per victim type), not one generic system.

**Note:** Despawn uses the `DespawnEntity` message pattern, not a `PendingDespawn` component.

**Note:** apply_damage for cells skips Locked entities; must order after check_lock_release.

**Note:** was_required_to_clear is queried from the still-alive entity, not on Destroyed<T>.

## bridge_destroyed System

Generic on T: GameEntity. Reads `Destroyed<T>` and fires death triggers:

1. **Died** — fires on VICTIM entity only
2. **Killed(KillTarget)** — fires on KILLER entity only (skip if killer is None or despawned)
3. **DeathOccurred(DeathTarget)** — fires on ALL entities with BoundEffects

## Kill Attribution

The dealer entity propagates from the original damage source through the entire chain:

- Bolt hits cell → `dealer: Some(bolt)`
- Bolt's shockwave kills cell → `dealer: Some(bolt)` (shockwave inherits from spawning bolt)
- Bolt's chain lightning kills cell → `dealer: Some(bolt)` (arc inherits from source)
- Powder keg: bolt B kills cell → cell explodes → `dealer: Some(bolt_B)` (from DeathContext.killer)
- Environmental/timer death → `dealer: None` → Killed doesn't fire

Effects that deal damage read `TriggerContext` to propagate the dealer when constructing `DamageDealt` messages.

## DespawnEntity

Centralized despawn message. All entity despawns go through this instead of direct `.despawn()` calls.

```rust
fn process_despawn_requests(mut reader: MessageReader<DespawnEntity>, mut commands: Commands) {
    for msg in reader.read() {
        commands.entity(msg.entity).try_despawn();
    }
}
```

Runs in **PostFixedUpdate** — after all FixedUpdate systems have had a chance to read the entity. Uses `try_despawn` for graceful handling of already-cleaned-up entities.
