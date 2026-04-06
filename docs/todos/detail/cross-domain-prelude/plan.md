# Plan: Cross-Domain Prelude & Import Cleanup

## Context

Importing types across domains currently requires knowing exact module paths (e.g., `crate::bolt::components::BoltVelocity`). This is fragile and verbose. The codebase has also accumulated `super::super::` chains — **291 occurrences across 183 files** in breaker-game alone, plus **59 more across 55 files** in the other workspace crates. The worst offenders are `effect/core/types/definitions/fire.rs` and `reverse.rs`, which use `super::super::super::super::effects::` for every effect dispatch (59 occurrences in 2 files).

This refactor creates a `crate::prelude` module with category-based re-exports, then replaces all `super::super::` chains with `crate::` absolute paths. No logic changes — imports only.

## Phase 1: Create Prelude Module

**Scope**: New files only. No changes to existing code. Single wave.

### Files to Create

**`breaker-game/src/prelude/mod.rs`** — module root + curated tier 1-2 glob:
```rust
pub(crate) mod components;
pub(crate) mod messages;
pub(crate) mod resources;
pub(crate) mod states;

// Curated re-export: tier 1-2 universal types for `use crate::prelude::*`
pub(crate) use components::{
    Bolt, Breaker, Cell, Wall,                              // entity markers
    BoundEffects, StagedEffects,                            // effect containers
    NodeScalingFactor,  // lifecycle (CleanupOnExit<S> comes from rantzsoft_lifecycle directly)
    ActivePiercings, ActiveDamageBoosts, ActiveSizeBoosts,  // effect state (tier 2)
    ActiveSpeedBoosts, ActiveVulnerability,
    AnchorActive, AnchorPlanted, FlashStepActive,
};
pub(crate) use messages::*;  // all messages are cross-domain by definition
pub(crate) use resources::{PlayfieldConfig, GameRng, InputActions};  // universal resources
pub(crate) use states::*;  // all states are universal
```

**`breaker-game/src/prelude/states.rs`** — all state types:
```rust
pub(crate) use crate::state::types::{
    AppState, ChipSelectState, GameState, MenuState,
    NodeState, RunEndState, RunState,
};
```

**`breaker-game/src/prelude/components.rs`** — all cross-domain components:
- Entity markers: `Bolt`, `Breaker`, `Cell`, `Wall` (from their domain `components` modules)
- Shared: `NodeScalingFactor`, `BaseWidth`, `BaseHeight` (from `shared::components`); `CleanupOnExit<S>` comes from `rantzsoft_lifecycle` directly — not re-exported here
- Effect: `BoundEffects`, `StagedEffects`, `ActivePiercings`, `ActiveDamageBoosts`, `ActiveSizeBoosts`, `ActiveSpeedBoosts`, `ActiveVulnerability`, `AnchorActive`, `AnchorPlanted`, `FlashStepActive`, `LivesCount`, `ShieldWall` (from `effect`)
- Cross-domain bolt: `BoltServing` (from `bolt`)
- Cross-domain cells: `RequiredToClear` (from `cells`)

**`breaker-game/src/prelude/resources.rs`** — all cross-domain resources:
- Shared: `PlayfieldConfig`, `PlayfieldDefaults`, `GameRng`, `RunSeed` (from `shared`)
- Input: `InputActions` (from `input`)
- Bolt: `BoltRegistry` (from `bolt`)
- Breaker: `BreakerRegistry`, `SelectedBreaker` (from `breaker`)
- Cells: `CellTypeRegistry` (from `cells`)
- Chips: `ChipCatalog`, `ChipInventory` (from `chips`)

**`breaker-game/src/prelude/messages.rs`** — all cross-domain messages:
- Bolt messages: `BoltLost`, `BoltSpawned`, `BoltImpactBreaker`, `BoltImpactCell`, `BoltImpactWall`, `RequestBoltDestroyed`
- Breaker messages: `BumpPerformed`, `BumpWhiffed`, `BreakerSpawned`, `BreakerImpactCell`, `BreakerImpactWall`
- Cell messages: `DamageCell`, `CellDestroyedAt`, `RequestCellDestroyed`, `CellImpactWall`
- State messages: `NodeCleared`, `ApplyTimePenalty`, `ReverseTimePenalty`, `ChipSelected`

### Wiring

Add to `breaker-game/src/lib.rs`:
```rust
pub(crate) mod prelude;
```

### Key Files
- `breaker-game/src/shared/mod.rs` — existing common layer (unchanged, prelude re-exports from it)
- `breaker-game/src/effect/mod.rs` — already has `pub use core::*` re-exporting effect types
- `breaker-game/src/state/types/mod.rs` — source of all 7 state types
- `breaker-game/src/bolt/components/mod.rs`, `breaker/components/mod.rs`, `cells/components/mod.rs`, `walls/components/mod.rs` — sources for entity markers

### What's NOT in the prelude
- Effect enums (`EffectKind`, `RootEffect`, `EffectNode`, `Target`, `Trigger`) — these are already well-served by `use crate::effect::*` and are domain-specific to effect wiring
- Enums like `BumpGrade`, `Rarity`, `GameDrawLayer`, `GameAction` — narrow cross-domain usage
- Constants (`BOLT_LAYER`, etc.) — stay in `crate::shared`
- Plugins, system sets — wiring code only, not general imports
- `#[cfg(feature = "dev")]` types — out of scope for Phase 1
- `#[cfg(test)]` types — out of scope for Phase 1

---

## Phase 2: Replace `super::super::` Chains

**Scope**: Replace all `super::super::` (2+ levels) with `crate::` absolute paths. No logic changes. Pure import-path rewriting.

**Scale**: ~350 occurrences across ~238 files in 4 crates.

### Wave 2A: Effect dispatch files (highest impact, 2 files)

Replace `super::super::super::super::effects::X` → `crate::effect::effects::X` in:
- `breaker-game/src/effect/core/types/definitions/fire.rs` (30 occurrences)
- `breaker-game/src/effect/core/types/definitions/reverse.rs` (29 occurrences)

These two files account for 59 of the 350 occurrences.

### Wave 2B: Effect production code (~14 files)

Replace `super::super::entity_position` → `crate::effect::effects::entity_position` in:
- `effect/effects/spawn_bolts/effect.rs`
- `effect/effects/gravity_well/effect.rs`
- `effect/effects/chain_bolt/effect.rs`
- `effect/effects/explode/effect.rs`
- `effect/effects/spawn_phantom/effect.rs`
- `effect/effects/piercing_beam/effect.rs`
- `effect/effects/chain_lightning/effect.rs`
- `effect/effects/tether_beam/effect.rs`
- `effect/effects/shockwave/effect.rs`
- `effect/effects/pulse/effect.rs`
- `effect/effects/anchor/effect.rs`
- `effect/effects/circuit_breaker/effect.rs`

Plus: `effect/commands/ext.rs`, `effect/triggers/until/system.rs`

### Wave 2C: breaker-game test files (~165 files, by domain, parallelizable)

All test files using `super::super::` to reach parent helpers/systems. Replace with `crate::` paths. Domains can be done in parallel:

| Domain | Files | Pattern |
|--------|-------|---------|
| effect tests | ~80 | `super::super::effect::*` → `crate::effect::effects::<module>::*` |
| bolt tests | ~20 | `super::super::helpers::*` → `crate::bolt::systems::<system>::tests::helpers::*` etc. |
| breaker tests | ~15 | Same pattern as bolt |
| cells tests | ~5 | Same pattern |
| chips tests | ~15 | Same pattern |
| state tests | ~20 | Same pattern |
| walls tests | ~2 | Same pattern |
| debug tests | ~1 | Same pattern |

### Wave 2D: Other workspace crates (~55 files, parallelizable)

| Crate | Files | Occurrences |
|-------|-------|-------------|
| rantzsoft_spatial2d | 12 | 12 |
| rantzsoft_physics2d | 3 | 3 |
| breaker-scenario-runner | 40 | 44 |

---

## Implementation Strategy

This is a **mechanical refactor**, not a feature. No new behavior to test with TDD. The validation is: everything compiles, all existing tests pass, all scenarios pass.

### Sequencing

1. **Phase 1** (Wave 1): Create prelude module → Basic Verification Tier
2. **Phase 2** (Waves 2A-2D): Migrate imports → Basic Verification Tier per wave
3. **Standard Verification Tier** after all waves complete
4. **Commit**
5. **Full Verification Tier** → merge

### Parallelism

- Waves 2A + 2B can run sequentially (both touch effect domain, small scope)
- Wave 2C domains can run in parallel (each touches different files)
- Wave 2D crates can run in parallel with 2C (different crates entirely)

---

## Verification

- `cargo fmt` — formatting
- `cargo all-dclippy` — lints across all crates
- `cargo all-dtest` — tests across all crates
- `cargo scenario -- --all` — scenario runner
- Standard + Full verification tiers at commit/merge gates
