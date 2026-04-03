# Killed Trigger, Die Effect, and Unified Death Messaging

## Summary
Unify all entity death into a single generic pattern: `Die` effect → `KillYourself<S, T>` message → domain decides → `Destroyed<S, T>` message → three triggers fire. Replace all bespoke death types.

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

Generic on both killer type (S) and victim type (T). Each (S, T) pair is a separate Bevy message queue. `MessageReader` is non-consuming — multiple systems can read the same queue independently.

## Effect

`EffectKind::Die` — fires on self. Sends `KillYourself<S, T>` where T = the entity's own marker type, S = killer type from TriggerContext.

## Triggers

```rust
enum KillTarget { Cell, Bolt, Wall, Breaker, Any }
enum DeathTarget { Cell, Bolt, Wall, Breaker, Any }
```

| Trigger | Fires on | Context | Purpose |
|---------|----------|---------|---------|
| `Killed(KillTarget)` | **Killer** entity | Both killer + victim | "I killed a Cell" — kill reward |
| `Died` | **Victim** entity | Both killer + victim | "I died" — cell revenge, mark-and-reward |
| `Death(DeathTarget)` | **All** entities globally | Both killer + victim | "A Cell died somewhere" — cascade, chain reaction |

## Death Chain

```
Do(Die) evaluates on entity
  → sends KillYourself<S, T> { victim: self, killer: from TriggerContext }

Domain kill handler receives KillYourself<S, T>
  → checks invuln/shield (can ignore — no Destroyed sent, no triggers fire)
  → sends Destroyed<S, T> { victim, killer, positions }
  → does NOT despawn yet

bridge_destroyed<S, T> receives Destroyed<S, T>
  → fires Killed(KillTarget) on KILLER entity (if killer exists)
  → fires Died on VICTIM entity
  → fires Death(DeathTarget) GLOBALLY on all entities with BoundEffects
  → (entity survives through all trigger evaluation + death animation)
  → despawn after
```

## Domain Kill Handlers

Each domain owns its kill handler, generic over S (killer type):

```rust
// cell/systems/handle_cell_kill.rs
fn handle_cell_kill<S: Component>(
    mut reader: MessageReader<KillYourself<S, Cell>>,
    cells: Query<(&Position2D, Option<&Invulnerable>, Option<&RequiredToClear>), With<Cell>>,
    mut writer: MessageWriter<Destroyed<S, Cell>>,
) {
    for msg in reader.read() {
        let Ok((pos, invuln, _required)) = cells.get(msg.victim) else { continue };
        if invuln.is_some() { continue; }
        writer.write(Destroyed { victim: msg.victim, killer: msg.killer, victim_pos: pos.0, .. });
    }
}

// wall/systems/handle_wall_kill.rs
fn handle_wall_kill<S: Component>(
    mut reader: MessageReader<KillYourself<S, Wall>>,
    walls: Query<(&Position2D, Option<&Invulnerable>), With<Wall>>,
    mut writer: MessageWriter<Destroyed<S, Wall>>,
) { ... }

// bolt/systems/handle_bolt_kill.rs  
fn handle_bolt_kill<S: Component>(...) { ... }
```

Registration — each domain plugin registers for its valid killer types:

```rust
// cell plugin
app.add_systems(FixedUpdate, handle_cell_kill::<Bolt>);
app.add_systems(FixedUpdate, handle_cell_kill::<Breaker>);
app.add_systems(FixedUpdate, handle_cell_kill::<Cell>);

// wall plugin
app.add_systems(FixedUpdate, handle_wall_kill::<Bolt>);

// bolt plugin
app.add_systems(FixedUpdate, handle_bolt_kill::<()>);
```

## TriggerContext Mapping

Trait maps (S, T) → TriggerContext at compile time:

```rust
trait DeathAttribution: Send + Sync + 'static {
    fn context(killer: Option<Entity>, victim: Entity) -> TriggerContext;
    fn kill_target() -> KillTarget;
    fn death_target() -> DeathTarget;
}

impl DeathAttribution for (Bolt, Cell) {
    fn context(killer: Option<Entity>, victim: Entity) -> TriggerContext {
        TriggerContext { bolt: killer, cell: Some(victim), ..default() }
    }
    fn kill_target() -> KillTarget { KillTarget::Cell }
    fn death_target() -> DeathTarget { DeathTarget::Cell }
}
```

## Bridge System

Generic, registered per (S, T) pair:

```rust
fn bridge_destroyed<S, T>(
    mut reader: MessageReader<Destroyed<S, T>>,
    mut query: Query<(Entity, &BoundEffects, &mut StagedEffects)>,
    mut commands: Commands,
) where S: Component, T: Component, (S, T): DeathAttribution {
    for msg in reader.read() {
        let context = <(S, T)>::context(msg.killer, msg.victim);
        let kill_target = <(S, T)>::kill_target();
        let death_target = <(S, T)>::death_target();

        // Killed — fires on KILLER: "I killed a Cell"
        if let Some(killer) = msg.killer {
            if let Ok((entity, bound, mut staged)) = query.get_mut(killer) {
                evaluate(&Trigger::Killed(kill_target), entity, bound, &mut staged, &mut commands, context);
            }
        }

        // Died — fires on VICTIM: "I died"
        if let Ok((entity, bound, mut staged)) = query.get_mut(msg.victim) {
            evaluate(&Trigger::Died, entity, bound, &mut staged, &mut commands, context);
        }

        // Death — fires GLOBALLY: "a Cell died somewhere"
        for (entity, bound, mut staged) in &mut query {
            evaluate(&Trigger::Death(death_target), entity, bound, &mut staged, &mut commands, context);
        }
    }
}

// Registration
app.add_systems(FixedUpdate, bridge_destroyed::<Bolt, Cell>);
app.add_systems(FixedUpdate, bridge_destroyed::<Breaker, Cell>);
app.add_systems(FixedUpdate, bridge_destroyed::<Cell, Cell>);
app.add_systems(FixedUpdate, bridge_destroyed::<Bolt, Wall>);
app.add_systems(FixedUpdate, bridge_destroyed::<(), Bolt>);
```

## RON Examples

```ron
// Kill reward: bolt chip — "every cell I kill boosts my speed"
On(target: Bolt, then: [
    When(trigger: Killed(Cell), then: [Do(SpeedBoost(multiplier: 1.3))])
])

// Generic kill reward: "every kill boosts my speed"
On(target: Bolt, then: [
    When(trigger: Killed(Any), then: [Do(SpeedBoost(multiplier: 1.1))])
])

// Cell revenge: volatile cell — "when I die, cripple whoever killed me"
On(target: Cell, then: [
    When(trigger: Died, then: [
        On(target: Bolt, then: [Do(SpeedBoost(multiplier: 0.3))])
    ])
])

// Cascade: "when any cell dies, shockwave from bolt"
On(target: Bolt, then: [
    When(trigger: Death(Cell), then: [Do(Shockwave(...))])
])

// One-shot wall: "when hit by bolt, kill myself"
On(target: Wall, then: [
    When(trigger: Impacted(Bolt), then: [Do(Die)])
])

// Global death reaction: "when anything dies, pulse"
On(target: Bolt, then: [
    When(trigger: Death(Any), then: [Do(Pulse(...))])
])
```

## RON Migration (~15 files)

```ron
// Before                              // After
When(trigger: CellDestroyed, ...)  →  When(trigger: Death(Cell), ...)
When(trigger: Death, ...)          →  When(trigger: Death(Any), ...)
```

Files: `cascade`, `chain_reaction`, `splinter`, `feedback_loop`, `entropy_engine`, `voltchain`, `gravity_well`, `supernova`, `split_decision`, `chain_reaction` (evolution), `breaker.example.ron`.

## Damage Attribution (source_entity plumbing)

```rust
// DamageCell — add source_entity field
struct DamageCell {
    pub cell: Entity,
    pub damage: f32,
    pub source_chip: Option<String>,
    pub source_entity: Option<Entity>,  // NEW
}

// Component on cells — updated on every DamageCell
struct LastDamageSource(pub Option<Entity>);
```

`bolt_cell_collision` sets `source_entity = Some(bolt)`. Effect damage (shockwave, explode, etc.) sets it from `TriggerContext.bolt`. `handle_cell_kill` reads `LastDamageSource` to get the killer entity for `KillYourself<S, Cell>`.

## Types Eliminated

| Before | After |
|--------|-------|
| `DamageCell` | `DamageDealt<Cell>` |
| `RequestCellDestroyed` | `KillYourself<S, Cell>` |
| `CellDestroyedAt` | `Destroyed<S, Cell>` |
| `RequestBoltDestroyed` | `KillYourself<(), Bolt>` |
| `Trigger::CellDestroyed` | `Trigger::Death(Cell)` |
| `Trigger::Death` (no discriminant) | `Trigger::Death(DeathTarget)` |
| `bridge_cell_destroyed` system | deleted — `bridge_destroyed::<S, Cell>` |
| `bridge_death` (hardcoded 2 readers) | deleted — N × `bridge_destroyed::<S, T>` |
| `bridge_died` (hardcoded 2 readers) | deleted — collapsed into `bridge_destroyed` |
| `effect/triggers/cell_destroyed.rs` | deleted |

## Adding a New (S, T) Pair

1. `impl DeathAttribution for (S, T)` — one trait impl
2. `app.add_message::<KillYourself<S, T>>()` — one registration
3. `app.add_message::<Destroyed<S, T>>()` — one registration
4. `app.add_systems(handle_victim_kill::<S>)` — one line in victim's plugin
5. `app.add_systems(bridge_destroyed::<S, T>)` — one line in effect plugin

## Valid (S, T) Pairs

| S | T | Source |
|---|---|--------|
| `Bolt` | `Cell` | Direct hit + bolt-sourced effects (shockwave, explode, etc.) |
| `Breaker` | `Cell` | Breaker-cell collision |
| `Cell` | `Cell` | Chain reaction |
| `Bolt` | `Wall` | One-shot wall hit by bolt |
| `()` | `Bolt` | Bolt lost, lifespan expiry |
| `()` | `Wall` | Timer expiry |

## Dependencies

- Depends on: TriggerContext infrastructure — DONE
- Depends on: Wall builder (introduces wall death)
- Blocks: Cell revenge, kill-reward chips, one-shot/timed walls

## Research

- [Generic message patterns](research/generic-message-patterns.md) — initial analysis (Option C recommended, later rejected)
- [Option A vs B deep-dive](research/option-a-vs-b-deep-dive.md) — MessageReader is non-consuming, both viable
- Final decision: Option A (`<S, T>` fully generic) with `DeathAttribution` trait, generic domain handlers, generic bridge

## RON Migration
See [ron-migration.md](ron-migration.md) — 17 RON files, mechanical find-replace.

## Chain Attribution (resolved)

Context threading through the synchronous `fire()` call stack only. Deferred effects (`StagedEffects`, `BoundEffects`) get context from whatever trigger activates them later — not from the original source.

### Change: add `context: TriggerContext` parameter

Thread context through the call stack:
1. `fire_effect(entity, effect, source_chip)` → `fire_effect(entity, effect, source_chip, context)`
2. `FireEffectCommand { entity, effect, source_chip }` → add `context` field
3. `EffectKind::fire(entity, source_chip, world)` → `fire(entity, source_chip, context, world)`
4. All ~30 per-effect `fire()` signatures gain `context: TriggerContext` parameter

### How damage-dealing effects use it

Effects that send `DamageCell` read the attribution from context:
```rust
// shockwave::fire(entity, ..., context, world)
writer.write(DamageCell {
    cell: nearby,
    damage: 10.0,
    source_chip: chip_name,
    source_entity: context.bolt,  // bolt that started the chain
});
```

Applies to: Shockwave, Explode, ChainLightning, PiercingBeam, TetherBeam, Pulse, RampingDamage.

### Why deferred effects don't need this

Deferred effects (`When(Died, Do(Explode))`) fire when a future trigger matches. That trigger creates its own `TriggerContext` from the `Destroyed<S, T>` message — which already carries the killer. So the attribution propagates across trigger boundaries naturally:

```
bolt hits cell → shockwave → kills nearby cell
  → Destroyed<Bolt, Cell> { killer: bolt }
  → bridge fires Died on nearby cell with context = { bolt: bolt }
  → nearby cell has When(Died, Do(Explode))
  → explode fires with context.bolt = bolt (from the Destroyed message)
  → explode damages more cells with source_entity = bolt
```

The bolt stays attributed through arbitrarily deep chains because `Destroyed` carries the killer, which feeds back into the next `TriggerContext`.

### No RON changes needed for context

`TriggerContext` is runtime-only. Not on `EffectNode`, not on `EffectKind`, not in RON. Just a parameter on the `fire()` call stack.

## Generic Damage Pipeline — `DamageDealt<T>` + `HealthShield`

### `DamageDealt<T>` replaces `DamageCell`

```rust
struct DamageDealt<T: Component> {
    pub target: Entity,
    pub damage: f32,
    pub source_chip: Option<String>,
    pub source_entity: Option<Entity>,
}
```

`T` is the target's marker component (`Cell`, `Bolt`, `Breaker`, `Wall`). Each domain registers its own `DamageDealt<T>` message and implements a listener system. Only `DamageDealt<Cell>` exists today (replacing `DamageCell`). Other variants are added later by registering the message type and adding a domain listener.

### `HealthShield` component

```rust
#[derive(Component)]
struct HealthShield {
    pub shield_hp: f32,
}
```

Added to any entity (cell, bolt, breaker, wall) via effects or builder. The domain's damage listener checks for `HealthShield` on the target:
1. Subtract damage from `shield_hp` first
2. If `shield_hp` reaches 0, remove the `HealthShield` component, apply remaining damage to entity health
3. If `shield_hp` absorbs all damage, entity health is untouched

This is distinct from the existing `Shield` effect (which creates a floor wall to prevent bolt-lost, with timer cost per reflection). `HealthShield` absorbs HP damage; `Shield` prevents bolt-lost.

### Domain damage listeners

```rust
// cells domain
fn apply_damage_to_cell(
    mut reader: MessageReader<DamageDealt<Cell>>,
    mut cells: Query<(&mut CellHealth, Option<&mut HealthShield>), With<Cell>>,
) {
    for msg in reader.read() {
        let Ok((mut health, shield)) = cells.get_mut(msg.target) else { continue };
        let effective_damage = absorb_shield(shield, msg.damage);
        health.take_damage(effective_damage);
    }
}
```

Each domain decides whether to register a damage listener. Bolts, breakers, and walls can opt in later.

### Interaction with kill pipeline

`DamageDealt<T>` feeds into the kill pipeline: when `take_damage` returns `is_destroyed() == true`, the domain sends `KillYourself<S, T>` (using `source_entity` from `DamageDealt` to determine S). The damage and kill pipelines are separate steps — damage reduces HP, kill handles death triggers.

## Remaining Design Details

### HealthShield component migration
`HealthShield { shield_hp: f32 }` is initially defined in `cells/components/` by the cell builder todo (#4) so cells can use it immediately. When this todo lands, move the component to `effect/` (or `effect/components/`) since it applies to any entity type. Update the cell builder's import path accordingly.

### Despawn timing
Entity must survive through `Killed`/`Died`/`Death` trigger evaluation + death animation before actual despawn. Needs a `PendingDespawn` marker or similar that a cleanup system processes after triggers have flushed.

### `was_required_to_clear` migration
Moves to a `RequiredToClear` component on cell entities. `handle_cell_kill` reads it. Downstream consumers query the component while entity still exists.

### `Die` effect S type resolution
`Die` fires on self — it knows T (the entity's own type from marker components). S (killer type) comes from `TriggerContext`: check each context field (bolt, breaker, cell, wall), first non-None wins. Sends the correctly typed `KillYourself<S, T>`.

## Status
`ready` — all major design questions resolved. Implementation touches: ~30 `fire()` signatures, `FireEffectCommand`, `fire_effect()`, `EffectKind::fire()`, `DamageCell.source_entity` field, `LastDamageSource` component, `KillYourself<S,T>`/`Destroyed<S,T>` messages, `DeathAttribution` trait, domain kill handlers, bridge systems, RON migration (17 files), Rust trigger migration (~25 test files).
