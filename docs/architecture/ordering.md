# System Ordering

## The Principle

**Loose with key constraints.** No named phase sets, no global pipeline. Add `.before()` / `.after()` only where actual data dependencies exist. Let Bevy parallelize everything else.

If system B reads what system A writes (a component value, a message, a resource field), order them. Otherwise leave them unordered. Speculative ordering — "this feels like it should run after that" — creates fragile chains that break the moment someone changes the data flow.

## SystemSet Convention

Cross-domain ordering MUST go through a domain-owned `SystemSet`, never a bare function name.

Each domain that exposes ordering points defines `pub enum {Domain}Systems` in its `sets.rs` (e.g. `BoltSystems`, `BreakerSystems`, `EffectV3Systems`, `NodeSystems`). The owning domain tags one or more systems with `.in_set(Variant)`; consuming domains order with `.after(Variant)`.

Rules:

- **Variants name a pivotal anchor**, not the system itself. `BoltSystems::CellCollision` is the moment in the frame when bolt-vs-cell collisions have been resolved — not the literal `bolt_cell_collision` function.
- **Don't reference bare system names across domain boundaries.** Refactors rename functions; sets are stable.
- **Only create a variant when another domain actually orders against it.** Empty sets exist for one reason: to be ordered against. If nothing consumes it, delete it.
- **Group systems with the same constraint.** `(sys_a, sys_b).after(Target)` keeps the shared dependency in one place.
- **Phase sets are an exception to the pivotal-anchor rule.** A SystemSet variant that represents a *pipeline phase* (multiple plugins legitimately contribute) may be tagged by external systems. The owning plugin still configures the chain. The `rantzsoft_dmg` crate's `DmgSystems` is the canonical example — its 16 variants are pipeline phases each hosting multiple systems. Don't invent phase sets speculatively.

```rust
// breaker/sets.rs
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    Move, Reset, GradeBump, UpdateState,
    UpdatePreviousState, HandleBoltLost, CellCollision,
}

// breaker/plugin.rs — owning domain tags
move_breaker.in_set(BreakerSystems::Move)

// bolt/plugin.rs — consuming domain orders
hover_bolt.after(BreakerSystems::Move)
```

## The Cross-Domain FixedUpdate Flow

The canonical per-tick ordering, at a conceptual level:

```
maintain_quadtree                                    [physics]
  → breaker_move → bolt_collisions (cell, wall, breaker)   [collisions tagged BoltSystems::*Collision]
    → grade_bump                                     [BreakerSystems::GradeBump]
      → effect bridges                               [EffectV3Systems::Bridge]
        → effect ticks                               [EffectV3Systems::Tick]
          → effect conditions                        [EffectV3Systems::Conditions]
            → DmgSystems::EmitDamage → MutateDamage → ApplyDamage
              → DmgSystems::EmitKill → ApplyKill
                → EffectV3Systems::Death             [death bridges]
                  → DmgSystems::EmitHeal → ApplyHeal
```

Then `process_despawn_requests` runs in `FixedPostUpdate` (after all `FixedUpdate` consumers have observed the dying entity).

The `rantzsoft_dmg` chain (`EmitDamage → … → ApplyHeal`, 16 variants) is configured `.chain()` inside `RantzDmgPlugin` — game systems just tag in. **Game-side `DamageDealt<T>` writers MUST live in `DmgSystems::EmitDamage`** so emissions accumulate before the chain flushes. The exception: systems already in `EffectV3Systems::Tick` are transitively before `EmitDamage` and tagging them again creates a cycle.

For exhaustive per-system constraints, read the source — `rg "in_set\(" -t rust` and `rg "\.after\(" -t rust` are authoritative. This document is for principles; the source is for facts.

## Schedule Placement

| Schedule | What goes here | Why |
|----------|----------------|-----|
| `FixedUpdate` | All gameplay and physics | Deterministic, seed-reproducible. Must use `Velocity2D` / `Position2D`, not `Transform`. |
| `Update` | Visuals only — interpolation, UI, shaders | Variable framerate; never mutates gameplay state. |
| `FixedPostUpdate` | Despawn flush | Lets `FixedUpdate` consumers observe dying entities one last time. |
| `FixedFirst` | Snapshot last-tick state for interpolation | `save_previous` from `rantzsoft_spatial2d`. |
| `RunFixedMainLoop` / `AfterFixedMainLoop` | Transform derivation | After all FixedUpdate ticks, before render. Game systems NEVER write `Transform`. |
| `OnEnter(State)` / `OnExit(State)` | Spawn / despawn / resource init / per-state cleanup | One-shot setup, not per-tick mechanics. |

If you find yourself reaching for `Update` to mutate gameplay state, you have a bug, not an exception.
