# RNG Architecture

## Overview

Gameplay randomness is partitioned across per-domain RNG resources, each
seeded deterministically from the run seed via a hierarchical seed derivation
tree. The same `RunSeed` produces identical node sequences, node composition,
bolt launch angles, chip offerings, protocol offerings, and hazard offerings
across runs. Visual randomness uses a separate OS-entropy stream (`FxRng`)
that is intentionally not tied to the run seed — cosmetic variance across
runs that share a gameplay seed is allowed by design. Effect dispatch uses
ephemeral per-fire RNG derived from a stable `EffectBaseSeed` plus an
`EffectEventCounter`, keeping individual effect outcomes deterministic while
isolating their streams from each other.

The legacy `GameRng` type is still present in `shared/rng.rs` as the
root-entropy resource initialized at run start (consumed by
`capture_run_seed` and `reset_run_state`). Gameplay systems do not draw from
`GameRng` directly anymore; all gameplay randomness flows through the
domain-specific RNG resources listed below.

## Per-Domain RNG Resources

| Resource | Seeded From | Reset Point | Purpose |
|----------|-------------|-------------|---------|
| `FxRng` | OS entropy | App startup | Visual jitter (popup positions, transition style). Cosmetic only — never affects gameplay. |
| `NodeSequenceRng` | `derive_seed_named(run_seed, "node_sequence")` | `OnExit(MenuState::Main)` | Tier ordering and node-type shuffle. Single consumer: `generate_node_sequence_system`. |
| `NodeGenRng` | `derive_seed_named(derive_seed(run_seed, node_index), "node_gen")` | `OnEnter(NodeState::Loading)` (via `seed_node_rng`) | Frame selection, block selection, slot splitting, per-cell constraint resolution. |
| `BoltRng` | `derive_seed_named(derive_seed(run_seed, node_index), "bolt")` | `OnEnter(NodeState::Loading)` (via `seed_node_rng`) | Bolt launch and respawn angles. |
| `ChipRng` | `derive_seed(derive_seed_named(run_seed, "chip"), chip_select_count)` | `OnEnter(ChipSelectState::Selecting)` (via `reseed_chip_rng`) | Chip offering weighted selection. |
| `ProtocolRng` | `derive_seed(derive_seed_named(run_seed, "protocol"), protocol_offering_count)` | `OnEnter(ChipSelectState::Selecting)` (via `reseed_protocol_rng`) | Protocol offering selection. |
| `HazardRng` | `derive_seed(derive_seed_named(run_seed, "hazard"), tier_index)` | `OnEnter(HazardSelectState::Selecting)` (via `reseed_hazard_rng`) | Hazard offering selection, hazard wind direction, tether pair shuffle. |
| `EffectBaseSeed` | `derive_seed_named(run_seed, "effect")` | `OnExit(MenuState::Main)` (set once per run) | Root seed for ephemeral per-effect RNG. |
| `EffectEventCounter` | starts at `0` | `OnEnter(NodeState::Loading)` (via `seed_node_rng`) | Monotonic counter incremented per effect-fire dispatch. |

The ephemeral per-fire effect RNG is not a resource — `fire_dispatch` derives
a fresh `ChaCha8Rng` for each effect by combining `EffectBaseSeed` with the
current `EffectEventCounter` value via `derive_seed`, then increments the
counter. Each effect therefore consumes a deterministic, isolated stream
that is reproducible from `RunSeed` plus the node-deterministic effect
emission order.

## Seed Derivation Tree

```
run_seed (from RunSeed or OS entropy, captured at run start)
  │
  ├─ derive_seed_named(run_seed, "node_sequence")          → NodeSequenceRng
  │    └─ generate_node_sequence_system
  │
  ├─ derive_seed(run_seed, node_index)                     → per-node seed
  │    ├─ derive_seed_named(node_seed, "node_gen")         → NodeGenRng
  │    │    ├─ frame selection
  │    │    ├─ slot splitting decisions
  │    │    ├─ block selection per slot
  │    │    └─ constraint resolution per cell
  │    └─ derive_seed_named(node_seed, "bolt")             → BoltRng
  │         ├─ launch angle
  │         └─ respawn angles
  │
  ├─ derive_seed(derive_seed_named(run_seed, "chip"),
  │              chip_select_count)                        → ChipRng
  │    └─ chip offering weighted pick
  │
  ├─ derive_seed(derive_seed_named(run_seed, "protocol"),
  │              protocol_offering_count)                  → ProtocolRng
  │    └─ protocol offering pick from active pool
  │
  ├─ derive_seed(derive_seed_named(run_seed, "hazard"),
  │              tier_index)                               → HazardRng
  │    ├─ hazard offering pick
  │    ├─ drift wind direction
  │    └─ tether pair shuffle
  │
  └─ derive_seed_named(run_seed, "effect")                 → EffectBaseSeed
       └─ derive_seed(effect_base_seed, event_counter)     → per-fire RNG
            ├─ chain lightning targeting
            ├─ spawn-bolts angle
            ├─ phantom bolt angle
            ├─ chain bolt angle
            ├─ tether beam angles
            ├─ random effect selection
            └─ entropy engine selections
```

## Seed Derivation Helpers

Two `pub const fn` helpers live in `shared/rng.rs`:

```rust
pub const fn derive_seed(parent: u64, discriminator: u64) -> u64 {
    let mut h = parent
        .wrapping_add(discriminator)
        .wrapping_add(0x9E37_79B9_7F4A_7C15);
    h = h.wrapping_mul(0x517C_C1B7_2722_0A95);
    h ^ (h >> 32)
}

pub const fn derive_seed_named(parent: u64, name: &str) -> u64 {
    // Byte-wise multiplicative hash (acc * 31 + byte), then mix via derive_seed.
    // While-loop form is used because `<[u8]>::iter` is not yet const.
    // ...
    derive_seed(parent, name_hash)
}
```

The golden-ratio constant `0x9E37_79B9_7F4A_7C15` is added before the
multiplier so that `derive_seed(0, 0)` is non-zero and `derive_seed(u64::MAX,
u64::MAX)` differs from `derive_seed(0, 0)` — a `wrapping_add`-then-mix
pipeline avoids the boundary collision a pure XOR pipeline produces.

**Known property**: `derive_seed(a, b) == derive_seed(b, a)`. The combiner
is commutative in its two arguments. Channel separation comes from
`derive_seed_named` (the name byte-hash is keyed by position, not symmetric)
and from the ordering convention in the tree above (node-index inside,
channel-name outside), so the symmetry of `derive_seed` itself does not
introduce cross-channel collisions in practice.

## What the Run Seed Covers

Given the same `RunSeed`, the following are reproducible across runs:

- Node sequence (tier ordering, node type distribution, boss placement)
- Node composition (frame, block, slot splits, per-cell constraint resolution)
- Cell behaviours and layouts
- Bolt launch and respawn angles
- Chip offerings at each chip-select visit
- Protocol offerings at each protocol-select visit
- Hazard offerings at each tier boundary

NOT covered by design — these intentionally vary even between runs that
share a `RunSeed`:

- Effect fire outcomes that depend on world state at fire time (chain
  targets, spawn-bolt angles, random-effect picks, entropy-engine rolls).
  These are deterministic given identical player input but will differ
  between two players who share a seed and play differently.
- Visual jitter (popup positions, transition style) — `FxRng` is
  OS-entropy seeded; cosmetic variance is allowed.
- Scenario-runner input injection — the runner has its own `SmallRng`
  completely separate from gameplay RNG.
- OS entropy for unseeded runs — by design, every unseeded run is unique.

## Scenario Runner

The scenario runner forces `RunSeed(Some(scenario.seed))` via
`bypass_menu_to_playing` and triggers `OnExit(MenuState::Main)`. The Wave 2
ordering chain — `reset_run_state` → `capture_run_seed` →
`generate_node_sequence_system` — derives every sub-seed from `RunSeed`, so
scenarios are deterministic without the runner touching any domain-specific
RNG resource directly.

The runner's own `SmallRng` exists purely for input injection (key/mouse
event timing in chaos scenarios) and is completely separate from gameplay
RNG. Scenarios that hard-code expected outputs (target positions, effect
roll outcomes) may need re-recording once a seed-derivation change lands;
this is tracked as a Wave-3 follow-up rather than a writer-code concern
under the regular gameplay RNG migration.
