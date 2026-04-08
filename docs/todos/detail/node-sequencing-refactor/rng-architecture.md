# RNG Architecture — Implementation Plan

Based on [RNG usage research](research/rng-usage.md). This document specifies the concrete changes needed to move from the current flat single-stream `GameRng` to hierarchical seed derivation with domain-partitioned RNG.

## Current State (what exists today)

- Single `GameRng(ChaCha8Rng)` resource in `shared/rng.rs`
- `RunSeed(Option<u64>)` resource in `shared/resources.rs`
- 19 call sites all drawing from the same stream (see research doc for full table)
- No domain partitioning, no visual/gameplay separation
- Ordering gaps in `OnEnter(NodeState::Loading)` and between effect systems in `FixedUpdate`

## Target State

### New Resources

| Resource | Type | Seeded From | Purpose | Reset When |
|----------|------|-------------|---------|------------|
| `GameRng` | `ChaCha8Rng` | `run_seed` | **Removed as a shared draw target.** Kept only as the root entropy source for deriving sub-seeds at run start. After sub-seeds are derived, nothing should draw from `GameRng` directly during gameplay. | `OnExit(MenuState::Main)` |
| `FxRng` | `ChaCha8Rng` | OS entropy | Visual-only randomness (popup jitter, transition style). Never affects gameplay. Not tied to run_seed — cosmetic variance across seed-shared runs is fine. | App startup (once) |
| `NodeSequenceRng` | `ChaCha8Rng` | `hash(run_seed, "node_sequence")` | Node sequence generation (tier counts, shuffle). Used only in `generate_node_sequence_system`. | `OnExit(MenuState::Main)` |
| `NodeGenRng` | `ChaCha8Rng` | `hash(run_seed, tier_index, node_index)` | Frame selection, block selection, slot splitting, constraint resolution for a single node. | `OnEnter(NodeState::Loading)` |
| `BoltRng` | `ChaCha8Rng` | `hash(run_seed, "bolt", node_index)` | Bolt launch angles, respawn angles. Per-node seed so bolt behavior is reproducible per node. | `OnEnter(NodeState::Loading)` |
| `ChipRng` | `ChaCha8Rng` | `hash(run_seed, "chip", chip_select_index)` | Chip offering weighted selection. Per-chip-select seed. | `OnEnter(GameState::ChipSelect)` |
| `ProtocolRng` | `ChaCha8Rng` | `hash(run_seed, "protocol", offering_index)` | Protocol offering selection (when player picks protocols between tiers). Per-offering seed. | When protocol selection triggers |
| `HazardRng` | `ChaCha8Rng` | `hash(run_seed, "hazard", tier_index)` | Hazard pool selection (3 random hazards offered per tier). Per-tier seed. | When tier boundary triggers hazard selection |
| `EffectEventCounter` | `u64` | 0 | Monotonic counter incremented each time the effect bridge dispatches an effect. Used to derive per-effect RNG. | `OnEnter(NodeState::Loading)` |

### Per-Effect RNG (not a resource — ephemeral)

Effects don't get a persistent resource. Instead, when the effect bridge dispatches an effect:

```rust
// In the effect bridge dispatch:
let effect_seed = hash(node_seed, effect_event_counter.next());
let mut effect_rng = ChaCha8Rng::seed_from_u64(effect_seed);
// Pass effect_rng to the effect's fire() function
```

The bridge processes effects from a message queue (ordered by send time). Since the collision system produces messages deterministically given the same physics state, the counter increments in the same order across sessions. Each effect fire gets the same seed regardless of what other systems did with other RNG resources that tick.

**Effect fire() signature change**: `fire()` currently takes `ResMut<GameRng>`. Change to take `&mut ChaCha8Rng` (the ephemeral per-event RNG). Same for `tick_chain_lightning` — it needs a derived RNG per tick per chain entity, seeded from `hash(node_seed, "chain_tick", chain_entity_index, tick_number)`.

### Seed Derivation Tree

```
run_seed (from RunSeed or OS entropy, captured before anything else)
  │
  ├─ hash(run_seed, "node_sequence")     → NodeSequenceRng
  │    └─ used by: generate_node_sequence_system
  │
  ├─ hash(run_seed, "tier", tier_index)  → per-tier derivation
  │    └─ hash(tier_seed, node_index)    → NodeGenRng (per node)
  │         ├─ frame selection
  │         ├─ slot splitting decisions
  │         ├─ block selection per slot
  │         └─ constraint resolution per cell
  │
  ├─ hash(run_seed, "bolt", node_index)  → BoltRng (per node)
  │    ├─ launch angles
  │    └─ respawn angles
  │
  ├─ hash(run_seed, "chip", select_idx)  → ChipRng (per chip select)
  │    └─ weighted chip offerings
  │
  ├─ hash(run_seed, "protocol", idx)      → ProtocolRng (per protocol offering)
  │    └─ protocol selection from weighted pool
  │
  ├─ hash(run_seed, "hazard", tier_idx)  → HazardRng (per tier)
  │    └─ 3 random hazards offered from pool
  │
  └─ hash(run_seed, "effect")            → base seed for effect derivation
       └─ hash(effect_base, counter)     → ephemeral per-effect-fire RNG
            ├─ chain lightning target
            ├─ spawn bolts angle
            ├─ phantom bolt angle
            ├─ chain bolt angle
            ├─ tether beam angles
            ├─ random effect selection
            └─ entropy engine selections
```

### Hash Function

Use a simple, fast hash that combines a seed with a discriminator. `ChaCha8Rng` is already cryptographic-quality so the hash doesn't need to be fancy — just needs to produce distinct seeds:

```rust
fn derive_seed(parent: u64, discriminator: u64) -> u64 {
    // Simple hash: XOR with golden ratio constant, then mix
    let mut h = parent ^ discriminator;
    h = h.wrapping_mul(0x517cc1b727220a95);  // large prime
    h ^= h >> 32;
    h
}

fn derive_seed_named(parent: u64, name: &str) -> u64 {
    // For named channels: hash the name to a u64 first
    let name_hash = name.bytes().fold(0u64, |acc, b| {
        acc.wrapping_mul(31).wrapping_add(b as u64)
    });
    derive_seed(parent, name_hash)
}
```

## Systems to Modify

### Run Start (OnExit(MenuState::Main))

| System | Current | New |
|--------|---------|-----|
| `reset_run_state` | Reseeds `GameRng` from `RunSeed` or OS entropy | Same, but also: capture run_seed into `RunStats.seed` HERE (not later). Derive and insert `NodeSequenceRng`. |
| `capture_run_seed` | Runs in `OnEnter(NodeState::Loading)`, draws from GameRng, reseeds | **Move to `OnExit(MenuState::Main)`, ordered before `generate_node_sequence_system`**. For unseeded runs: draw one u64 from GameRng to establish run_seed, then reseed GameRng from it. For seeded runs: use the provided seed directly. Either way, `RunStats.seed` is set before any gameplay draws. |
| `generate_node_sequence_system` | Draws from `GameRng` | Draws from `NodeSequenceRng` instead. Ordered `.after(capture_run_seed)`. |

### Node Loading (OnEnter(NodeState::Loading))

| System | Current | New |
|--------|---------|-----|
| `setup_run` | Draws bolt angle from `GameRng` | Draws from `BoltRng`. `BoltRng` seeded from `hash(run_seed, "bolt", node_index)` at node start. |
| `reset_bolt` | Draws bolt angle from `GameRng` | Draws from `BoltRng`. |
| New: `seed_node_rng` | — | New system. Derives and inserts `NodeGenRng` from `hash(run_seed, "tier", tier_index, node_index)`. Resets `EffectEventCounter` to 0. Runs before `setup_run` and `reset_bolt`. |

### Gameplay (FixedUpdate, Playing)

| System | Current | New |
|--------|---------|-----|
| `launch_bolt` | `ResMut<GameRng>` | `ResMut<BoltRng>` |
| `bolt_lost` | `ResMut<GameRng>` | `ResMut<BoltRng>` |
| `tick_chain_lightning` | `ResMut<GameRng>` | Derive per-chain-per-tick RNG from `hash(effect_base_seed, chain_entity_index, tick)`. No shared resource needed — each chain instance gets its own deterministic stream. |
| Effect bridge `fire()` calls | `ResMut<GameRng>` | `&mut ChaCha8Rng` (ephemeral, derived from `hash(effect_base_seed, event_counter)`) |
| All effect `fire()` impls | Take `ResMut<GameRng>` | Take `&mut ChaCha8Rng` parameter instead |

### Chip Select (OnEnter(GameState::ChipSelect))

| System | Current | New |
|--------|---------|-----|
| `generate_chip_offerings` | `ResMut<GameRng>` | `ResMut<ChipRng>`. `ChipRng` seeded from `hash(run_seed, "chip", chip_select_count)` where count tracks how many chip selects have occurred this run. |

### Visual (Update / OnEnter transitions)

| System | Current | New |
|--------|---------|-----|
| `spawn_highlight_text` | `ResMut<GameRng>` | `ResMut<FxRng>` |
| `spawn_transition_out` | `ResMut<GameRng>` | `ResMut<FxRng>` |
| `spawn_transition_in` | `ResMut<GameRng>` | `ResMut<FxRng>` |

## New Files

| File | Purpose |
|------|---------|
| `shared/rng.rs` | Expand existing — add `FxRng`, `NodeSequenceRng`, `NodeGenRng`, `BoltRng`, `ChipRng`, `EffectEventCounter`, `derive_seed()`, `derive_seed_named()` |
| `docs/architecture/rng.md` | Architecture doc — seed derivation tree, what the run seed covers, seed-sharing guarantees |

## Ordering Constraints (new)

```
OnExit(MenuState::Main):
  capture_run_seed → reset_run_state → generate_node_sequence_system

OnEnter(NodeState::Loading):
  seed_node_rng → setup_run
  seed_node_rng → reset_bolt

FixedUpdate (Playing):
  (no new ordering needed — BoltRng and effect RNG are independent resources/ephemerals)
```

## What the Run Seed Covers (for docs/architecture/rng.md)

With hierarchical seeding, the run seed deterministically produces:
- Node sequence (tier ordering, node type distribution, boss order)
- Node composition (frame selection, block selection, constraint resolution per cell)
- Cell behaviors and layouts
- Bolt launch and respawn angles
- Chip offerings at each chip select
- Protocol offerings at each protocol selection
- Hazard offerings at each tier boundary

NOT covered (by design — gameplay-dependent, varies with player input):
- Effect fire outcomes (chain targets, spawn angles, random effect picks, entropy engine rolls) — deterministic given same player input, but expected to differ between players sharing a seed
- Visual effects (popup positions, transition styles) — uses FxRng, cosmetic only
- Scenario runner input injection — uses separate SmallRng
- OS entropy for unseeded runs — by design, each unseeded run is unique

## Scenario Runner Impact

No changes needed. The scenario runner has its own `SmallRng` for input injection (completely separate from game RNG) and forces `RunSeed(Some(scenario.seed.unwrap_or(0)))` via `bypass_menu_to_playing`. The game's `OnExit(MenuState::Main)` systems derive all sub-seeds from `RunSeed`, so scenarios produce deterministic game state automatically after the migration. The runner never touches `GameRng` or any of the new domain-specific RNG resources directly.

## Migration Strategy

This can be implemented incrementally:
1. **Phase 1**: Add `FxRng`, migrate 3 visual systems. Zero gameplay risk.
2. **Phase 2**: Add `BoltRng`, migrate `launch_bolt`, `bolt_lost`, `setup_run`, `reset_bolt`. Pin ordering.
3. **Phase 3**: Add `ChipRng`, migrate `generate_chip_offerings`.
4. **Phase 4**: Add `EffectEventCounter` + ephemeral effect RNG, migrate all effect `fire()` signatures and `tick_chain_lightning`.
5. **Phase 5**: Add `NodeSequenceRng`, move `capture_run_seed` to `OnExit(MenuState::Main)`, migrate `generate_node_sequence_system`.
6. **Phase 6**: Add `NodeGenRng` + `seed_node_rng` system — this is the node sequencing refactor itself.
7. **Phase 7**: Write `docs/architecture/rng.md`.

Phases 1-5 can land before the node sequencing refactor. Phase 6 lands with it. Phase 7 lands after.
