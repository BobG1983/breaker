# Effect System Architecture

The effect system is a data-driven trigger→effect pipeline. RON-defined effect trees are dispatched onto game entities at chip selection and breaker initialization. At runtime, game events (collisions, bumps, deaths) are bridged into triggers. Trigger systems evaluate each entity's effect chains, firing effects and arming deeper chains.

**Core principle: effects act on the entity they live on.** The entity IS the implicit target. There is no `targets` field, no typed event structs, no dispatch categorization. When `Do(SpeedBoost)` fires on a bolt entity, the bolt gets faster. When it fires on a breaker entity, the breaker gets faster.

## Components

### EffectChains

Permanent effect trees on an entity. Populated at dispatch time (chip selection, breaker init). **Never consumed by trigger evaluation** — entries persist and re-evaluate each time a matching trigger fires. This is the source of truth for what an entity reacts to.

### ArmedEffects

Working set of partially-resolved chains. Entries are **consumed when matched**. Populated by:
- EffectChains evaluation (nested When/Until/Once children pushed here)
- `On` redirects from other entities
- Until children armed for future triggers

## Node Types

### When(trigger, children)

Gate. If the current trigger matches, evaluate children. Otherwise skip.

- In **EffectChains**: permanent — re-evaluates on every matching trigger
- In **ArmedEffects**: consumed when matched

```ron
When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])
```

### Do(effect)

Terminal. Fire the effect on the entity whose chain is being evaluated. The effect handler receives the entity and queries it for needed components. If the entity lacks required components, graceful no-op.

```ron
Do(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))
```

### Once(children)

One-shot wrapper. Evaluate children against the current trigger. If any child matches, fire/arm it AND remove the Once from whichever component it's in. If nothing matches, keep it.

```ron
Once([When(trigger: BoltLost, then: [Do(SecondWind(invuln_secs: 1.0))])])
```

### On(target, children)

Redirect. Resolves target to entity/entities from trigger context. **On never fires anything on the current entity** — it only transfers chains to other entities.

- Bare `Do` children → fire directly on the target entity
- Non-Do children (When, Until, Once, nested On) → push to the target entity's ArmedEffects

```ron
On(target: Bolt, then: [Do(SpeedBoost(multiplier: 1.2))])
```

### Until(until_trigger, children)

Duration-scoped effects. When first encountered during evaluation:
1. Fires/arms children immediately on the entity
2. Stays in ArmedEffects as a runtime node with timer or trigger tracking

When the until-trigger fires (timer expires or trigger matches):
1. Reverses children (calls each effect's reverse function)
2. Removes itself from ArmedEffects

The Until node **is visible at runtime** — the timer system finds Until entries in ArmedEffects and ticks them.

```ron
Until(until: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
```

## Dispatch (Chip Selection / Breaker Init)

RON defines effects as `Vec<RootEffect>` where every top-level entry is `On(target, children)`:

```ron
// Chip: Surge
effects: [
    On(target: Bolt, then: [
        When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.2))])
    ])
]

// Breaker: Aegis
effects: [
    On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
    On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
]
```

The dispatch system resolves each `On(target)` to concrete entities and pushes `children` into those entities' **EffectChains**:

| Target | At dispatch |
|--------|------------|
| `Bolt` | Primary bolt entity. New bolts inherit if `SpawnBolts(inherit: true)`. |
| `AllBolts` | All bolt entities |
| `Breaker` | The breaker entity |
| `Cell` / `AllCells` / `Wall` | No-op (entities don't exist yet) |

Passives use `When(Selected, [Do(Piercing(1))])` — the `Selected` trigger fires at dispatch time.

## Target Resolution

**Singular targets** (Bolt, Cell, Wall, Breaker) are context-sensitive:
- At dispatch: `Bolt` = primary bolt, `Breaker` = breaker entity
- At runtime: resolved from trigger context. The trigger system fills these out from the message it bridged (e.g., Impacted provides both collision participants).

**Plural targets** (AllBolts, AllCells) always resolve via query — push to all matching entities.

**Breaker** resolves via query (single entity) in both contexts.

## Triggers

Each trigger type has its own module in `effect/triggers/`. A trigger module contains:
1. A **bridge system** that reads game messages and translates them into trigger evaluation
2. Knowledge of **which entities to evaluate** (global vs targeted)
3. Knowledge of **how to resolve On targets** from trigger context

### Trigger Scope Table

**Global triggers** — sweep ALL entities with EffectChains:

| Trigger | Context entities |
|---------|-----------------|
| `PerfectBump` | bolt |
| `EarlyBump` | bolt |
| `LateBump` | bolt |
| `BumpWhiff` | (none) |
| `NoBump` | bolt |
| `Death` | dying entity |
| `BoltLost` | (none) |
| `CellDestroyed` | cell position |
| `NodeTimerThreshold(f32)` | (none) |

**Targeted triggers** — evaluate only the specific entity:

| Trigger | Evaluated on | Context entities |
|---------|-------------|-----------------|
| `PerfectBumped` | the bolt | bolt |
| `EarlyBumped` | the bolt | bolt |
| `LateBumped` | the bolt | bolt |
| `Impacted(Cell)` | the entity that hit a cell | both entities |
| `Impacted(Bolt)` | the entity hit by a bolt | both entities |
| `Impacted(Wall)` | the entity that hit a wall | both entities |
| `Impacted(Breaker)` | the entity that hit a breaker | both entities |
| `Died` | the dying entity | dying entity |

**Special triggers:**

| Trigger | Behavior |
|---------|----------|
| `Selected` | Fires at dispatch time (chip selection) |
| `TimeExpires(f32)` | Timer system ticks Until entries in ArmedEffects |

### Impacted Fires Both Directions

When a bolt hits a cell, the bridge fires TWO triggers:
- `Impacted(Cell)` on the bolt — "I hit a cell"
- `Impacted(Bolt)` on the cell — "I was hit by a bolt"

Both entities receive context about the other. Any entity type can be on either side as mechanics expand (moving cells, breaker collisions, etc.).

### Bump / Bumped Split

| Trigger | Scope | Perspective |
|---------|-------|------------|
| `PerfectBump` | Global | "A perfect bump happened" |
| `PerfectBumped` | Targeted (bolt) | "I was perfect bumped" |
| `Bump` / `Bumped` | Global / Targeted | Any non-whiff bump |
| `EarlyBump` / `EarlyBumped` | Global / Targeted | Early bump |
| `LateBump` / `LateBumped` | Global / Targeted | Late bump |
| `BumpWhiff` | Global | "I whiffed" (breaker only) |
| `NoBump` | Global | "Bolt hit me without bump input" |

### Death / Died Split

| Trigger | Scope | Perspective |
|---------|-------|------------|
| `Death` | Global | "Something died" — sweep all entities |
| `Died` | Targeted | "I died" — on the dying entity |

## Evaluation Algorithm

For a given entity receiving trigger T, the trigger system runs the collect/execute algorithm:

### Phase 1 — Collect

Loop through ArmedEffects, then EffectChains. Build three vecs: `to_fire`, `to_arm`, `to_transfer`.

For each node:

**When(trigger, children):** If T matches trigger, evaluate children:
- `Do(effect)` → add to `to_fire`
- Anything else → add to `to_arm`
- If from ArmedEffects: consume the entry (remove it)
- If from EffectChains: entry stays (permanent)

**Until(until_trigger, children):** If T matches the until-trigger:
- Reverse children (call each Do child's reverse function)
- Remove the Until from ArmedEffects

**Once(children):** Evaluate children against T:
- If any child matches: fire/arm it, remove the Once from its component
- If nothing matches: keep the Once

**On(target, children):** Resolve target from trigger context. Add `(resolved_entities, children)` to `to_transfer`.

**Do(effect):** Should not appear bare at runtime, but if encountered: add to `to_fire`.

### Phase 2 — Execute

1. **Fire** all effects in `to_fire` on this entity (in order)
2. **Arm** — push all entries in `to_arm` into this entity's ArmedEffects
3. **Transfer** — for each entry in `to_transfer`:
   - Bare `Do` children → fire directly on the target entity
   - Non-Do children → push to the target entity's ArmedEffects

## Effect Execution

When `Do(effect)` fires on entity X:
- The effect's handler function receives entity X and the effect parameters
- It queries entity X for whatever components it needs (Position2D, Velocity2D, BoltBaseSpeed, etc.)
- If the entity doesn't have the required components, graceful no-op
- **Direct function call** — no typed events, no observer event structs, no bolt/global/pool dispatch

## Reversal

Each reversible effect defines a reverse function in its own file. The `Effect` enum has a method:

```rust
impl Effect {
    fn reverse(&self, entity: Entity, commands: &mut Commands) {
        match self {
            Effect::SpeedBoost { multiplier } => reverse_speed_boost(entity, multiplier, commands),
            Effect::DamageBoost(value) => reverse_damage_boost(entity, value, commands),
            // ... each reversible variant calls its own function
            _ => {} // non-reversible effects (Shockwave, SpawnBolts, etc.) are no-ops
        }
    }
}
```

Each match arm is one line — a call to a function defined in the effect's own file. All reversal logic lives in the effect module. The Until system never imports effect internals.

Non-reversible effects (Shockwave, SpawnBolts, LoseLife, etc.) are no-ops on reverse.

## Buff Stacking

Passive effects that modify stats are tracked in per-entity vecs. Each application pushes an entry; each removal removes one entry. The actual stat is **recalculated from the vec** every tick — no incremental mutation.

| Effect | Stacking | Recalculation |
|--------|----------|---------------|
| `SpeedBoost` | Multiplicative | `base_speed * product(boosts)`, clamped `[min, max]` |
| `DamageBoost` | Multiplicative | `base_damage * product(boosts)` |
| `Piercing` | Additive | `sum(pierce_counts)` |
| `SizeBoost` | Additive | `base_size + sum(boosts)` |
| `BumpForce` | Additive | `base_force + sum(boosts)` |

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half).

## Bridge Systems

Bridge systems live in `effect/triggers/<trigger_name>.rs`. Each bridge:
1. Reads a game message (e.g., `BoltHitCell`, `BumpPerformed`, `CellDestroyRequested`)
2. Translates it into trigger evaluation with the right context
3. Knows which entities to evaluate (global: all with EffectChains, targeted: specific)
4. Knows how to resolve On targets from message data

Example — `bridge_bolt_hit_cell` reads `BoltHitCell { bolt, cell }`:
- Fires `Impacted(Cell)` trigger, evaluating the bolt entity (context: bolt + cell)
- Fires `Impacted(Bolt)` trigger, evaluating the cell entity (context: bolt + cell)

## Domain Structure

```
effect/
  definition/        — Effect, Trigger, EffectNode, Target, RootEffect enums
  triggers/          — one module per trigger type (bridge + evaluation)
  effects/           — one module per effect (handler, reverse, components, register)
  evaluate.rs        — shared evaluation algorithm (collect/execute phases)
  plugin.rs          — EffectPlugin (calls each effect's register())
```

Each effect module in `effects/` contains:
- The fire handler function
- The reverse function (if reversible)
- Per-effect components (e.g., `ActiveSpeedBoosts`)
- Runtime systems (e.g., `apply_speed_boosts` recalculation)
- `register(app)` function that wires runtime systems

## Examples

### Overclock — temporary speed boost on perfect bump

```ron
On(target: Bolt, then: [
    When(trigger: PerfectBumped, then: [
        Until(until: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
    ])
])
```

1. **Dispatch**: pushes `When(PerfectBumped, [Until(...)])` to bolt's EffectChains
2. **PerfectBumped fires on bolt**: When matches → child is Until → Until fires SpeedBoost immediately on bolt, arms itself in ArmedEffects with 2.0s timer
3. **2 seconds pass**: timer system decrements, hits zero → Until calls `SpeedBoost.reverse()` → removes entry from ActiveSpeedBoosts → recalculation system restores base speed
4. **EffectChains entry stays** — next perfect bump re-triggers

### Wall redirecting to bolt

```ron
On(target: Wall, then: [
    When(trigger: Impacted(Bolt), then: [
        On(target: Bolt, then: [Do(SpeedBoost(multiplier: 1.2))])
    ])
])
```

1. **Dispatch**: pushes `When(Impacted(Bolt), [On(Bolt, [Do(SpeedBoost)])])` to wall's EffectChains
2. **Bolt hits wall**: `Impacted(Bolt)` fires on wall → When matches → child is On(Bolt, ...) → added to `to_transfer`
3. **Transfer phase**: On resolves Bolt from trigger context → `Do(SpeedBoost)` is bare Do → fire SpeedBoost directly on the bolt entity

### Cascade — shockwave on cell destruction

```ron
On(target: Bolt, then: [
    When(trigger: CellDestroyed, then: [Do(Shockwave(base_range: 20.0, ...))])
])
```

1. **Dispatch**: pushes `When(CellDestroyed, [Do(Shockwave)])` to bolt's EffectChains
2. **CellDestroyed fires** (global sweep): bolt's When matches → Shockwave fires on bolt → queries bolt's Position2D → spawns shockwave at bolt position

### Last Stand — breaker speed boost on bolt lost

```ron
On(target: Breaker, then: [
    When(trigger: BoltLost, then: [Do(SpeedBoost(multiplier: 1.15))])
])
```

1. **Dispatch**: pushes `When(BoltLost, [Do(SpeedBoost)])` to breaker's EffectChains
2. **BoltLost fires** (global sweep): breaker's When matches → SpeedBoost fires on breaker → queries breaker for speed components → boosts breaker speed (not bolt speed — effect acts on self)

### Impact chip — nested triggers with arming

```ron
On(target: Bolt, then: [
    When(trigger: PerfectBumped, then: [
        When(trigger: Impacted(Cell), then: [
            Do(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))
        ])
    ])
])
```

1. **Dispatch**: pushes outer When to bolt's EffectChains
2. **PerfectBumped fires on bolt**: outer When matches → inner When is non-Do → pushed to bolt's ArmedEffects
3. **Bolt hits cell**: `Impacted(Cell)` fires on bolt → ArmedEffects entry matches → Shockwave fires on bolt → ArmedEffects entry consumed
4. **EffectChains entry stays** — next perfect bump re-arms
