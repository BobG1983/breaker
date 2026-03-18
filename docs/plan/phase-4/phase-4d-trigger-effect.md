# Phase 4d: Trigger/Effect Architecture

**Goal**: Recursive RON-defined trigger chains for overclocks. Bolt behaviors domain. Surge overclock as proof-of-concept.

**Wave**: 2 (after 4b) — parallel with 4c and 4e. **Highest-risk stage in Phase 4.**

## Dependencies

- 4b (Chip Effect System) — need the effect application mechanism

## Pre-Implementation

Use **researcher-bevy-api** to verify the observer/event pattern needed for trigger chain evaluation before writing any code. This stage introduces an architecturally novel pattern (recursive trigger evaluation with intermediate state) that doesn't exist anywhere else in the codebase.

## Sub-Stages

### 4d.1: TriggerChain Types + RON Parsing (Session 5)

**Domain**: chips/ or shared (types only)

Define the recursive enum and verify RON round-trips:

```rust
/// A trigger chain that evaluates conditions and fires an effect.
#[derive(Deserialize, Clone, Debug)]
enum TriggerChain {
    // Leaf — fire this effect when all parent triggers are satisfied
    Shockwave { range: f32 },
    MultiBolt { count: u32 },
    Shield { duration: f32 },

    // Triggers — each wraps another TriggerChain
    OnPerfectBump(Box<TriggerChain>),
    OnImpact(Box<TriggerChain>),
    OnCellDestroyed(Box<TriggerChain>),
    OnBoltLost(Box<TriggerChain>),
}
```

**RON examples**:
```ron
// Simple: trigger -> effect
OnCellDestroyed(Shockwave(range: 64.0))

// Chained: trigger -> trigger -> effect
OnPerfectBump(OnImpact(Shockwave(range: 64.0)))

// Deep: trigger -> trigger -> trigger -> effect
OnPerfectBump(OnImpact(OnCellDestroyed(MultiBolt(count: 2))))
```

**Delegatable**: Yes — pure types + parsing tests.

### 4d.2: Bolt Behaviors Module + Intermediate State (Session 5)

**Domain**: bolt/

New `src/bolt/behaviors/` module (mirrors `src/breaker/behaviors/`):
- Bolt behavior definitions loaded from RON
- Trigger evaluation system that reads bolt state + game messages
- Intermediate state tracking: marker components (e.g., `Surging`) added to bolt when a trigger fires but the chain continues

**Delegatable**: Yes — writer-tests → writer-code, scoped to bolt/ domain.

### 4d.3: Shockwave Effect Implementation (Session 6)

**Domain**: bolt/ or physics/

The first concrete effect, proving the leaf-effect execution path:
- **Shockwave**: expanding ring VFX, any cell within range takes 1 damage
- **Range parameter**: RON-configurable, upgradeable via stacking
- Shockwave queries all Cell entities within range of impact point

**Delegatable**: Yes — scoped system + VFX.

### 4d.4: Surge Overclock End-to-End (Session 6)

**Integration task** — likely manual (main agent):

- **Trigger chain**: `OnPerfectBump(OnImpact(Shockwave(range: 64.0)))`
- **Flow**: Perfect bump → mark bolt "surging" → on next impact → fire shockwave at impact point
- Wires together 4d.1 (types), 4d.2 (trigger evaluation), 4d.3 (shockwave)
- Validates the architecture works end-to-end

### Hot-Reload Support

Overclock RON changes → rebuild trigger chains → re-evaluate active overclocks.

## Scenario Coverage

### New Invariants
- **`TriggerChainDepthBounded`** — active trigger chains never exceed a configurable max depth (prevents infinite recursion if a trigger chain accidentally references itself). Checked every frame.
- **`ShockwaveRadiusBounded`** — shockwave range never exceeds playfield dimensions (sanity check on RON-configured values).

### New Scenarios
- `mechanic/surge_overclock.scenario.ron` — Chaos input with Surge overclock active. High-frequency perfect bumps (scripted initial sequence to trigger surging state, then chaos). Verifies `BoltInBounds`, `NoNaN`, `NoEntityLeaks` (shockwave entities must despawn).
- `stress/overclock_chain_stress.scenario.ron` — Multiple overclocks active simultaneously under chaos input. Verifies no entity leaks from VFX, no NaN from stacked effects, and trigger chains resolve cleanly.

### Existing Scenario Updates
- Existing prism scenarios (which already test multi-bolt) should pass with overclock components present but inactive.

## Acceptance Criteria

1. Surge overclock works end-to-end: perfect bump → impact → shockwave → cells damaged
2. Trigger chains parse from RON with arbitrary nesting
3. Intermediate state (surging marker) is properly set and consumed
4. Shockwave visual effect plays at impact point
5. Adding a new trigger or effect requires only a new enum variant + handler — no system rewiring
