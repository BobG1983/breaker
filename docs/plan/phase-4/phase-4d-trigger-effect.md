# Phase 4d: Trigger/Effect Architecture

**Goal**: Recursive RON-defined trigger chains for overclocks. Bolt behaviors domain. Surge overclock as proof-of-concept.

## Dependencies

- 4b (Chip Effect System) — need the effect application mechanism

## What to Build

### RON Trigger Syntax

Recursive enum — arbitrary nesting depth:

```ron
// Simple: trigger -> effect
OnCellDestroyed(Shockwave(range: 64.0))

// Chained: trigger -> trigger -> effect
OnPerfectBump(OnImpact(Shockwave(range: 64.0)))

// Deep: trigger -> trigger -> trigger -> effect
OnPerfectBump(OnImpact(OnCellDestroyed(MultiBolt(count: 2))))
```

### Rust Types

```rust
/// A trigger chain that evaluates conditions and fires an effect.
#[derive(Deserialize, Clone, Debug)]
enum TriggerChain {
    // Leaf — fire this effect when all parent triggers are satisfied
    Shockwave { range: f32 },
    MultiBolt { count: u32 },
    Shield { duration: f32 },
    // ... more effects

    // Triggers — each wraps another TriggerChain
    OnPerfectBump(Box<TriggerChain>),
    OnImpact(Box<TriggerChain>),
    OnCellDestroyed(Box<TriggerChain>),
    OnBoltLost(Box<TriggerChain>),
    // ... more triggers
}
```

### Bolt Behaviors Domain

New `src/bolt/behaviors/` module (mirrors `src/breaker/behaviors/`):
- Bolt behavior definitions loaded from RON
- Trigger evaluation system that reads bolt state + game messages
- Intermediate state tracking: when a trigger fires but the chain continues, a marker component is added to the bolt (e.g., `Surging`) to track that the chain is partially evaluated
- Effect execution systems (shockwave, multi-bolt, shield, etc.)

### Shockwave Effect (Surge Overclock)

The first concrete overclock, proving the architecture:
- **Trigger chain**: `OnPerfectBump(OnImpact(Shockwave(range: 64.0)))`
- **Flow**: Perfect bump -> mark bolt as "surging" -> on next impact -> fire shockwave at impact point
- **Shockwave**: expanding ring VFX, any cell within range takes 1 damage
- **Range parameter**: RON-configurable, upgradeable via stacking

### Hot-Reload Support

Overclock RON changes -> rebuild trigger chains -> re-evaluate active overclocks

## Acceptance Criteria

1. Surge overclock works end-to-end: perfect bump -> impact -> shockwave -> cells damaged
2. Trigger chains parse from RON with arbitrary nesting
3. Intermediate state (surging marker) is properly set and consumed
4. Shockwave visual effect plays at impact point
5. Adding a new trigger or effect requires only a new enum variant + handler — no system rewiring
